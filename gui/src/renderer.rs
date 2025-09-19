use std::env;
use std::fmt;
use std::time::Instant;

use eframe;
use serde::{Deserialize, Serialize};
use wgpu::AdapterInfo;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RendererKind {
    Wgpu,
    Glow,
}

impl RendererKind {}

#[derive(Debug, Clone)]
pub struct RendererDiagnostics {
    pub forced_software: bool,
    pub env_forced: bool,
    pub fallback_used: bool,
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
            compositor: None,
            backend: "wgpu".to_string(),
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
}

impl LaunchConfig {
    pub fn from_args() -> Self {
        let env_forced = env_flag("MICROSERIAL_FORCE_SOFTWARE");
        let mut force_software = env_forced;
        let mut headless = false;
        for arg in std::env::args().skip(1) {
            match arg.as_str() {
                "--force-software" => force_software = true,
                "--headless-detect" => headless = true,
                _ => {}
            }
        }
        Self {
            force_software,
            headless,
            env_forced,
        }
    }
}

fn env_flag(key: &str) -> bool {
    matches!(env::var(key), Ok(v) if v == "1" || v.eq_ignore_ascii_case("true"))
}

#[derive(Clone)]
pub struct RendererDecision {
    pub kind: RendererKind,
    pub options: eframe::NativeOptions,
    pub diagnostics: RendererDiagnostics,
    pub fallback_available: bool,
}

impl RendererDecision {
    pub fn new(kind: RendererKind, diagnostics: RendererDiagnostics) -> Self {
        let mut options = eframe::NativeOptions::default();
        match kind {
            RendererKind::Wgpu => {
                options.renderer = eframe::Renderer::Wgpu;
                options.hardware_acceleration = eframe::HardwareAcceleration::Preferred;
            }
            RendererKind::Glow => {
                options.renderer = eframe::Renderer::Glow;
                options.hardware_acceleration = eframe::HardwareAcceleration::Off;
            }
        }
        Self {
            kind,
            options,
            diagnostics,
            fallback_available: true,
        }
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
        if self.diagnostics.fallback_used {
            writeln!(f, "Fallback engaged")?;
        }
        Ok(())
    }
}

pub fn detect(launch: &LaunchConfig) -> RendererDecision {
    let mut diagnostics = RendererDiagnostics::new();
    diagnostics.env_forced = launch.env_forced;
    diagnostics.compositor = compositor_name();

    if launch.force_software {
        diagnostics.forced_software = true;
        diagnostics.backend = "glow".to_string();
        unsafe {
            std::env::set_var("LIBGL_ALWAYS_SOFTWARE", "1");
        }
        return RendererDecision::new(RendererKind::Glow, diagnostics);
    }

    match probe_wgpu() {
        Ok(info) => {
            diagnostics.backend = "wgpu".to_string();
            diagnostics.adapter_name = Some(info.name.clone());
            diagnostics.adapter_type = Some(format!("{:?}", info.device_type));
            diagnostics.backend_details = Some(format!(
                "Backend: {:?}, Driver: {}",
                info.backend, info.driver
            ));
            RendererDecision::new(RendererKind::Wgpu, diagnostics)
        }
        Err(err) => {
            diagnostics.backend = "glow".to_string();
            diagnostics.fallback_used = true;
            diagnostics.failure_reason = Some(err.clone());
            unsafe {
                std::env::set_var("LIBGL_ALWAYS_SOFTWARE", "1");
            }
            let mut decision = RendererDecision::new(RendererKind::Glow, diagnostics);
            decision.fallback_available = false;
            decision
        }
    }
}

pub fn force_glow(reason: String, mut diagnostics: RendererDiagnostics) -> RendererDecision {
    diagnostics.backend = "glow".to_string();
    diagnostics.fallback_used = true;
    diagnostics.failure_reason = Some(reason);
    unsafe {
        std::env::set_var("LIBGL_ALWAYS_SOFTWARE", "1");
    }
    RendererDecision::new(RendererKind::Glow, diagnostics)
}

pub fn run_headless_probe(launch: &LaunchConfig) -> HeadlessReport {
    let decision = detect(launch);
    HeadlessReport {
        diagnostics: decision.diagnostics,
    }
}

fn probe_wgpu() -> Result<AdapterInfo, String> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        dx12_shader_compiler: Default::default(),
        ..Default::default()
    });

    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::LowPower,
        force_fallback_adapter: false,
        compatible_surface: None,
    }))
    .ok_or_else(|| "No compatible GPU adapters".to_string())?;

    let info = adapter.get_info();
    if info.device_type == wgpu::DeviceType::Cpu {
        return Err("Only CPU adapter available".into());
    }

    Ok(info)
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
