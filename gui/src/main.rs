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
use renderer::{LaunchConfig, RendererDecision, RendererKind};
use settings::Settings;

fn main() -> eframe::Result<()> {
    env_logger::init();
    let mut launch = LaunchConfig::from_args();
    let settings = Settings::load().unwrap_or_default();
    if settings.force_software {
        launch.force_software = true;
    }

    if launch.headless {
        let report = renderer::run_headless_probe(&launch);
        println!("{}", report);
        return Ok(());
    }

    let decision = renderer::detect(&launch);
    match run_with_decision(&decision, &settings) {
        Ok(value) => Ok(value),
        Err(err) => {
            if decision.kind == RendererKind::Wgpu {
                eprintln!("Hardware renderer failed: {err}. Falling back to software...");
                let fallback = renderer::force_glow(err.to_string(), decision.diagnostics.clone());
                run_with_decision(&fallback, &settings)
            } else {
                Err(err)
            }
        }
    }
}

fn run_with_decision(decision: &RendererDecision, settings: &Settings) -> eframe::Result<()> {
    let diagnostics = decision.diagnostics.clone();
    let app_settings = settings.clone();
    let mut options = decision.options.clone();
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
