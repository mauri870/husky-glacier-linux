# Husky Glacier Linux

Linux port of my [Husky Glacier Windows App](https://github.com/mauri870/HuskyGlacier). Refer to the original project for details.

Tested with a water cooler pump reported as USB Vendor ID and Product ID `aa88:8666 (铭研科技 温度显示HID设备)`.

## Requirements

- Rust toolchain (`cargo`)

## Installation

```bash
make
sudo make install
```

This will:
1. Build the release binary
2. Create a dedicated `huskyglacier` system user and group
3. Install the udev rule so the pump is accessible to the `huskyglacier` group
4. Install, enable, and start the systemd service

## Logs

```bash
journalctl -u huskyglacier.service -f
```

## Uninstall

```bash
sudo make uninstall
```
