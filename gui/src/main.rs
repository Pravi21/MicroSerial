mod app;
mod console;
mod core;
mod device_scan;
mod diagnostics;
mod profiles;
mod renderer;
mod send_panel;
mod session;
mod settings;
mod theme;

use app::MicroSerialApp;
use egui_wgpu::WgpuError;
use renderer::{LaunchConfig, RendererSelection};
use settings::Settings;

fn main() -> eframe::Result<()> {
    env_logger::init();
    let mut launch = LaunchConfig::from_args();
    let settings = Settings::load().unwrap_or_default();
    if settings.force_software {
        launch.enable_force_software();
    }

    if launch.headless {
        let report = renderer::run_headless_probe(&launch);
        println!("{}", report);
        return Ok(());
    }

    let total_attempts = launch.attempt_count();
    let mut attempt_index = 0;
    let mut previous_error: Option<String> = None;

    while attempt_index < total_attempts {
        let mut selection = match renderer::detect(&launch, attempt_index) {
            Ok(selection) => selection,
            Err(err) => {
                if let Some(previous) = previous_error.take() {
                    eprintln!("Renderer failed after fallback: {previous}");
                }
                eprintln!("Renderer detection failed: {err}");
                return Err(eframe::Error::Wgpu(WgpuError::NoSuitableAdapterFound));
            }
        };

        if attempt_index > 0 || selection.attempt > 0 {
            selection.diagnostics.fallback_used = true;
        }

        if let Some(previous) = previous_error.take() {
            selection.diagnostics.fallback_used = true;
            selection.diagnostics.failure_reason = match selection.diagnostics.failure_reason.take()
            {
                Some(existing) => Some(format!("{existing} -> {previous}")),
                None => Some(previous),
            };
        }

        match run_with_selection(&selection, &settings) {
            Ok(value) => return Ok(value),
            Err(err) => {
                eprintln!(
                    "Renderer attempt '{}' failed: {err}",
                    selection.attempt_label
                );
                previous_error = Some(err.to_string());
                attempt_index = selection.attempt + 1;
                if attempt_index >= total_attempts {
                    return Err(err);
                }
            }
        }
    }

    Err(eframe::Error::Wgpu(WgpuError::NoSuitableAdapterFound))
}

fn run_with_selection(selection: &RendererSelection, settings: &Settings) -> eframe::Result<()> {
    let diagnostics = selection.diagnostics.clone();
    let app_settings = settings.clone();
    let mut options = selection.options.clone();
    options.follow_system_theme = false;
    eframe::run_native(
        "MicroSerial",
        options,
        Box::new(move |_cc| {
            Box::new(MicroSerialApp::new(
                diagnostics.clone(),
                app_settings.clone(),
            ))
        }),
    )
}
