BINARY  = huskyglacier
BINDIR  = /usr/local/bin
SYSTEMD = /etc/systemd/system
UDEVDIR = /etc/udev/rules.d

.PHONY: all install uninstall

all:
	cargo build --release

install:
	@echo ">>> Installing binary"
	install -Dm755 target/release/$(BINARY) $(BINDIR)/$(BINARY)

	@echo ">>> Creating system group and user"
	getent group  $(BINARY) > /dev/null 2>&1 || groupadd -r $(BINARY)
	id -u         $(BINARY) > /dev/null 2>&1 || useradd -r -M -s /usr/sbin/nologin -g $(BINARY) $(BINARY)

	@echo ">>> Installing udev rules"
	install -Dm644 99-$(BINARY).rules $(UDEVDIR)/99-$(BINARY).rules
	udevadm control --reload
	udevadm trigger --subsystem-match=hidraw

	@echo ">>> Installing and enabling systemd service"
	install -Dm644 $(BINARY).service $(SYSTEMD)/$(BINARY).service
	systemctl daemon-reload
	systemctl enable --now $(BINARY)

	@echo ">>> Done. Logs: journalctl -u $(BINARY).service -f"

uninstall:
	@echo ">>> Stopping and disabling service"
	systemctl disable --now $(BINARY) 2>/dev/null || true

	@echo ">>> Removing files"
	rm -f $(BINDIR)/$(BINARY)
	rm -f $(SYSTEMD)/$(BINARY).service
	rm -f $(UDEVDIR)/99-$(BINARY).rules

	@echo ">>> Reloading systemd and udev"
	systemctl daemon-reload
	udevadm control --reload

	@echo ">>> Removing system user and group"
	userdel  $(BINARY) 2>/dev/null || true
	groupdel $(BINARY) 2>/dev/null || true

	@echo ">>> Uninstall complete"
