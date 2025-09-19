# Build & Run Guide

## Prerequisites

* **Linux or macOS** with POSIX toolchain
* CMake ≥ 3.18, Clang/GCC with C11 support
* Rust toolchain (stable) + Cargo
* `pkg-config`, `clang`, and development headers for X11/Wayland (auto-pulled by egui/wgpu)

## Build the C Core

```
cmake -S core -B build/core -DCMAKE_BUILD_TYPE=Release
cmake --build build/core --config Release
ctest --test-dir build/core
```

## Build the GUI (Rust)

The Rust crate drives CMake automatically through `build.rs`, compiling the static core and regenerating FFI bindings.

```
cargo build --manifest-path gui/Cargo.toml
```

The binary lives under `target/debug/microserial_gui` (or `target/release/...`).

## Combined Build Script

```
./scripts/build_all.sh
```

## Running the Loopback Demo

The integration test uses a pseudo-terminal pair to emulate a serial cable. Run it via CTest or directly:

```
ctest --test-dir build/core -R serial_loopback_test
```

For an interactive UI smoke-test without hardware, start the GUI and select one of the pseudo-terminals exposed under `/dev/pts/*`. Use `socat -d -d pty,raw,echo=0 pty,raw,echo=0` to create loopback pairs for experimentation.

## Tooling

* `clang-format` / `clang-tidy` – run against `core/`
* `rustfmt` / `cargo clippy` – run within `gui/`
* `scripts/build_all.sh` – builds both layers reproducibly

CI recipes (GitHub Actions) should execute the same steps on Linux and macOS runners.
