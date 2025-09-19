use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

use crate::core::{SerialDevice, list_serial_ports};

#[derive(Debug, Clone)]
pub struct ScanResult {
    pub devices: Vec<SerialDevice>,
    pub completed_at: Instant,
    pub duration: Duration,
    pub error: Option<String>,
}

pub struct DeviceScanner {
    pending: Option<Receiver<ScanResult>>,
    last_result: Option<ScanResult>,
    last_started: Option<Instant>,
    pub scanning: bool,
}

impl DeviceScanner {
    pub fn new() -> Self {
        Self {
            pending: None,
            last_result: None,
            last_started: None,
            scanning: false,
        }
    }

    pub fn last_result(&self) -> Option<&ScanResult> {
        self.last_result.as_ref()
    }

    pub fn refresh(&mut self) {
        if self.scanning {
            return;
        }
        let (tx, rx) = mpsc::channel();
        self.scanning = true;
        self.last_started = Some(Instant::now());
        thread::spawn(move || {
            let started = Instant::now();
            let result = match list_serial_ports() {
                Ok(devices) => ScanResult {
                    devices,
                    completed_at: Instant::now(),
                    duration: started.elapsed(),
                    error: None,
                },
                Err(code) => ScanResult {
                    devices: Vec::new(),
                    completed_at: Instant::now(),
                    duration: started.elapsed(),
                    error: Some(format!("enumeration failed ({code})")),
                },
            };
            let _ = tx.send(result);
        });
        self.pending = Some(rx);
    }

    pub fn poll(&mut self) -> Option<&ScanResult> {
        if let Some(rx) = &self.pending {
            if let Ok(result) = rx.try_recv() {
                self.last_result = Some(result);
                self.pending = None;
                self.scanning = false;
            }
        }
        self.last_result()
    }

    pub fn auto_refresh_due(&self, interval: Duration) -> bool {
        if self.scanning {
            return false;
        }
        match (&self.last_result, self.last_started) {
            (Some(result), _) => result.completed_at.elapsed() >= interval,
            (None, Some(start)) => start.elapsed() >= interval,
            _ => true,
        }
    }
}
