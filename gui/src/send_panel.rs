use std::collections::VecDeque;

use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SendMode {
    Text,
    Hex,
}

impl SendMode {
    pub fn label(self) -> &'static str {
        match self {
            SendMode::Text => "Text",
            SendMode::Hex => "Hex",
        }
    }
}

#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub value: String,
    pub mode: SendMode,
    pub favorited: bool,
}

pub struct SendPanelState {
    pub input: String,
    pub mode: SendMode,
    pub history: VecDeque<HistoryEntry>,
    pub max_history: usize,
}

impl SendPanelState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_history(&mut self, value: String) {
        if value.trim().is_empty() {
            return;
        }
        let entry = HistoryEntry {
            value: value.clone(),
            mode: self.mode,
            favorited: false,
        };
        self.history.push_front(entry);
        while self.history.len() > self.max_history {
            self.history.pop_back();
        }
        self.input.clear();
    }

    pub fn toggle_favorite(&mut self, index: usize) {
        if let Some(entry) = self.history.get_mut(index) {
            entry.favorited = !entry.favorited;
        }
    }

    pub fn favorites(&self) -> impl Iterator<Item = &HistoryEntry> {
        self.history.iter().filter(|entry| entry.favorited)
    }

    pub fn parse_payload(&self, value: &str) -> Result<Vec<u8>, PayloadError> {
        match self.mode {
            SendMode::Text => Ok(value.as_bytes().to_vec()),
            SendMode::Hex => parse_hex(value),
        }
    }
}

impl Default for SendPanelState {
    fn default() -> Self {
        Self {
            input: String::new(),
            mode: SendMode::Text,
            history: VecDeque::new(),
            max_history: 50,
        }
    }
}

#[derive(Debug, Error)]
pub enum PayloadError {
    #[error("invalid hex sequence")]
    InvalidHex,
}

fn parse_hex(input: &str) -> Result<Vec<u8>, PayloadError> {
    let mut bytes = Vec::new();
    let mut buffer = String::new();
    for ch in input.chars() {
        if ch.is_ascii_hexdigit() {
            buffer.push(ch);
            if buffer.len() == 2 {
                let byte = u8::from_str_radix(&buffer, 16).map_err(|_| PayloadError::InvalidHex)?;
                bytes.push(byte);
                buffer.clear();
            }
        } else if ch.is_ascii_whitespace() {
            continue;
        } else {
            return Err(PayloadError::InvalidHex);
        }
    }
    if !buffer.is_empty() {
        return Err(PayloadError::InvalidHex);
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::{SendMode, SendPanelState, parse_hex};

    #[test]
    fn parse_hex_accepts_whitespace() {
        let bytes = parse_hex("DE AD BE EF").expect("parse");
        assert_eq!(bytes, vec![0xDE, 0xAD, 0xBE, 0xEF]);
    }

    #[test]
    fn parse_hex_rejects_invalid() {
        assert!(parse_hex("gg").is_err());
    }

    #[test]
    fn history_tracks_recent_entries() {
        let mut panel = SendPanelState::new();
        panel.mode = SendMode::Text;
        panel.input = "hello".to_string();
        panel.push_history(panel.input.clone());
        assert_eq!(panel.history.len(), 1);
        assert_eq!(panel.history.front().unwrap().value, "hello");
    }
}
