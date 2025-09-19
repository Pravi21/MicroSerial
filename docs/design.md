# MicroSerial Architecture Design

## 1. Overview

MicroSerial is a cross-platform serial communications toolchain split into a hardened C core and a modern Rust graphical client. The core delivers deterministic, low-latency serial I/O with a plugin-friendly architecture, while the GUI focuses on user experience, orchestration, and future extensibility. The project embraces a layered approach to keep responsibilities isolated and APIs stable.

```
+-------------------------------------------------------------+
|                         GUI (Rust)                          |
|  egui/eframe UI  |  Profile mgmt  |  Recorder  |  FFI layer |
+------------------+----------------+------------+------------+
          |                |               |          ^
          v                v               v          |
+-------------------------------------------------------------+
|                   Rust FFI Safe Wrapper Layer               |
|  Safe bindings to ms_core.h  |  Session orchestration        |
+------------------------------+------------------------------+
                              |
                              v
+-------------------------------------------------------------+
|                         Core (C11)                          |
|  util   |   io (serial, buffers)   | os (epoll/kqueue) | proto |
| logging |   ring buffers           | discovery          | ABI   |
+---------+--------------------------+--------------------+-------+
                              |
                              v
+-------------------------------------------------------------+
|                    Operating System Services                |
|  POSIX termios  |  epoll/kqueue  |  pthreads  |  Filesystem |
+-------------------------------------------------------------+
```

Key design tenets:

* **Isolation:** GUI never touches file descriptors directly; it can only interact through the stable `ms_core.h` API.
* **Throughput-first:** All data paths are zero-copy once inside the core; the GUI receives already-buffered slices.
* **Safety:** Strict validation, defensive defaults, and extensive error propagation guard against malformed input and privilege escalation.

## 2. Module Responsibilities

### Core (`core/`)

* **`io/`** – Serial session lifecycle, DMA-friendly ring buffers, async event loop, and flow-control aware write scheduling.
* **`os/`** – Platform shims (termios configuration, epoll/kqueue wiring, device discovery).
* **`util/`** – Logging, high-resolution time, lock-free SPSC ring buffer.
* **`proto/`** – Reserved for future protocol parsers and plugin dispatch.
* **`plugins/`** – Stable ABI for third-party protocol decoders, sandboxed via capability-aware callbacks.

Each exported function lives in its own header under `include/MicroSerial/functions/`, while module headers (e.g., `io/serial.h`) aggregate related calls to keep the public surface explicit and versionable.

### Rust GUI (`gui/`)

* **`core.rs`** – Safe wrapper around the generated FFI bindings. Handles pointer ownership, callback trampolines, and idiomatic errors.
* **`main.rs`** – egui/eframe powered desktop shell with port discovery, console view, transmit pane, and hot-refresh.
* **`build.rs`** – Invokes the CMake toolchain, compiles the core static library, and uses `bindgen` to regenerate bindings automatically.

### Plugins (`plugins/`)

* Placeholder for first-class decoders (COBS, SLIP, binary analyzers). Each plugin exports `ms_plugin_query_fn` returning metadata, init/shutdown hooks, and a pure function for decoding/transforming frames.

### Docs & Scripts

* **`docs/`** – Architecture, build, testing, security, and roadmap documentation.
* **`scripts/`** – Repeatable build/test automation entrypoints.

## 3. Threading & Event Model

1. **GUI thread (Rust)** – Handles user events, orchestrates sessions, drains message channels from the C callbacks, and keeps the UI responsive at 60 FPS.
2. **I/O thread (C)** – Spawned per open port. Uses epoll (Linux) or kqueue (macOS) to multiplex serial fd readiness with the wake pipe. The thread:
   * Reads available bytes into a stack buffer, pushes them into the RX ring, and immediately notifies registered callbacks.
   * Flushes TX ring data using batched writes when the device is writable, honoring flow control.
   * Emits state transitions and errors through the event callback.
3. **Producer/Consumer discipline** – Application code writes into the TX ring under a fast mutex, while the I/O thread reads and writes to the device. The RX ring is written by the I/O thread and consumable by higher-level parsers.

The wake pipe provides backpressure: writers poke the pipe to signal pending outbound data, and the event loop monitors the pipe alongside the serial fd.

## 4. Data Flow (Open → Configure → Read/Write → Close)

1. **Open** – `ms_serial_port_open` validates parameters, opens the fd with `O_NONBLOCK|O_NOCTTY`, prepares wake pipes, and initializes internal state.
2. **Configure** – `ms_serial_port_configure` applies sanitized settings via termios, configures flow control, and reinitializes RX/TX ring buffers sized per profile.
3. **Start** – `ms_serial_port_start` registers callbacks, arms epoll/kqueue, and launches the dedicated thread.
4. **I/O** – The event loop reads/writes in `4096` byte batches, guarding against EAGAIN/flow stalls and bubbling errors via callbacks.
5. **Write API** – `ms_serial_port_write` enqueues data atomically; partial writes return the number of bytes accepted so higher layers can backpressure.
6. **Close** – `ms_serial_port_stop` cancels the thread, drains the wake pipe, closes handles, and destroys buffers.

## 5. Plugin System

The C ABI is defined in `plugins/plugin_abi.h`. Plugins are shared objects providing:

* **Descriptor metadata** – Identifier, human-readable name, semantic version.
* **Lifecycle hooks** – `initialize` receives a restricted context (logging callback and ABI version); `shutdown` allows deterministic cleanup.
* **Decode entrypoint** – Stateless transform (`decode`) for converting raw frames into decoded payloads. Future extensions will provide structured metadata and sandbox policies (rate limits, capability flags).

The core will host a plugin manager responsible for discovery (`dlopen`/`dlsym`), ABI negotiation, and sandboxing (pledges, seccomp where available). Rust GUI modules will surface plugin output as alternate console views (hex, structured, graphs).

## 6. Security Posture

* **Defensive defaults** – No auto-execution, telemetry opt-in only, strict bounds checks, and immediate error bubbling.
* **Hardening flags** – `-fstack-protector-strong` and `_FORTIFY_SOURCE=2` enabled by default; ASan/UBSan optional toggles for CI.
* **Ownership clarity** – Each allocation has a single owner; ring buffers use atomics, not raw pointer arithmetic.
* **Input validation** – Termios parameters clamped, enumeration ignores non-existent files, and plugins run behind stable ABI boundaries.

## 7. Extensibility & Next Steps

* **Multi-port orchestration** – Extend the GUI to manage multiple open sessions with tabbed or tiled layouts.
* **Protocol decoders** – Implement bundled plugins (COBS, SLIP, Modbus) plus scripting hooks (Lua/Python) through the plugin ABI.
* **Recorder & exporter** – Stream RX data to `.bin` and `.pcapng` with JSON metadata, plus indexing for search.
* **Profiling and benchmarks** – Integrate Criterion-based harness for latency and throughput; document results in `docs/performance.md`.
* **Security hardening** – Sandboxed plugin host with seccomp/macOS sandbox profiles, threat model expansion, and fuzz coverage.

This design keeps the system maintainable, high-performance, and ready for rapid evolution into a professional-grade serial analysis suite.
