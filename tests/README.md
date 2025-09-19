# Testing Strategy

* **Unit tests (C)** – Added via CTest under `core/tests/`, covering pseudo-terminal loopback flows.
* **Rust unit/integration tests** – To be added for GUI state reducers and FFI wrappers (`cargo test`).
* **Fuzzing** – Planned for protocol parsers (libFuzzer + honggfuzz harness).
* **Benchmarking** – Dedicated suite will measure throughput and latency using loopback harnesses and high-resolution timers.
