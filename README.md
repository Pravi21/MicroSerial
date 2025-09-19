# MicroSerial

MicroSerial is a high-performance serial terminal stack that combines a hardened C11 core with a polished Rust GUI. It delivers deterministic low-latency I/O, plugin-ready extensibility, and a delightful operator experience on Linux and macOS.

## Repository Layout

```
core/      # C core: async serial engine, ring buffers, OS shims
gui/       # Rust/egui desktop application with FFI wrappers
plugins/   # Future protocol/decoder plugins (stable C ABI)
docs/      # Architecture, build guides, roadmap, threat model
scripts/   # Build helpers
tests/     # Integration & fuzz harness entrypoints
```

## Quick Start

1. **Build everything:** `./scripts/build_all.sh`
2. **Run core tests:**
   ```
   cmake -S core -B build/core
   cmake --build build/core
   ctest --test-dir build/core
   ```
3. **Launch the GUI:** `cargo run --manifest-path gui/Cargo.toml`

The GUI auto-discovers serial ports, opens a dedicated async session, and renders RX/TX streams in real time. For loopback demos without hardware, pair pseudo-terminals with `socat` or rely on the built-in integration test.

## Platform Setup & Detailed Build Instructions

### Linux

1. **Install prerequisites** (example for Debian/Ubuntu):
   ```bash
   sudo apt update
   sudo apt install build-essential cmake ninja-build pkg-config libudev-dev libclang-dev
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
   source "$HOME/.cargo/env"
   ```
   *CMake* drives the C11 core build, `libclang` powers the Rust FFI bindings, and `pkg-config`/`libudev` provide serial device discovery headers.
2. **Build the C core:**
   ```bash
   cmake -S core -B build/core -DCMAKE_BUILD_TYPE=Release
   cmake --build build/core --config Release
   ctest --test-dir build/core
   ```
3. **Compile the Rust GUI:**
   ```bash
   cargo build --manifest-path gui/Cargo.toml
   ```
   The GUI crate regenerates bindings and links against the freshly built `microserial_core` static library. The `eframe` dependency enables both Wayland and X11 window backends so either display server is supported out of the box.
4. **Run it:** `cargo run --manifest-path gui/Cargo.toml`

### macOS

1. **Install developer tools:**
   ```bash
   xcode-select --install
   /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
   brew install cmake ninja pkg-config llvm rustup
   rustup-init -y
   source "$HOME/.cargo/env"
   ```
   Homebrew supplies a modern Clang toolchain used by both the C core and `bindgen`.
2. **Export LLVM for bindgen** (needed when Homebrew installs LLVM outside of `/usr/bin`):
   ```bash
   export LIBCLANG_PATH="$(brew --prefix llvm)/lib"
   export PATH="$(brew --prefix llvm)/bin:$PATH"
   ```
3. **Build core + GUI:** reuse the same `cmake` and `cargo` commands as on Linux. `./scripts/build_all.sh` also works inside a POSIX shell on macOS.
4. **Run the app:** `cargo run --manifest-path gui/Cargo.toml`

### Windows (MSVC toolchain)

1. **Install dependencies:**
   * [Visual Studio Build Tools 2022](https://visualstudio.microsoft.com/downloads/) with the “Desktop development with C++” workload.
   * [CMake](https://cmake.org/download/), [Ninja](https://github.com/ninja-build/ninja/releases), and the [LLVM](https://releases.llvm.org/download.html) binaries.
   * [rustup-init.exe](https://win.rustup.rs/) to install the stable Rust toolchain (`rustup default stable`).
2. **Open the “x64 Native Tools Command Prompt for VS 2022”** so MSVC, CMake, and Ninja are on `PATH`. If LLVM is installed in a custom directory, set `LIBCLANG_PATH=C:\\Program Files\\LLVM\\bin` before building.
3. **Configure and build the core:**
   ```powershell
   cmake -S core -B build\core -G "Ninja" -DCMAKE_BUILD_TYPE=Release
   cmake --build build\core --config Release
   ctest --test-dir build\core
   ```
4. **Build the GUI:**
   ```powershell
   cargo build --manifest-path gui/Cargo.toml
   ```
   The Rust build script invokes CMake automatically, links against the MSVC-built static core, and bundles the resulting executable under `gui\target\debug\microserial_gui.exe` (or `...\release\`).
5. **Run the GUI:** `cargo run --manifest-path gui/Cargo.toml`

> **Tip:** On any platform you can rerun everything in one shot via `./scripts/build_all.sh` (or `bash scripts/build_all.sh` on Windows when using Git Bash).

## Documentation

* [Design Document](docs/design.md)
* [Build Guide](docs/build.md)
* [Roadmap](docs/roadmap.md)

## License

Licensed under the MIT License. See [LICENSE](LICENSE).
