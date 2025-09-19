use crate::core::SerialConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerialProfile {
    pub name: String,
    pub config: SerialConfig,
}

impl SerialProfile {
    pub fn new(name: impl Into<String>, config: SerialConfig) -> Self {
        Self {
            name: name.into(),
            config,
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ProfileStore {
    pub profiles: Vec<SerialProfile>,
    pub active: Option<String>,
}

impl ProfileStore {
    pub fn ensure_default(&mut self) {
        if self.profiles.is_empty() {
            self.profiles
                .push(SerialProfile::new("Default", SerialConfig::default()));
            self.active = Some("Default".to_string());
        }
    }

    pub fn get_active(&self) -> Option<&SerialProfile> {
        match &self.active {
            Some(name) => self.profiles.iter().find(|profile| profile.name == *name),
            None => None,
        }
    }

    pub fn set_active(&mut self, name: &str) {
        if self.profiles.iter().any(|profile| profile.name == name) {
            self.active = Some(name.to_string());
        }
    }

    pub fn upsert(&mut self, profile: SerialProfile) {
        if let Some(existing) = self
            .profiles
            .iter_mut()
            .find(|existing| existing.name == profile.name)
        {
            *existing = profile;
        } else {
            self.profiles.push(profile);
        }
    }

    pub fn delete(&mut self, name: &str) {
        self.profiles.retain(|profile| profile.name != name);
        if self.active.as_deref() == Some(name) {
            self.active = self.profiles.first().map(|profile| profile.name.clone());
        }
    }
}
