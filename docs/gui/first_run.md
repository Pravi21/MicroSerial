# First Run Checklist

MicroSerial’s GUI is designed to boot safely even when no serial hardware is attached. The first launch presents a welcoming empty-state panel with a prominent **Refresh** button and a help link. Use the steps below to prepare your system.

## Linux (Wayland & X11)

1. **Add udev permissions** – ensure your user is part of the `dialout` (Debian/Ubuntu) or `uucp` (Arch) group.
2. **Reload udev rules** if you changed group membership: `sudo udevadm control --reload-rules && sudo udevadm trigger`.
3. **Hot-plug support** – the device list refreshes automatically every few seconds. Use the refresh button to force an immediate scan.
4. **Headless mode** – export `MICROSERIAL_FORCE_SOFTWARE=1` to guarantee software rendering on minimal VMs or CI.

## macOS

1. When the application first accesses serial devices macOS will present a privacy prompt. Accept it to authorise the process.
2. If the prompt is dismissed accidentally, grant access under **System Settings → Privacy & Security → Input Monitoring**.
3. Use the **Diagnostics** dialog to verify whether Metal or the software renderer is active.

## Empty state UX

- When no `/dev/tty*` (Linux) or `/dev/cu.*` (macOS) devices are detected the console remains active, and a centered card explains the situation with links to these instructions.
- The **Connect** button stays disabled until a port is selected, preventing accidental attempts against “phantom” devices.

## Keyboard shortcuts

- **Enter** in the send panel transmits the current payload.
- **Ctrl+L / Cmd+L** clears the console (available via the context menu).
- **Ctrl+R / Cmd+R** triggers a device rescan.

## Profiles & settings persistence

Profiles store the full serial configuration. Create as many as needed (e.g. “Bootloader”, “Firmware test”) and switch instantly without losing console history. Settings live under the platform’s configuration directory (override with `MICROSERIAL_CONFIG_DIR` for testing).
