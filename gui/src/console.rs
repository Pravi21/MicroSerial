use std::fmt;

use serde::{Deserialize, Serialize};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    Rx,
    Tx,
    Event,
}

impl Direction {
    pub fn label(self) -> &'static str {
        match self {
            Direction::Rx => "RX",
            Direction::Tx => "TX",
            Direction::Event => "EVT",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsoleViewMode {
    Text,
    Hex,
    Mixed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleEntry {
    pub timestamp: OffsetDateTime,
    pub direction: Direction,
    pub text: String,
    pub hex: String,
}

impl ConsoleEntry {
    pub fn matches(&self, filter: &str) -> bool {
        if filter.trim().is_empty() {
            return true;
        }
        let filter_lower = filter.to_ascii_lowercase();
        self.text.to_ascii_lowercase().contains(&filter_lower)
            || self.hex.to_ascii_lowercase().contains(&filter_lower)
    }
}

impl fmt::Display for ConsoleEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ts = self.timestamp.format(&Rfc3339).unwrap_or_default();
        write!(f, "[{ts}] {} {}", self.direction.label(), self.text)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleBuffer {
    pub entries: Vec<ConsoleEntry>,
    pub show_timestamps: bool,
    pub view_mode: ConsoleViewMode,
    pub filter: String,
}

impl Default for ConsoleBuffer {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            show_timestamps: true,
            view_mode: ConsoleViewMode::Mixed,
            filter: String::new(),
        }
    }
}

impl ConsoleBuffer {
    pub fn push_rx(&mut self, data: &[u8]) {
        self.entries.push(ConsoleEntry {
            timestamp: OffsetDateTime::now_utc(),
            direction: Direction::Rx,
            text: String::from_utf8_lossy(data).to_string(),
            hex: to_hex(data),
        });
    }

    pub fn push_tx(&mut self, data: &[u8]) {
        self.entries.push(ConsoleEntry {
            timestamp: OffsetDateTime::now_utc(),
            direction: Direction::Tx,
            text: String::from_utf8_lossy(data).to_string(),
            hex: to_hex(data),
        });
    }

    pub fn push_event(&mut self, message: &str) {
        self.entries.push(ConsoleEntry {
            timestamp: OffsetDateTime::now_utc(),
            direction: Direction::Event,
            text: message.to_string(),
            hex: to_hex(message.as_bytes()),
        });
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn iter(&self) -> impl Iterator<Item = &ConsoleEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.matches(&self.filter))
    }
}

fn to_hex(data: &[u8]) -> String {
    data.iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}
