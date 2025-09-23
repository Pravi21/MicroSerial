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
                  │
                  ▼
     ┌────────────────────────────┐
     │  renderer attempt sequence │
     │  1. system defaults (wgpu) │
     │  2. WGPU_BACKEND=vulkan    │
     │  3. WGPU_BACKEND=gl + SW   │
     │  4. eframe glow (OpenGL)   │
     │  5. WGPU_BACKEND=metal (*) │
     └────────────────────────────┘
                  │
                  ▼
          eframe::run_native (wgpu/glow)

(*) macOS only
```

## Detection flow

1. **Launch configuration** collects signals from environment variables, command-line flags, and persisted settings. `MICROSERIAL_FORCE_SOFTWARE=1`, the CLI `--force-software` flag, or the in-app toggle call `LaunchConfig::enable_force_software`, exporting `LIBGL_ALWAYS_SOFTWARE=1` and `WGPU_POWER_PREF=low_power` before any adapter probe runs.
2. **`renderer::detect`** records the active compositor (Wayland or X11) and attempts to obtain a `wgpu` adapter while iterating through the fallback chain. Each step sets `WGPU_BACKEND`/`LIBGL_ALWAYS_SOFTWARE` as required before probing.
3. **Runtime fallback**: if `eframe::run_native` still fails to build a surface with the chosen backend, the launcher advances to the next attempt in the chain without ever touching glutin. On non-Windows platforms the "software" steps rely on Mesa's llvmpipe/Zink stack or pure OpenGL contexts, so the launcher avoids requesting wgpu's explicit fallback adapter (which only exists on Windows and would prevent the Mesa adapters from being enumerated) and finally falls back to `eframe`'s built-in `glow` renderer.

## Rationale

- **Predictability:** All renderer decisions are centralised in `renderer.rs`, making it trivial to unit-test the decision matrix and to expose the chosen backend via the diagnostics panel.
- **Graceful failure:** The retry loop isolates GPU driver crashes from the rest of the application. Users receive a working UI instead of a crash, and diagnostics clearly document the fallback path.
- **Cross-platform compliance:** The approach works across Wayland/X11 on Linux and Metal on macOS via the pure `wgpu` backend when available, while retaining an OpenGL escape hatch (`glow`) for environments where EGL/DRI access is blocked (e.g. VMs, containers).
- **Operator control:** Environment toggles (`MICROSERIAL_FORCE_SOFTWARE`, `LIBGL_ALWAYS_SOFTWARE`, `WGPU_BACKEND`) and the in-app switch empower operators and CI to pick a deterministic backend. Headless/CI jobs run with software rendering forced, guaranteeing reproducible results.

## Diagnostics & telemetry

The diagnostics dialog surfaces:

- Selected backend (Vulkan/GL/Metal)
- Adapter name, type, and driver string when hardware rendering is active
- Whether the compositor is Wayland or X11
- Flags indicating forced software rendering (environment or user setting)
- Failure reasons when the fallback path is engaged

A `--headless-detect` CLI mode reuses the same pipeline to print renderer information without spawning a window, enabling automated health checks.
