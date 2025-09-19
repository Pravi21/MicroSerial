# Troubleshooting

## Package manager issues

* **Winget reports missing source agreements.** Run `winget source update` once as an administrator and accept the default sources.
* **Homebrew refuses to run under sudo.** Use a standard user shell and let the script escalate where necessary; Homebrew maintains its own permissions.
* **Corporate mirrors/proxies.** Export `HTTPS_PROXY` / `HTTP_PROXY` before running the scripts. Package managers inherit these variables.

## Toolchain detection failures

* **VSDevCmd missing after Visual Studio install.** Launch the Visual Studio Installer and ensure "Desktop development with C++" components are present. The script looks for `VsDevCmd.bat` inside `Common7\Tools`.
* **pkg-config not found on Windows.** Strawberry Perl and Chocolatey both provide a `pkg-config.exe`. After installation ensure `C:\Strawberry\c\bin` or the Chocolatey shims folder is on `PATH`.
* **Rust stable toolchain missing.** Run `rustup toolchain install stable` manually and re-run the bootstrap with `--build`.

## Build problems

* **CMake cannot find Ninja.** Verify Ninja is on the `PATH`. Re-run the bootstrap with `--force` to reinstall the toolchain.
* **Cargo build fails to link against MSVC.** Ensure the Windows SDK and MSVC components are installed. `--force` on Windows will re-run the Visual Studio installer with the required workloads.
* **Gatekeeper blocks the macOS app.** Codesigning/notarization are optional; for local testing run `xattr -rd com.apple.quarantine gui/target/release/microserial_gui`.

## Cleanup

Run `./scripts/<os>/bootstrap --uninstall` to remove build outputs. Toolchains installed via system package managers should be removed with those same managers.
