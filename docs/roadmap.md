# MicroSerial Roadmap

## Short Term (0–3 months)

* **Protocol Views** – Hex/mixed inspectors, framing analyzers (ASCII, binary, COBS, SLIP) surfaced via plugin ABI.
* **Recorder MVP** – Lossless session capture, metadata JSON export, `.pcapng` writer for Wireshark interoperability.
* **Profiles & Presets** – Persisted device configurations with quick-switch palette and hotkeys.
* **CI Hardening** – Linux/macOS runners executing ASan/UBSan variants, fuzz harness via libFuzzer for parsers.

## Mid Term (3–6 months)

* **Multi-session UI** – Split-screen or tabbed layouts, synchronized timelines, aggregated logging.
* **Scripting Engine** – Sandboxed Lua or WASM environment for macros, triggers, and automation.
* **Visualization** – Inline oscilloscopes, traffic graphs, and throughput dashboards leveraging recorded metrics.
* **Security Sandbox** – seccomp-bpf profiles on Linux, macOS App Sandbox templates, capability-limited plugin contexts.

## Long Term (6–12 months)

* **Headless Daemon** – Core service exposing gRPC/REST for remote monitoring and integration with CI rigs.
* **Embedded Targets** – Cross-compilation story for ARM SBCs, packaging as snaps/AppImages/Homebrew formula.
* **Advanced Analytics** – Protocol-specific decoders (CAN, Modbus, LIN), anomaly detection, ML-assisted heuristics.
* **Enterprise Features** – Signed update channel, telemetry opt-in with transparent governance, role-based workspace sharing.
