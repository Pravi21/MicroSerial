use std::fs;

use microserial_gui::core::SerialConfig;
use microserial_gui::profiles::SerialProfile;
use microserial_gui::settings::Settings;
use microserial_gui::theme::ThemePreference;
use tempfile::tempdir;

#[test]
fn settings_round_trip() {
    let dir = tempdir().expect("tempdir");
    unsafe {
        std::env::set_var("MICROSERIAL_CONFIG_DIR", dir.path());
    }

    let mut settings = Settings::default();
    settings.force_software = true;
    settings.theme.preference = ThemePreference::Dark;
    settings
        .profiles
        .upsert(SerialProfile::new("Lab", SerialConfig::default()));
    settings.save().expect("save");

    let loaded = Settings::load().expect("load");
    assert!(loaded.force_software);
    assert_eq!(loaded.theme.preference, ThemePreference::Dark);
    assert!(!loaded.profiles.profiles.is_empty());

    unsafe {
        std::env::remove_var("MICROSERIAL_CONFIG_DIR");
    }
    dir.close().expect("close");
}

#[test]
fn settings_file_is_json() {
    let dir = tempdir().expect("tempdir");
    unsafe {
        std::env::set_var("MICROSERIAL_CONFIG_DIR", dir.path());
    }

    let settings = Settings::default();
    settings.save().expect("save");

    let file = dir.path().join("settings.json");
    let content = fs::read_to_string(file).expect("read");
    assert!(content.contains("theme"));

    unsafe {
        std::env::remove_var("MICROSERIAL_CONFIG_DIR");
    }
    dir.close().expect("close");
}
