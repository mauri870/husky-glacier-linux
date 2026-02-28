use hidapi::{HidApi, HidDevice};
use log::{error, info, warn};
use regex::Regex;
use std::sync::LazyLock;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Log directly to the systemd journal with proper priority metadata.
    systemd_journal_logger::JournalLog::new()
        .map_err(|e| format!("Failed to create journal logger: {}", e))?
        .install()
        .map_err(|e| format!("Failed to install journal logger: {}", e))?;
    log::set_max_level(log::LevelFilter::Info);

    // Husky Glacier HWT700PT pump
    const PUMPVID: u16 = 0xAA88;
    const PUMPPID: u16 = 0x8666;

    let cpu_sensor_path = match find_cpu_temp_sensor(0) {
        Some(path) => {
            info!("Using CPU temperature sensor: {}", path);
            path
        }
        None => {
            error!("No CPU temperature sensor found in /sys/class/hwmon");
            return Err("No CPU temperature sensor found".into());
        }
    };

    let api = HidApi::new().map_err(|e| {
        error!("Failed to initialize HID API: {}", e);
        e
    })?;

    let hid_device = api.open(PUMPVID, PUMPPID).map_err(|e| {
        error!("Failed to open HID device {:04X}:{:04X}: {}", PUMPVID, PUMPPID, e);
        e
    })?;

    info!("Connected to pump device {:04X}:{:04X}", PUMPVID, PUMPPID);

    loop {
        let temp = get_cpu_temp(&cpu_sensor_path);
        if let Err(e) = send_temp_to_pump(&hid_device, temp) {
            error!("Failed to send temperature to pump: {}", e);
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

fn get_cpu_temp(cpu_sensor_path: &str) -> u8 {
    let temp_str = match std::fs::read_to_string(cpu_sensor_path) {
        Ok(s) => s,
        Err(e) => {
            warn!("Failed to read CPU temperature sensor '{}': {}", cpu_sensor_path, e);
            return 0;
        }
    };
    match temp_str.trim().parse::<u32>() {
        Ok(raw) => (raw / 999) as u8,
        Err(e) => {
            warn!("Failed to parse temperature value from '{}': {}", cpu_sensor_path, e);
            0
        }
    }
}

fn send_temp_to_pump(device: &HidDevice, temp: u8) -> Result<(), hidapi::HidError> {
    let mut report: [u8; 10] = [0; 10];
    report[1] = temp;
    device.write(&report)?;
    Ok(())
}

static RE_TEMP_LABEL:  LazyLock<Regex> = LazyLock::new(|| Regex::new(r"temp\d+_label").unwrap());
static RE_TEMP1_INPUT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"temp1_input").unwrap());
static RE_TDIE:        LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)Tdie").unwrap());

// Search in /sys/class/hwmon for a suitable CPU temp sensor file.
//
// Ported from C++ to Rust, original code from CPU-X project.
// See https://github.com/TheTumultuousUnicornOfDarkness/CPU-X/blob/84f2da456e57898e5f6655794b2ff0712f41b8c9/src/util.cpp#L507-574
fn find_cpu_temp_sensor(core_id: u16) -> Option<String> {
    // core_id is dynamic so this regex can't be a static.
    let re_core_n = match Regex::new(&format!(r"(?i)Core\s*{}", core_id)) {
        Ok(r) => r,
        Err(e) => {
            error!("Failed to compile core-id regex: {}", e);
            return None;
        }
    };

    // (driver_name, filename_regex, optional_label_regex)
    // Mirrors the C++ drivers unordered_multimap:
    //   coretemp -> tempN_label containing "Core <id>"
    //   k8temp   -> tempN_label containing "Core <id>"
    //   k10temp  -> tempN_label containing "Tdie"
    //   k10temp  -> temp1_input directly
    //   zenpower -> temp1_input directly
    let drivers: &[(&str, &Regex, Option<&Regex>)] = &[
        ("coretemp", &RE_TEMP_LABEL,  Some(&re_core_n)),
        ("k8temp",   &RE_TEMP_LABEL,  Some(&re_core_n)),
        ("k10temp",  &RE_TEMP_LABEL,  Some(&RE_TDIE)),
        ("k10temp",  &RE_TEMP1_INPUT, None),
        ("zenpower", &RE_TEMP1_INPUT, None),
    ];

    let hwmon_dir = match std::fs::read_dir("/sys/class/hwmon") {
        Ok(d) => d,
        Err(e) => {
            error!("Failed to read /sys/class/hwmon: {}", e);
            return None;
        }
    };

    for entry in hwmon_dir.flatten() {
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
