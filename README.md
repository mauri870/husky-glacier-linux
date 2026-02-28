# Husky Glacier Linux

> This is a Linux port of my HuskyGlacier Windows program.

# Installation

```bash
cp ./target/release/huskyglacier /usr/local/bin/
```

```bash
sudo useradd -r -s /usr/sbin/nologin husky
# add group husky too?
```

```bash
cp huskyglacier.service /etc/systemd/system/
systemctl daemon-reload
systemctl enable --now huskyglacier
```

```bash
cp 99-husky.rules /etc/udev/rules.d/99-husky.rules
sudo udevadm control --reload
sudo udevadm trigger
```