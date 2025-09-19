mod core;

use crate::core::{SerialDevice, SerialPort, default_config, list_serial_ports};
use eframe::egui;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Instant;

#[derive(Debug)]
enum UiMessage {
    Data(String),
    Event(String),
}

struct MicroSerialApp {
    ports: Vec<SerialDevice>,
    selected: Option<usize>,
    session: Option<SerialSession>,
    log: Vec<String>,
    input: String,
    last_refresh: Instant,
}

struct SerialSession {
    port: SerialPort,
    rx: Receiver<UiMessage>,
    _tx: Sender<UiMessage>,
}

impl SerialSession {
    fn new(mut port: SerialPort) -> Result<Self, String> {
        let (tx, rx) = mpsc::channel();
        let data_tx = tx.clone();
        port.configure(&default_config())
            .map_err(|e| format!("configure error {e}"))?;
        port.start(
            move |bytes| {
                if let Ok(text) = String::from_utf8(bytes.to_vec()) {
                    let _ = data_tx.send(UiMessage::Data(text));
                } else {
                    let hex = bytes
                        .iter()
                        .map(|b| format!("{b:02X}"))
                        .collect::<Vec<_>>()
                        .join(" ");
                    let _ = data_tx.send(UiMessage::Data(format!("[{hex}]")));
                }
            },
            move |code, message| {
                let _ = tx.send(UiMessage::Event(format!("event {code}: {message}")));
            },
        )
        .map_err(|e| format!("start error {e}"))?;
        Ok(Self { port, rx, _tx: tx })
    }

    fn write(&mut self, text: &str) -> Result<(), String> {
        let payload = text.as_bytes();
        let written = self
            .port
            .write(payload)
            .map_err(|e| format!("write failed {e}"))?;
        if written != payload.len() {
            Err("write truncated".to_string())
        } else {
            Ok(())
        }
    }
}

impl Drop for SerialSession {
    fn drop(&mut self) {
        self.port.stop();
    }
}

impl MicroSerialApp {
    fn new() -> Self {
        let ports = list_serial_ports().unwrap_or_default();
        Self {
            ports,
            selected: None,
            session: None,
            log: Vec::new(),
            input: String::new(),
            last_refresh: Instant::now(),
        }
    }

    fn refresh_ports(&mut self) {
        if let Ok(ports) = list_serial_ports() {
            self.ports = ports;
            self.last_refresh = Instant::now();
        }
    }

    fn poll_messages(&mut self) {
        if let Some(session) = &self.session {
            while let Ok(msg) = session.rx.try_recv() {
                match msg {
                    UiMessage::Data(text) => self.log.push(format!("RX: {text}")),
                    UiMessage::Event(text) => self.log.push(format!("{text}")),
                }
            }
        }
    }
}

impl eframe::App for MicroSerialApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_messages();
        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.heading("MicroSerial");
            if ui.button("Refresh ports").clicked() {
                self.refresh_ports();
            }
            ui.label(format!("Ports discovered: {}", self.ports.len()));
        });

        egui::SidePanel::left("ports").show(ctx, |ui| {
            ui.heading("Ports");
            for (idx, port) in self.ports.iter().enumerate() {
                let label = format!("{} — {}", port.path, port.description);
                if ui
                    .selectable_label(self.selected == Some(idx), label)
                    .clicked()
                {
                    self.selected = Some(idx);
                }
            }
            if ui.button("Open").clicked() {
                if let Some(idx) = self.selected {
                    self.session = None;
                    let path = self.ports[idx].path.clone();
                    match SerialPort::open(&path) {
                        Ok(port) => match SerialSession::new(port) {
                            Ok(session) => {
                                self.log.push(format!("Opened {path}"));
                                self.session = Some(session);
                            }
                            Err(err) => {
                                self.log.push(format!("Failed start {err}"));
                            }
                        },
                        Err(code) => {
                            self.log.push(format!("Open failed ({code})"));
                        }
                    }
                }
            }
            if ui.button("Close").clicked() {
                self.session = None;
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Console");
            egui::ScrollArea::vertical().show(ui, |ui| {
                for entry in &self.log {
                    ui.label(entry);
                }
            });
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Send");
                let response = ui.text_edit_singleline(&mut self.input);
                if ui.button("Send").clicked()
                    || response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))
                {
                    if let Some(session) = self.session.as_mut() {
                        if !self.input.is_empty() {
                            if let Err(err) = session.write(&self.input) {
                                self.log.push(err);
                            } else {
                                self.log.push(format!("TX: {}", self.input));
                            }
                            self.input.clear();
                        }
                    }
                }
            });
        });
        ctx.request_repaint_after(std::time::Duration::from_millis(16));
    }
}

fn main() -> eframe::Result {
    env_logger::init();
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "MicroSerial",
        options,
        Box::new(|_| Box::new(MicroSerialApp::new())),
    )
}
