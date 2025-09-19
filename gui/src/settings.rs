use std::fs;
use std::path::PathBuf;

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::console::ConsoleViewMode;
use crate::profiles::ProfileStore;
use crate::theme::{ThemePreference, ThemeState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub theme: ThemeState,
    pub force_software: bool,
    pub profiles: ProfileStore,
    pub console_view: ConsoleViewMode,
    pub show_timestamps: bool,
}

impl Default for Settings {
    fn default() -> Self {
        let mut profiles = ProfileStore::default();
        profiles.ensure_default();
        Self {
            theme: ThemeState::default(),
            force_software: false,
            profiles,
            console_view: ConsoleViewMode::Mixed,
            show_timestamps: true,
        }
    }
}

#[derive(Debug, Error)]
pub enum SettingsError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialize(#[from] serde_json::Error),
    #[error("config directory unavailable")]
    MissingConfigDir,
}

impl Settings {
    pub fn load() -> Result<Self, SettingsError> {
        let path = config_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let data = fs::read_to_string(path)?;
        let mut settings: Settings = serde_json::from_str(&data)?;
        settings.profiles.ensure_default();
        Ok(settings)
    }

    pub fn save(&self) -> Result<(), SettingsError> {
        let path = config_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_string_pretty(self)?;
        fs::write(path, data)?;
        Ok(())
    }
}

pub fn config_path() -> Result<PathBuf, SettingsError> {
    if let Ok(custom) = std::env::var("MICROSERIAL_CONFIG_DIR") {
        let custom_path = PathBuf::from(custom);
        return Ok(custom_path.join("settings.json"));
    }
    let dirs = ProjectDirs::from("dev", "MicroSerial", "MicroSerial")
        .ok_or(SettingsError::MissingConfigDir)?;
    Ok(dirs.config_dir().join("settings.json"))
}

pub fn theme_options() -> Vec<ThemePreference> {
    vec![
        ThemePreference::System,
        ThemePreference::Light,
        ThemePreference::Dark,
    ]
}
