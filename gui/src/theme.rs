use eframe::egui::{self, FontFamily, FontId, Style};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, EnumIter, Display)]
#[strum(serialize_all = "title_case")]
pub enum ThemePreference {
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ThemeState {
    pub preference: ThemePreference,
    pub font_size: f32,
}

impl Default for ThemeState {
    fn default() -> Self {
        Self {
            preference: ThemePreference::System,
            font_size: 16.0,
        }
    }
}

impl ThemeState {
    pub fn apply(&self, ctx: &egui::Context) {
        let mut style: Style = (*ctx.style()).clone();
        style.text_styles.insert(
            egui::TextStyle::Heading,
            FontId::new(self.font_size + 4.0, FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Body,
            FontId::new(self.font_size, FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Monospace,
            FontId::new(self.font_size, FontFamily::Monospace),
        );
        ctx.set_style(style);

        match self.preference {
            ThemePreference::System => {
                if ctx.style().visuals.dark_mode {
                    ctx.set_visuals(egui::Visuals::dark());
                } else {
                    ctx.set_visuals(egui::Visuals::light());
                }
            }
            ThemePreference::Light => ctx.set_visuals(light_visuals()),
            ThemePreference::Dark => ctx.set_visuals(dark_visuals()),
        }
    }
}

fn light_visuals() -> egui::Visuals {
    let mut visuals = egui::Visuals::light();
    visuals.override_text_color = Some(egui::Color32::from_rgb(30, 35, 40));
    visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(245, 248, 250);
    visuals
}

fn dark_visuals() -> egui::Visuals {
    let mut visuals = egui::Visuals::dark();
    visuals.override_text_color = Some(egui::Color32::from_rgb(230, 235, 240));
    visuals
}
