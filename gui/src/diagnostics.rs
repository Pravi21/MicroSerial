use eframe::egui::{self, RichText};

use crate::renderer::RendererDiagnostics;

#[derive(Default)]
pub struct DiagnosticsState {
    pub open: bool,
    pub renderer: RendererDiagnostics,
    pub last_error: Option<String>,
}

impl DiagnosticsState {
    pub fn show(&mut self, ctx: &egui::Context) {
        if !self.open {
            return;
        }
        egui::Window::new("Diagnostics")
            .open(&mut self.open)
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Renderer");
                ui.separator();
                ui.label(format!("Backend: {}", self.renderer.backend));
                if let Some(adapter) = &self.renderer.adapter_name {
                    ui.label(format!("Adapter: {adapter}"));
                }
                if let Some(kind) = &self.renderer.adapter_type {
                    ui.label(format!("Adapter Type: {kind}"));
                }
                if let Some(comp) = &self.renderer.compositor {
                    ui.label(format!("Compositor: {comp}"));
                }
                if let Some(detail) = &self.renderer.backend_details {
                    ui.label(RichText::new(detail).italics());
                }
                if self.renderer.fallback_used {
                    ui.label("Fallback backend engaged");
                }
                if self.renderer.software_backend {
                    ui.label("Software fallback active");
                }
                ui.label(format!(
                    "Renderer probed {:.1?} ago",
                    self.renderer.started_at.elapsed()
                ));
                if let Some(reason) = &self.renderer.failure_reason {
                    ui.colored_label(
                        egui::Color32::from_rgb(200, 80, 80),
                        format!("Fallback: {reason}"),
                    );
                }
                if self.renderer.forced_software {
                    ui.label("Software rendering forced");
                }
                if self.renderer.env_forced {
                    ui.label("Set by MICROSERIAL_FORCE_SOFTWARE");
                }
                if let Some(error) = &self.last_error {
                    ui.separator();
                    ui.colored_label(
                        egui::Color32::from_rgb(255, 120, 40),
                        format!("Last error: {error}"),
                    );
                }
            });
    }
}
