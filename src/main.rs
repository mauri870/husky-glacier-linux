use regex::Regex;
use hidapi::{HidApi, HidDevice};

fn main() {
    // Husky Glacier HWT700PT pump
    const PUMPVID: i32 = 0xAA88;
    const PUMPPID: i32 = 0x8666;

    let cpu_sensor_path = find_cpu_temp_sensor(0).unwrap();

    let hid_device = HidApi::new()
        .and_then(|api| api.open(PUMPVID as u16, PUMPPID as u16))
        .expect("Failed to open HID device");

    loop {
        let temp  = get_cpu_temp(&cpu_sensor_path);
        send_temp_to_pump(&hid_device, temp);
        println!("CPU Temp: {}°C", temp);
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

fn get_cpu_temp(cpu_sensor_path: &str) -> u8 {
    let temp_str = std::fs::read_to_string(&cpu_sensor_path).unwrap_or_default();
    let temp = temp_str.trim().parse::<u32>().unwrap_or(0);
    (temp / 999) as u8
}

fn send_temp_to_pump(device: &HidDevice, temp: u8) {
    let mut report: [u8; 10] = [0; 10];
    report[1] = temp;
    device.write(&report).expect("Failed to write to HID device");
}

// search in /sys/class/hwmon for a suitable cpu temp sensor file
//
// Ported from C++ to rust, origin code from CPU-X project.
// See https://github.com/TheTumultuousUnicornOfDarkness/CPU-X/blob/84f2da456e57898e5f6655794b2ff0712f41b8c9/src/util.cpp#L507-574 
fn find_cpu_temp_sensor(core_id: u16) -> Option<String> {
    let re_temp_label  = Regex::new(r"temp\d+_label").unwrap();
    let re_temp1_input = Regex::new(r"temp1_input").unwrap();
    let re_core_n      = Regex::new(&format!(r"(?i)Core\s*{}", core_id)).unwrap();
    let re_tdie        = Regex::new(r"(?i)Tdie").unwrap();

    // (driver_name, filename_regex, optional_label_regex)
    // Mirrors the C++ drivers unordered_multimap:
    //   coretemp -> tempN_label containing "Core <id>"
    //   k8temp   -> tempN_label containing "Core <id>"
    //   k10temp  -> tempN_label containing "Tdie"
    //   k10temp  -> temp1_input directly
    //   zenpower -> temp1_input directly
    let drivers: &[(&str, &Regex, Option<&Regex>)] = &[
        ("coretemp", &re_temp_label,  Some(&re_core_n)),
        ("k8temp",   &re_temp_label,  Some(&re_core_n)),
        ("k10temp",  &re_temp_label,  Some(&re_tdie)),
        ("k10temp",  &re_temp1_input, None),
        ("zenpower", &re_temp1_input, None),
    ];

    for entry in std::fs::read_dir("/sys/class/hwmon").ok()?.flatten() {
        let driver_name = std::fs::read_to_string(entry.path().join("name"))
            .unwrap_or_default()
            .trim()
            .to_string();

        if !drivers.iter().any(|(d, _, _)| *d == driver_name) {
            continue;
        }

        for (drv, re_filename, re_label) in drivers {
            if *drv != driver_name {
                continue;
            }

            for sub_entry in std::fs::read_dir(entry.path())
                .into_iter()
                .flatten()
                .flatten()
            {
                let sensor_path = sub_entry.path().to_string_lossy().into_owned();

                if !re_filename.is_match(&sensor_path) {
                    continue;
                }

                let result = if let Some(label_re) = re_label {
                    let label = std::fs::read_to_string(&sensor_path).unwrap_or_default();
                    if label_re.is_match(&label) {
                        Some(sensor_path.replace("_label", "_input"))
                    } else {
                        None
                    }
                } else {
                    Some(sensor_path)
                };

                if result.is_some() {
                    return result;
                }
            }
        }
    }

    None
}
