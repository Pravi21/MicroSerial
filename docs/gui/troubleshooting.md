# Troubleshooting

## Dark / blank window on Linux

1. **Check the diagnostics dialog** – it lists the active renderer and any fallback reasons.
2. **Force software rendering** – start the app with `MICROSERIAL_FORCE_SOFTWARE=1` or toggle *Appearance → Force software rendering* inside the GUI.
3. **Verify Wayland vs X11** – some older drivers only work on X11. Run `MICROSERIAL_FORCE_SOFTWARE=1` to confirm the UI renders correctly, then set `WINIT_UNIX_BACKEND=x11` to pin the compositor.
4. **Inspect driver logs** – DRI authentication failures typically stem from outdated `mesa` or missing permissions under `/dev/dri/*`.

## No devices listed

- Confirm the device is present under `/dev/tty*` (Linux) or `/dev/cu.*` (macOS).
- Ensure your user is in the appropriate serial group (see the [First Run](./first_run.md) guide).
- Use the **Refresh** button; scans are non-blocking and safe to trigger rapidly.
- The diagnostics panel exposes the last enumeration error code if the C core reported an issue.

## Repeated disconnects / resume from suspend

- The session layer listens for EIO/EAGAIN and restarts read loops automatically.
- If the USB hub powers down on suspend, unplugging and replugging should trigger an automatic rescan within ~4 seconds.

## Headless CI runs

- Use `cargo test --manifest-path gui/Cargo.toml -- --test-threads=1` with `MICROSERIAL_FORCE_SOFTWARE=1` to avoid any GPU reliance.
- A dedicated GitHub workflow (`gui-headless.yml`) runs the headless probe and unit tests under software rendering.

## Diagnostics bundle

The **Diagnostics** dialog aggregates:

- Renderer backend, adapter name/type, compositor, fallback reasons
- Settings relevant to rendering (force-software toggles)
- Last enumeration or session errors surfaced to the UI

Capture this information when filing bug reports to reduce turnaround time.
