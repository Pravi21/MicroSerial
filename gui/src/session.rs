use std::sync::mpsc::{self, Receiver, Sender};

use crate::core::{SerialConfig, SerialPort};
use thiserror::Error;

#[derive(Debug, Clone)]
pub enum SessionMessage {
    Data(Vec<u8>),
    Event(SessionEvent),
}

#[derive(Debug, Clone)]
pub struct SessionEvent {
    pub code: i32,
    pub message: String,
}

pub struct SerialSession {
    port: SerialPort,
    rx: Receiver<SessionMessage>,
    _tx: Sender<SessionMessage>,
}

#[derive(Debug, Error)]
pub enum SessionError {
    #[error("open failed with code {0}")]
    Open(i32),
    #[error("configuration failed with code {0}")]
    Configure(i32),
    #[error("start failed with code {0}")]
    Start(i32),
    #[error("write failed with code {0}")]
    Write(i32),
    #[error("write truncated")]
    Truncated,
}

impl SerialSession {
    pub fn open(path: &str, config: &SerialConfig) -> Result<Self, SessionError> {
        let mut port = SerialPort::open(path).map_err(SessionError::Open)?;
        port.configure(config).map_err(SessionError::Configure)?;
        let (tx, rx) = mpsc::channel();
        let data_tx = tx.clone();
        let event_tx = tx.clone();
        port.start(
            move |bytes| {
                let _ = data_tx.send(SessionMessage::Data(bytes.to_vec()));
            },
            move |code, message| {
                let _ = event_tx.send(SessionMessage::Event(SessionEvent {
                    code,
                    message: message.to_string(),
                }));
            },
        )
        .map_err(SessionError::Start)?;
        Ok(Self { port, rx, _tx: tx })
    }

    pub fn poll(&self) -> Vec<SessionMessage> {
        let mut messages = Vec::new();
        while let Ok(msg) = self.rx.try_recv() {
            messages.push(msg);
        }
        messages
    }

    pub fn write(&mut self, data: &[u8]) -> Result<(), SessionError> {
        let written = self.port.write(data).map_err(SessionError::Write)?;
        if written != data.len() {
            return Err(SessionError::Truncated);
        }
        Ok(())
    }

    pub fn stop(&mut self) {
        self.port.stop();
    }
}

impl Drop for SerialSession {
    fn drop(&mut self) {
        self.stop();
    }
}
