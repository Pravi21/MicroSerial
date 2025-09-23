use std::time::{Duration, Instant};

use eframe::egui::{self, Align, Color32, ComboBox, Frame, Layout, Margin, RichText, Rounding};
use strum::IntoEnumIterator;
use time::format_description::well_known::Rfc3339;

use crate::console::{ConsoleBuffer, ConsoleEntry, ConsoleViewMode};
use crate::core::{FlowControl, Parity, SerialConfig, SerialDevice, StopBits};
use crate::device_scan::DeviceScanner;
use crate::diagnostics::DiagnosticsState;
use crate::renderer::RendererDiagnostics;
use crate::send_panel::{PayloadError, SendMode, SendPanelState};
use crate::session::{SerialSession, SessionError, SessionMessage};
use crate::settings::{self, Settings};
use crate::theme::ThemeState;

const REFRESH_INTERVAL: Duration = Duration::from_secs(4);
const BAUD_PRESETS: &[u32] = &[
    9_600, 19_200, 38_400, 57_600, 115_200, 230_400, 460_800, 921_600,
];
const HELP_URL: &str = "https://github.com/microserial/docs/blob/main/docs/gui/first_run.md";

pub struct MicroSerialApp {
    renderer: RendererDiagnostics,
    scanner: DeviceScanner,
    ports: Vec<SerialDevice>,
    selected_port: Option<String>,
    config: SerialConfig,
    session: Option<SerialSession>,
    console: ConsoleBuffer,
    send_panel: SendPanelState,
    diagnostics: DiagnosticsState,
    settings: Settings,
    theme_state: ThemeState,
    status: Option<StatusBanner>,
    settings_dirty: bool,
    last_save: Instant,
    custom_baud: String,
}

struct StatusBanner {
    message: String,
    tone: StatusTone,
    created: Instant,
}

enum StatusTone {
    Info,
    Success,
    Warn,
    Error,
}

impl StatusTone {
    fn color(&self) -> Color32 {
        match self {
            StatusTone::Info => Color32::from_rgb(70, 120, 200),
            StatusTone::Success => Color32::from_rgb(60, 150, 90),
            StatusTone::Warn => Color32::from_rgb(210, 160, 60),
            StatusTone::Error => Color32::from_rgb(200, 70, 70),
        }
    }
}

impl MicroSerialApp {
    pub fn new(renderer: RendererDiagnostics, mut settings: Settings) -> Self {
        settings.profiles.ensure_default();
        let mut scanner = DeviceScanner::new();
        scanner.refresh();
        let theme_state = settings.theme;
        let config = settings
            .profiles
            .get_active()
            .map(|profile| profile.config.clone())
            .unwrap_or_else(SerialConfig::default);
        let custom_baud = config.baud_rate.to_string();
        let mut console = ConsoleBuffer::default();
        console.show_timestamps = settings.show_timestamps;
        console.view_mode = settings.console_view;

        let mut diagnostics = DiagnosticsState::default();
        diagnostics.renderer = renderer.clone();
        if settings.force_software {
            diagnostics.renderer.forced_software = true;
            diagnostics.renderer.software_backend = true;
        }

        Self {
            renderer,
            scanner,
            ports: Vec::new(),
            selected_port: None,
            config,
            session: None,
            console,
            send_panel: SendPanelState::new(),
            diagnostics,
            settings,
            theme_state,
            status: None,
            settings_dirty: false,
            last_save: Instant::now(),
            custom_baud,
        }
    }

    fn mark_dirty(&mut self) {
        self.settings_dirty = true;
    }

    fn poll_scanner(&mut self) {
        if self.scanner.auto_refresh_due(REFRESH_INTERVAL) {
            self.scanner.refresh();
        }
        if let Some(result) = self.scanner.poll().cloned() {
            self.ports = result.devices.clone();
            if let Some(error) = result.error {
                self.set_status(&error, StatusTone::Warn);
            }
        }
    }

    fn poll_session(&mut self) {
        if let Some(session) = &mut self.session {
            for message in session.poll() {
                match message {
                    SessionMessage::Data(bytes) => {
                        self.console.push_rx(&bytes);
                    }
                    SessionMessage::Event(event) => {
                        self.console
                            .push_event(&format!("{}: {}", event.code, event.message));
                        self.set_status(&event.message, StatusTone::Info);
                    }
                }
            }
        }
    }

    fn connect(&mut self) {
        let Some(path) = self.selected_port.clone() else {
            self.set_status("Select a port to connect", StatusTone::Warn);
            return;
        };
        match SerialSession::open(&path, &self.config) {
            Ok(session) => {
                self.session = Some(session);
                self.set_status(&format!("Connected to {path}"), StatusTone::Success);
            }
            Err(err) => {
                self.set_status(&format!("Connect failed: {err}"), StatusTone::Error);
            }
        }
    }

    fn disconnect(&mut self) {
        if self.session.is_some() {
            self.session = None;
            self.set_status("Disconnected", StatusTone::Info);
        }
    }

    fn send_current_payload(&mut self) {
        let payload = match self.send_panel.parse_payload(&self.send_panel.input) {
            Ok(bytes) => bytes,
            Err(PayloadError::InvalidHex) => {
                self.set_status("Invalid hex payload", StatusTone::Error);
                return;
            }
        };

        match self.session.as_mut() {
            Some(session) => match session.write(&payload) {
                Ok(()) => {
                    self.console.push_tx(&payload);
                    self.set_status("Payload sent", StatusTone::Success);
                    let value = self.send_panel.input.clone();
                    self.send_panel.push_history(value);
                }
                Err(SessionError::Truncated) => {
                    self.set_status("Write truncated", StatusTone::Warn);
                }
                Err(err) => {
                    self.set_status(&format!("Write error: {err}"), StatusTone::Error);
                }
            },
            None => self.set_status("Not connected", StatusTone::Warn),
        }
    }

    fn set_status(&mut self, message: &str, tone: StatusTone) {
        self.status = Some(StatusBanner {
            message: message.to_string(),
            tone,
            created: Instant::now(),
        });
    }

    fn status_pill(&self, ui: &mut egui::Ui) {
        let (label, tone) = if self.session.is_some() {
            ("Connected", StatusTone::Success)
        } else {
            ("Disconnected", StatusTone::Warn)
        };
        Frame::none()
            .fill(tone.color())
            .rounding(Rounding::same(10.0))
            .inner_margin(Margin::symmetric(12.0, 6.0))
            .show(ui, |ui| {
                ui.label(RichText::new(label).color(Color32::WHITE).strong());
            });
    }

    fn show_status_banner(&mut self, ui: &mut egui::Ui) {
        if let Some(banner) = &self.status {
            if banner.created.elapsed() < Duration::from_secs(6) {
                Frame::none()
                    .fill(banner.tone.color())
                    .rounding(Rounding::same(10.0))
                    .inner_margin(Margin::symmetric(12.0, 6.0))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(&banner.message)
                                .color(Color32::WHITE)
                                .strong(),
                        );
                    });
                return;
            }
        }
        self.status = None;
    }

    fn save_settings_if_needed(&mut self) {
        if self.settings_dirty && self.last_save.elapsed() > Duration::from_secs(1) {
            if let Err(err) = self.settings.save() {
                self.set_status(&format!("Settings save failed: {err}"), StatusTone::Error);
            }
            self.last_save = Instant::now();
            self.settings_dirty = false;
        }
    }

    fn devices_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Devices");
        ui.horizontal(|ui| {
            if ui
                .add(egui::Button::new(
                    RichText::new("Refresh").text_style(egui::TextStyle::Button),
                ))
                .clicked()
            {
                self.scanner.refresh();
            }
            ui.hyperlink_to("Help", HELP_URL);
            if let Some(result) = self.scanner.last_result() {
                ui.label(format!(
                    "Last scan {:.1?} ago ({:.1?})",
                    result.completed_at.elapsed(),
                    result.duration
                ));
            }
        });

        if self.ports.is_empty() {
            ui.spacing_mut().item_spacing.y = 8.0;
            ui.label("No serial devices detected.");
        }

        for port in &self.ports {
            let selected = self.selected_port.as_deref() == Some(&port.path);
            let label = if port.description.is_empty() {
                port.path.clone()
            } else {
                format!("{}\n{}", port.description, port.path)
            };
            let response = ui.selectable_label(selected, label);
            if response.clicked() {
                self.selected_port = Some(port.path.clone());
            }
        }
    }

    fn empty_state(&mut self, ui: &mut egui::Ui) {
        if !self.ports.is_empty() {
            return;
        }
        ui.vertical_centered(|ui| {
            ui.add_space(32.0);
            egui::Frame::group(ui.style())
                .fill(Color32::from_rgb(25, 90, 140).gamma_multiply(0.15))
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("No serial devices detected");
                        ui.label("Connect a device or check permissions.");
                        if ui.button("Refresh").clicked() {
                            self.scanner.refresh();
                        }
                        ui.hyperlink_to("Help with permissions", HELP_URL);
                    });
                });
            ui.add_space(24.0);
        });
    }

    fn configuration_panel(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("Connection", |ui| {
            ui.horizontal(|ui| {
                ui.label("Port");
                ComboBox::from_id_source("port_combo")
                    .selected_text(
                        self.selected_port
                            .as_deref()
                            .map(|p| p.to_string())
                            .unwrap_or_else(|| "Select".to_string()),
                    )
                    .show_ui(ui, |ui| {
                        for port in &self.ports {
                            ui.selectable_value(
                                &mut self.selected_port,
                                Some(port.path.clone()),
                                format!("{} ({})", port.description, port.path),
                            );
                        }
                    });
            });

            ui.separator();
            self.baud_row(ui);
            self.data_bits_row(ui);
            self.parity_row(ui);
            self.stop_bits_row(ui);
            self.flow_control_row(ui);

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Connect").clicked() {
                    self.connect();
                }
                if ui.button("Disconnect").clicked() {
                    self.disconnect();
                }
                self.status_pill(ui);
            });
        });

        ui.separator();
        ui.collapsing("Profiles", |ui| {
            self.profiles_ui(ui);
        });
    }

    fn baud_row(&mut self, ui: &mut egui::Ui) {
        let mut selected = self.config.baud_rate;
        ui.horizontal(|ui| {
            ui.label("Baud rate");
            ComboBox::from_id_source("baud_combo")
                .selected_text(format!("{} bps", selected))
                .show_ui(ui, |ui| {
                    for preset in BAUD_PRESETS {
                        ui.selectable_value(&mut selected, *preset, format!("{preset} bps"));
                    }
                });
            if selected != self.config.baud_rate {
                self.config.baud_rate = selected;
                self.custom_baud = selected.to_string();
                self.mark_dirty();
            }
        });
        ui.horizontal(|ui| {
            ui.label("Custom");
            let response = ui.text_edit_singleline(&mut self.custom_baud);
            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                if let Ok(value) = self.custom_baud.replace('_', "").parse::<u32>() {
                    self.config.baud_rate = value;
                    self.mark_dirty();
                } else {
                    self.set_status("Invalid baud rate", StatusTone::Error);
                }
            }
        });
    }

    fn data_bits_row(&mut self, ui: &mut egui::Ui) {
        let bits_options = [5_u8, 6, 7, 8];
        let mut selected = self.config.data_bits;
        ui.horizontal(|ui| {
            ui.label("Data bits");
            ComboBox::from_id_source("data_bits_combo")
                .selected_text(selected.to_string())
                .show_ui(ui, |ui| {
                    for &bits in &bits_options {
                        ui.selectable_value(&mut selected, bits, bits.to_string());
                    }
                });
        });
        if selected != self.config.data_bits {
            self.config.data_bits = selected;
            self.mark_dirty();
        }
    }

    fn parity_row(&mut self, ui: &mut egui::Ui) {
        let mut selected = self.config.parity;
        ui.horizontal(|ui| {
            ui.label("Parity");
            ComboBox::from_id_source("parity_combo")
                .selected_text(selected.to_string())
                .show_ui(ui, |ui| {
                    for parity in Parity::iter() {
                        ui.selectable_value(&mut selected, parity, parity.to_string());
                    }
                });
        });
        if selected != self.config.parity {
            self.config.parity = selected;
            self.mark_dirty();
        }
    }

    fn stop_bits_row(&mut self, ui: &mut egui::Ui) {
        let mut selected = self.config.stop_bits;
        ui.horizontal(|ui| {
            ui.label("Stop bits");
            ComboBox::from_id_source("stop_bits_combo")
                .selected_text(selected.to_string())
                .show_ui(ui, |ui| {
                    for stop in StopBits::iter() {
                        ui.selectable_value(&mut selected, stop, stop.to_string());
                    }
                });
        });
        if selected != self.config.stop_bits {
            self.config.stop_bits = selected;
            self.mark_dirty();
        }
    }

    fn flow_control_row(&mut self, ui: &mut egui::Ui) {
        let mut selected = self.config.flow_control;
        ui.horizontal(|ui| {
            ui.label("Flow control");
            ComboBox::from_id_source("flow_control_combo")
                .selected_text(selected.to_string())
                .show_ui(ui, |ui| {
                    for flow in FlowControl::iter() {
                        ui.selectable_value(&mut selected, flow, flow.to_string());
                    }
                });
        });
        if selected != self.config.flow_control {
            self.config.flow_control = selected;
            self.mark_dirty();
        }
    }

    fn profiles_ui(&mut self, ui: &mut egui::Ui) {
        let mut active_name = self
            .settings
            .profiles
            .active
            .clone()
            .unwrap_or_else(|| "Default".to_string());
        ComboBox::from_id_source("profiles_combo")
            .selected_text(active_name.clone())
            .show_ui(ui, |ui| {
                for profile in &self.settings.profiles.profiles {
                    ui.selectable_value(
                        &mut active_name,
                        profile.name.clone(),
                        profile.name.clone(),
                    );
                }
            });
        if Some(active_name.clone()) != self.settings.profiles.active {
            self.settings.profiles.set_active(&active_name);
            if let Some(profile) = self.settings.profiles.get_active() {
                self.config = profile.config.clone();
                self.custom_baud = self.config.baud_rate.to_string();
            }
            self.mark_dirty();
        }

        ui.horizontal(|ui| {
            if ui.button("Save profile").clicked() {
                let profile_name = self
                    .settings
                    .profiles
                    .active
                    .clone()
                    .unwrap_or_else(|| "Default".into());
                self.settings
                    .profiles
                    .upsert(crate::profiles::SerialProfile::new(
                        profile_name,
                        self.config.clone(),
                    ));
                self.set_status("Profile saved", StatusTone::Success);
                self.mark_dirty();
            }
            if ui.button("New profile").clicked() {
                let name = format!("Profile {}", self.settings.profiles.profiles.len() + 1);
                self.settings
                    .profiles
                    .upsert(crate::profiles::SerialProfile::new(
                        name.clone(),
                        self.config.clone(),
                    ));
                self.settings.profiles.set_active(&name);
                self.set_status("Profile created", StatusTone::Success);
                self.mark_dirty();
            }
            if ui.button("Delete").clicked() {
                if let Some(active) = self.settings.profiles.active.clone() {
                    self.settings.profiles.delete(&active);
                    self.set_status("Profile removed", StatusTone::Info);
                    self.mark_dirty();
                }
            }
        });
    }

    fn console_panel(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("Console");
            if ui.button("Clear").clicked() {
                self.console.clear();
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.checkbox(&mut self.console.show_timestamps, "Timestamps");
                if self.console.show_timestamps != self.settings.show_timestamps {
                    self.settings.show_timestamps = self.console.show_timestamps;
                    self.mark_dirty();
                }
                ComboBox::from_id_source("view_mode_combo")
                    .selected_text(format!("View: {:?}", self.console.view_mode))
                    .show_ui(ui, |ui| {
                        for mode in [
                            ConsoleViewMode::Text,
                            ConsoleViewMode::Hex,
                            ConsoleViewMode::Mixed,
                        ] {
                            ui.selectable_value(
                                &mut self.console.view_mode,
                                mode,
                                format!("{mode:?}"),
                            );
                        }
                    });
                if self.console.view_mode != self.settings.console_view {
                    self.settings.console_view = self.console.view_mode;
                    self.mark_dirty();
                }
            });
        });
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label("Filter");
            ui.text_edit_singleline(&mut self.console.filter);
        });
        ui.separator();

        egui::ScrollArea::vertical()
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for entry in self.console.iter() {
                    self.console_row(ui, entry);
                }
            });
    }

    fn console_row(&self, ui: &mut egui::Ui, entry: &ConsoleEntry) {
        Frame::group(ui.style())
            .fill(Color32::from_rgba_premultiplied(32, 64, 96, 20))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.colored_label(Color32::from_rgb(90, 140, 210), entry.direction.label());
                    if self.console.show_timestamps {
                        if let Ok(ts) = entry.timestamp.format(&Rfc3339) {
                            ui.label(ts);
                        }
                    }
                    match self.console.view_mode {
                        ConsoleViewMode::Text => {
                            ui.label(&entry.text);
                        }
                        ConsoleViewMode::Hex => {
                            ui.label(&entry.hex);
                        }
                        ConsoleViewMode::Mixed => {
                            ui.vertical(|ui| {
                                ui.label(&entry.text);
                                ui.add_space(2.0);
                                ui.label(RichText::new(&entry.hex).monospace().weak());
                            });
                        }
                    }
                });
            });
    }

    fn send_panel(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading("Send");
            ui.horizontal(|ui| {
                ComboBox::from_id_source("send_mode")
                    .selected_text(self.send_panel.mode.label())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.send_panel.mode,
                            SendMode::Text,
                            SendMode::Text.label(),
                        );
                        ui.selectable_value(
                            &mut self.send_panel.mode,
                            SendMode::Hex,
                            SendMode::Hex.label(),
                        );
                    });
                let response = ui.text_edit_singleline(&mut self.send_panel.input);
                let send_clicked = ui.button("Send").clicked();
                let enter_pressed =
                    response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                if send_clicked || enter_pressed {
                    self.send_current_payload();
                }
            });
            ui.separator();
            ui.label("History");
            egui::ScrollArea::vertical()
                .max_height(120.0)
                .show(ui, |ui| {
                    let entries: Vec<_> = self.send_panel.history.iter().cloned().collect();
                    for (index, entry) in entries.into_iter().enumerate() {
                        ui.horizontal(|ui| {
                            if ui
                                .selectable_label(false, format!("{}", entry.value))
                                .clicked()
                            {
                                self.send_panel.input = entry.value.clone();
                                self.send_panel.mode = entry.mode;
                            }
                            if ui
                                .small_button(if entry.favorited { "★" } else { "☆" })
                                .clicked()
                            {
                                self.send_panel.toggle_favorite(index);
                            }
                        });
                    }
                });
            let favorites: Vec<_> = self.send_panel.favorites().cloned().collect();
            if !favorites.is_empty() {
                ui.separator();
                ui.label("Favorites");
                ui.horizontal_wrapped(|ui| {
                    for fav in favorites {
                        if ui.button(format!("★ {}", fav.value)).clicked() {
                            self.send_panel.input = fav.value.clone();
                            self.send_panel.mode = fav.mode;
                        }
                    }
                });
            }
        });
    }

    fn top_bar(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading(RichText::new("MicroSerial").size(self.theme_state.font_size + 4.0));
            self.show_status_banner(ui);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.button("Diagnostics").clicked() {
                    self.diagnostics.open = true;
                    self.diagnostics.renderer = self.renderer.clone();
                }
                ui.menu_button("Appearance", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Theme");
                        ComboBox::from_id_source("theme_pref")
                            .selected_text(self.theme_state.preference.to_string())
                            .show_ui(ui, |ui| {
                                for pref in settings::theme_options() {
                                    ui.selectable_value(
                                        &mut self.theme_state.preference,
                                        pref,
                                        pref.to_string(),
                                    );
                                }
                            });
                    });
                    ui.add(
                        egui::Slider::new(&mut self.theme_state.font_size, 12.0..=22.0)
                            .text("Font size"),
                    );
                    if ui
                        .toggle_value(
                            &mut self.settings.force_software,
                            "Force software rendering",
                        )
                        .clicked()
                    {
                        self.mark_dirty();
                        self.set_status(
                            "Restart required to apply rendering change",
                            StatusTone::Warn,
                        );
                        self.renderer.forced_software = self.settings.force_software;
                    }
                });
            });
        });
        // Apply theme updates as soon as the menu changes them.
        self.theme_state.apply(ctx);
        if self.settings.theme != self.theme_state {
            self.settings.theme = self.theme_state;
            self.mark_dirty();
        }
    }
}

impl eframe::App for MicroSerialApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.theme_state.apply(ctx);
        self.poll_scanner();
        self.poll_session();

        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            self.top_bar(ctx, ui);
        });

        egui::SidePanel::left("devices")
            .resizable(true)
            .show(ctx, |ui| {
                self.devices_panel(ui);
                ui.separator();
                self.configuration_panel(ui);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.empty_state(ui);
            self.console_panel(ui);
        });

        egui::TopBottomPanel::bottom("send_panel")
            .min_height(160.0)
            .show(ctx, |ui| {
                self.send_panel(ui);
            });

        self.diagnostics.show(ctx);
        self.save_settings_if_needed();
        ctx.request_repaint_after(Duration::from_millis(16));
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        let _ = self.settings.save();
    }
}
