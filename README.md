# Husky Glacier Linux

> Linux port of HuskyGlacier — reads CPU temperature via sysfs hwmon and forwards it to the Husky Glacier HWT700PT pump over USB HID.

## Requirements

- Rust toolchain (`cargo`)
- A supported CPU temperature driver: `coretemp`, `k8temp`, `k10temp`, or `zenpower`

## Installation

```bash
make
sudo make install
```

This will:
1. Build the release binary
2. Create a dedicated `husky` system user and group
3. Install the udev rule so the pump is accessible to the `husky` group
4. Install, enable, and start the systemd service

## Logs

```bash
journalctl -u huskyglacier.service -f
```

## Uninstall

```bash
sudo make uninstall
```
