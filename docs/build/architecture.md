# Bootstrap Architecture

```
+---------------------+        +--------------------+
| scripts/common/*    |        | docs/build/*       |
|  - shlib.sh         |        |  - requirements    |
|  - pslib.ps1        |        |  - troubleshooting |
+----------+----------+        +---------+----------+
           |                            |
           v                            v
+----------+----------+        +-------+-------+
| OS Front Ends       |        | CI Integration|
|  linux/bootstrap.sh |        |  GitHub matrix|
|  macos/bootstrap.sh |        +---------------+
|  windows/*.ps1      |
+----------+----------+
           |
           v
+----------+-----------+
| Audit → Install → Build|
|  version detectors     |
|  package planners      |
|  cmake/cargo drivers   |
+------------------------+
```

## Shared helpers

* **Logging & reporting.** `shlib.sh` and `pslib.ps1` centralise colourised console output, structured logging (`build/logs/*.log`),
  and the preflight report accumulator.
* **Version comparison.** POSIX shells rely on an AWK-based semantic comparator; PowerShell uses `[version]` parsing.
* **Dry-run/offline enforcement.** Helper functions guard every shell command, aborting early when `--dry-run` or `--offline`
  disallow execution.

## OS-specific strategy

### Linux

* Detects package manager (`apt`, `dnf`, `pacman`) and maps requirement tokens to distro packages.
* Uses `gcc -dumpfullversion`/`clang --version` and `pkg-config --modversion libudev` for precise checks.
* Ensures `rustup` stable toolchain and the `x86_64-unknown-linux-gnu` target, running `rustup default stable` when needed.

### macOS

* Requires Xcode Command Line Tools via `xcode-select -p`; warns if missing because installation is interactive.
* Prefers Homebrew packages (`rustup-init`, `cmake`, `ninja`, `pkg-config`, `make`).
* Detects the active architecture to add the appropriate Rust target (`aarch64-apple-darwin` or `x86_64-apple-darwin`).

### Windows

* Locates MSVC Build Tools and Windows SDK with `vswhere` and registry lookups. Missing components trigger a `winget`
  installation with required workloads.
* Chooses between `winget`, `choco`, and `scoop` per token. `VsDevCmd.bat` seeds the environment for CMake and Cargo builds.
* Ensures the Rust MSVC toolchain and target (`x86_64-pc-windows-msvc` or `aarch64-pc-windows-msvc`).

## Build orchestration

After dependencies are satisfied all scripts run the same two-stage build:

1. `cmake -S core -B build/core -G Ninja -DCMAKE_BUILD_TYPE=Release` followed by `cmake --build`.
2. `cargo build --manifest-path gui/Cargo.toml --release`.

Artifact checks ensure `libmicroserial_core.{a,lib}` and `microserial_gui[.exe]` exist before reporting success.
