# Rendering & Fallback

The GUI boots through a layered renderer selection pipeline that favours hardware acceleration on systems where it is stable while guaranteeing a software fallback when the GPU stack is unavailable or misconfigured.

```
┌────────────────────────────────────────────────┐
│ LaunchConfig (env, CLI, persisted settings)    │
└────────────────────────────────────────────────┘
                   │
                   ▼
      ┌────────────────────────────┐
      │ renderer::detect           │
      │ • honour force_software    │
      │ • record compositor info   │
      │ • probe wgpu adapters      │
      └────────────────────────────┘
                 │          │
         hardware OK        │ probe failed / CPU only
                 │          │
                 ▼          ▼
      ┌────────────────┐   ┌────────────────────┐
      │ RendererKind:: │   │ RendererKind::Glow │
      │ Wgpu           │   │ (software, GL)     │
      └────────────────┘   └────────────────────┘
                 │          │
                 └──────┬───┘
                        ▼
        renderer::force_glow (runtime fallback)
```

## Detection flow

1. **Launch configuration** collects signals from environment variables, command-line flags, and persisted settings. `MICROSERIAL_FORCE_SOFTWARE=1` or the in-app “Force software rendering” toggle short-circuit the GPU probe and immediately enable the software renderer while exporting `LIBGL_ALWAYS_SOFTWARE=1` for downstream drivers.
2. **`renderer::detect`** records the active compositor (Wayland or X11) and attempts to obtain a `wgpu` adapter. If a non-CPU adapter is found, the app boots with the `wgpu` backend. Otherwise it falls back to the software (`glow`) renderer and marks diagnostics with the failure reason.
3. **Runtime fallback**: if the initial `eframe::run_native` call fails (e.g. DRI authentication or EGL creation errors), the launcher retries automatically with `renderer::force_glow`, ensuring a visible window even on machines without a functioning GPU stack.

## Rationale

- **Predictability:** All renderer decisions are centralised in `renderer.rs`, making it trivial to unit-test the decision matrix and to expose the chosen backend via the diagnostics panel.
- **Graceful failure:** The retry loop isolates GPU driver crashes from the rest of the application. Users receive a working (software-rendered) UI instead of a black window, and diagnostics clearly document the fallback path.
- **Cross-platform compliance:** The approach works across Wayland/X11 on Linux and Metal/ANGLE surfaces on macOS by relying on `wgpu` for hardware rendering and the `glow` backend for CPU fallback.
- **Operator control:** Environment toggles (`MICROSERIAL_FORCE_SOFTWARE`, `LIBGL_ALWAYS_SOFTWARE`) and the in-app switch empower operators and CI to pick a deterministic backend. Headless/CI jobs run with software rendering forced, guaranteeing reproducible results.

## Diagnostics & telemetry

The diagnostics dialog surfaces:

- Selected backend (wgpu vs glow)
- Adapter name, type, and driver string when hardware rendering is active
- Whether the compositor is Wayland or X11
- Flags indicating forced software rendering (environment or user setting)
- Failure reasons when the fallback path is engaged

A `--headless-detect` CLI mode reuses the same pipeline to print renderer information without spawning a window, enabling automated health checks.
