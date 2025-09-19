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

## Documentation

* [Design Document](docs/design.md)
* [Build Guide](docs/build.md)
* [Roadmap](docs/roadmap.md)

## License

Licensed under the MIT License. See [LICENSE](LICENSE).
