use std::env;
use std::fmt;
use std::time::Instant;

use wgpu::AdapterInfo;

#[derive(Clone)]
pub struct RendererSelection {
    pub attempt: usize,
    pub attempt_label: &'static str,
    pub options: eframe::NativeOptions,
    pub diagnostics: RendererDiagnostics,
}

impl RendererSelection {
    fn new_wgpu(
        attempt: usize,
        attempt_label: &'static str,
        diagnostics: RendererDiagnostics,
    ) -> Self {
        let mut options = eframe::NativeOptions::default();
        options.renderer = eframe::Renderer::Wgpu;
        if diagnostics.software_backend {
            options.hardware_acceleration = eframe::HardwareAcceleration::Off;
        } else {
            options.hardware_acceleration = eframe::HardwareAcceleration::Preferred;
        }
        Self {
            attempt,
            attempt_label,
            options,
            diagnostics,
        }
    }

    fn new_glow(
        attempt: usize,
        attempt_label: &'static str,
        mut diagnostics: RendererDiagnostics,
    ) -> Self {
        diagnostics.backend = "glow".to_string();
        diagnostics.backend_details = Some("Software renderer (glow fallback)".to_string());
        diagnostics.adapter_type = Some("Cpu".to_string());
        diagnostics.adapter_name = diagnostics
            .adapter_name
            .or_else(|| Some("Software Renderer".to_string()));

        let mut options = eframe::NativeOptions::default();
        options.renderer = eframe::Renderer::Glow;
        options.hardware_acceleration = eframe::HardwareAcceleration::Off;
        Self {
            attempt,
            attempt_label,
            options,
            diagnostics,
        }
    }

    fn new_glow(
        attempt: usize,
        attempt_label: &'static str,
        mut diagnostics: RendererDiagnostics,
    ) -> Self {
        diagnostics.backend = "glow".to_string();
        diagnostics.backend_details = Some("Software renderer (glow fallback)".to_string());
        diagnostics.adapter_type = Some("Cpu".to_string());
        diagnostics.adapter_name = diagnostics
            .adapter_name
            .or_else(|| Some("Software Renderer".to_string()));

        let mut options = eframe::NativeOptions::default();
        options.renderer = eframe::Renderer::Glow;
        options.hardware_acceleration = eframe::HardwareAcceleration::Off;
        Self {
            attempt,
            attempt_label,
            options,
            diagnostics,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RendererDiagnostics {
    pub forced_software: bool,
    pub env_forced: bool,
    pub fallback_used: bool,
    pub software_backend: bool,
    pub compositor: Option<String>,
    pub backend: String,
    pub adapter_name: Option<String>,
    pub adapter_type: Option<String>,
    pub backend_details: Option<String>,
    pub failure_reason: Option<String>,
    pub started_at: Instant,
}

impl Default for RendererDiagnostics {
    fn default() -> Self {
        Self {
            forced_software: false,
            env_forced: false,
            fallback_used: false,
            software_backend: false,
            compositor: None,
            backend: String::from("wgpu"),
            adapter_name: None,
            adapter_type: None,
            backend_details: None,
            failure_reason: None,
            started_at: Instant::now(),
        }
    }
}

impl RendererDiagnostics {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone)]
pub struct LaunchConfig {
    pub force_software: bool,
    pub headless: bool,
    pub env_forced: bool,
    original_backend: Option<String>,
    original_libgl: Option<String>,
    original_power_pref: Option<String>,
}

impl LaunchConfig {
    pub fn from_args() -> Self {
        let original_backend = env::var("WGPU_BACKEND").ok();
        let original_libgl = env::var("LIBGL_ALWAYS_SOFTWARE").ok();
        let original_power_pref = env::var("WGPU_POWER_PREF").ok();

        let env_forced = env_flag("MICROSERIAL_FORCE_SOFTWARE");
        let mut config = Self {
            force_software: false,
            headless: false,
            env_forced,
            original_backend,
            original_libgl,
            original_power_pref,
        };

        if env_forced {
            config.enable_force_software();
        }

        for arg in env::args().skip(1) {
            match arg.as_str() {
                "--force-software" => config.enable_force_software(),
                "--headless-detect" => config.headless = true,
                _ => {}
            }
        }

        config
    }

    pub fn enable_force_software(&mut self) {
        self.force_software = true;
        unsafe {
            env::set_var("LIBGL_ALWAYS_SOFTWARE", "1");
            env::set_var("WGPU_POWER_PREF", "low_power");
        }
    }

    pub fn attempt_count(&self) -> usize {
        self.fallback_plan().len()
    }

    fn fallback_plan(&self) -> Vec<Attempt> {
        let mut attempts = vec![Attempt::system_default(), Attempt::vulkan()];
        attempts.push(Attempt::gl_software());
        #[cfg(target_os = "macos")]
        attempts.push(Attempt::metal());
        attempts.push(Attempt::glow_software());
        attempts
    }
}

fn env_flag(key: &str) -> bool {
    matches!(env::var(key), Ok(v) if v == "1" || v.eq_ignore_ascii_case("true"))
}

#[derive(Clone, Copy)]
enum Attempt {
    Wgpu {
        label: &'static str,
        backend: Option<&'static str>,
        enforce_software: bool,
    },
    GlowSoftware {
        label: &'static str,
    },
}

impl Attempt {
    fn system_default() -> Self {
        Self::Wgpu {
            label: "system default",
            backend: None,
            enforce_software: false,
        }
    }

    fn vulkan() -> Self {
        Self::Wgpu {
            label: "WGPU_BACKEND=vulkan",
            backend: Some("vulkan"),
            enforce_software: false,
        }
    }

    fn gl_software() -> Self {
        Self::Wgpu {
            label: "WGPU_BACKEND=gl (software)",
            backend: Some("gl"),
            enforce_software: true,
        }
    }

    #[cfg(target_os = "macos")]
    fn metal() -> Self {
        Self::Wgpu {
            label: "WGPU_BACKEND=metal",
            backend: Some("metal"),
            enforce_software: false,
        }
    }

    fn glow_software() -> Self {
        Self::GlowSoftware {
            label: "software glow",
 codex/fix-gui-blank-screen-issue
        }
    }

    fn label(&self) -> &'static str {
        match self {
            Self::Wgpu { label, .. } | Self::GlowSoftware { label } => label,
        }
    }

    fn apply(&self, launch: &LaunchConfig, diagnostics: &mut RendererDiagnostics) -> AttemptConfig {

        }
    }

    fn label(&self) -> &'static str {
        match self {
            Self::Wgpu { label, .. } | Self::GlowSoftware { label } => label,
        }
    }

    fn apply(&self, launch: &LaunchConfig, diagnostics: &mut RendererDiagnostics) {
 main
        match self {
            Self::Wgpu {
                backend,
                enforce_software,
                ..
            } => {
                match (backend, &launch.original_backend) {
                    (Some(value), _) => unsafe { env::set_var("WGPU_BACKEND", value) },
                    (None, Some(original)) => unsafe { env::set_var("WGPU_BACKEND", original) },
                    (None, None) => unsafe { env::remove_var("WGPU_BACKEND") },
                }

                let software = launch.force_software || *enforce_software;
                diagnostics.software_backend = software;
                if software {
                    unsafe {
                        env::set_var("LIBGL_ALWAYS_SOFTWARE", "1");
                        env::set_var("WGPU_POWER_PREF", "low_power");
                    }
                } else {
                    if let Some(original) = &launch.original_libgl {
                        unsafe { env::set_var("LIBGL_ALWAYS_SOFTWARE", original) };
                    } else {
                        unsafe { env::remove_var("LIBGL_ALWAYS_SOFTWARE") };
                    }

                    if let Some(original) = &launch.original_power_pref {
                        unsafe { env::set_var("WGPU_POWER_PREF", original) };
                    } else {
                        unsafe { env::remove_var("WGPU_POWER_PREF") };
                    }
                }
 codex/fix-gui-blank-screen-issue

                AttemptConfig {
                    backends: backend_mask(*backend),
                    force_fallback: software,
                }

 main
            }
            Self::GlowSoftware { .. } => {
                diagnostics.software_backend = true;
                if let Some(original) = &launch.original_backend {
                    unsafe { env::set_var("WGPU_BACKEND", original) };
                } else {
                    unsafe { env::remove_var("WGPU_BACKEND") };
                }
                unsafe {
                    env::set_var("LIBGL_ALWAYS_SOFTWARE", "1");
                    env::remove_var("WGPU_POWER_PREF");
                }
 codex/fix-gui-blank-screen-issue
                AttemptConfig::glow()

 main
            }
        }
    }

    fn prefers_low_power(&self, launch: &LaunchConfig) -> bool {
        match self {
            Self::Wgpu {
                enforce_software, ..
            } => launch.force_software || *enforce_software,
            Self::GlowSoftware { .. } => true,
        }
    }

    fn is_wgpu(&self) -> bool {
        matches!(self, Self::Wgpu { .. })
 codex/fix-gui-blank-screen-issue
    }
}

#[derive(Clone, Copy)]
struct AttemptConfig {
    backends: wgpu::Backends,
    force_fallback: bool,
}

impl AttemptConfig {
    fn glow() -> Self {
        Self {
            backends: wgpu::Backends::empty(),
            force_fallback: false,
        }
    }
}

fn backend_mask(backend: Option<&'static str>) -> wgpu::Backends {
    match backend {
        Some("vulkan") => wgpu::Backends::VULKAN,
        Some("metal") => wgpu::Backends::METAL,
        Some("dx12") => wgpu::Backends::DX12,
        Some("gl") => wgpu::Backends::GL,
        _ => wgpu::Backends::all(),
 main
    }
}

pub struct HeadlessReport {
    pub diagnostics: RendererDiagnostics,
}

impl fmt::Display for HeadlessReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Renderer: {}", self.diagnostics.backend)?;
        if let Some(adapter) = &self.diagnostics.adapter_name {
            writeln!(f, "Adapter: {}", adapter)?;
        }
        if let Some(kind) = &self.diagnostics.adapter_type {
            writeln!(f, "Adapter Type: {}", kind)?;
        }
        if let Some(comp) = &self.diagnostics.compositor {
            writeln!(f, "Compositor: {}", comp)?;
        }
        if self.diagnostics.software_backend {
            writeln!(f, "Software fallback active")?;
        }
        if self.diagnostics.fallback_used {
            writeln!(f, "Fallback engaged")?;
        }
        if let Some(reason) = &self.diagnostics.failure_reason {
            writeln!(f, "Previous error: {}", reason)?;
        }
        Ok(())
    }
}

pub fn detect(launch: &LaunchConfig, start_attempt: usize) -> Result<RendererSelection, String> {
    let mut diagnostics = RendererDiagnostics::new();
    diagnostics.env_forced = launch.env_forced;
    diagnostics.compositor = compositor_name();
    diagnostics.forced_software = launch.force_software || launch.env_forced;

    let plan = launch.fallback_plan();
    let mut failures: Vec<String> = Vec::new();

    for (index, attempt) in plan.iter().enumerate().skip(start_attempt) {
        let mut current = diagnostics.clone();
        current.started_at = Instant::now();
        current.fallback_used = start_attempt > 0 || !failures.is_empty();

 codex/fix-gui-blank-screen-issue
        let attempt_config = attempt.apply(launch, &mut current);

        if attempt.is_wgpu() {
            let prefer_low_power = attempt.prefers_low_power(launch);
            match probe_wgpu(
                attempt_config.backends,
                prefer_low_power,
                attempt_config.force_fallback,
            ) {

        attempt.apply(launch, &mut current);

        if attempt.is_wgpu() {
            let prefer_low_power = attempt.prefers_low_power(launch);
            match probe_wgpu(prefer_low_power) {
 main
                Ok(info) => {
                    current.backend = format!("{:?}", info.backend);
                    current.adapter_name = Some(info.name.clone());
                    current.adapter_type = Some(format!("{:?}", info.device_type));
                    current.backend_details = Some(format!("Driver: {}", info.driver));
                    if info.device_type == wgpu::DeviceType::Cpu {
                        current.software_backend = true;
                    }
                    if !failures.is_empty() {
                        current.failure_reason = Some(failures.join(" -> "));
                    }
                    return Ok(RendererSelection::new_wgpu(index, attempt.label(), current));
                }
                Err(err) => {
                    failures.push(format!("{}: {}", attempt.label(), err));
                }
            }
        } else {
            if !failures.is_empty() {
                current.failure_reason = Some(failures.join(" -> "));
            }
            return Ok(RendererSelection::new_glow(index, attempt.label(), current));
        }
    }

    if failures.is_empty() {
        Err("No compatible wgpu adapters found".to_string())
    } else {
        Err(failures.join(" -> "))
    }
}

pub fn run_headless_probe(launch: &LaunchConfig) -> HeadlessReport {
    match detect(launch, 0) {
        Ok(selection) => HeadlessReport {
            diagnostics: selection.diagnostics,
        },
        Err(err) => {
            let mut diagnostics = RendererDiagnostics::new();
            diagnostics.failure_reason = Some(err);
            diagnostics.compositor = compositor_name();
            HeadlessReport { diagnostics }
        }
    }
}

fn probe_wgpu(
    backends: wgpu::Backends,
    low_power: bool,
    force_fallback_adapter: bool,
) -> Result<AdapterInfo, String> {
    let requested_backends = if backends.is_empty() {
        wgpu::Backends::all()
    } else {
        backends
    };

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: requested_backends,
        dx12_shader_compiler: Default::default(),
        ..Default::default()
    });

    let power_preference = if low_power {
        wgpu::PowerPreference::LowPower
    } else {
        wgpu::PowerPreference::HighPerformance
    };

    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference,
        force_fallback_adapter,
        compatible_surface: None,
    }))
    .ok_or_else(|| "No compatible GPU adapters".to_string())?;

    Ok(adapter.get_info())
}

fn compositor_name() -> Option<String> {
    if env::var("WAYLAND_DISPLAY").is_ok() {
        Some("Wayland".to_string())
    } else if env::var("DISPLAY").is_ok() {
        Some("X11".to_string())
    } else {
        None
    }
}
