use microserial_gui::renderer::{self, LaunchConfig};

#[test]
fn software_fallback_forced_by_env() {
    let key = "MICROSERIAL_FORCE_SOFTWARE";
    let original = std::env::var(key).ok();
    let libgl_key = "LIBGL_ALWAYS_SOFTWARE";
    let original_libgl = std::env::var(libgl_key).ok();
    unsafe {
        std::env::set_var(key, "1");
    }
    let launch = LaunchConfig::from_args();
    assert!(launch.force_software);
    assert!(launch.env_forced);
    assert_eq!(std::env::var(libgl_key).ok().as_deref(), Some("1"));
    if let Some(value) = original {
        unsafe {
            std::env::set_var(key, value);
        }
    } else {
        unsafe {
            std::env::remove_var(key);
        }
    }
    if let Some(value) = original_libgl {
        unsafe {
            std::env::set_var(libgl_key, value);
        }
    } else {
        unsafe {
            std::env::remove_var(libgl_key);
        }
    }
}

#[test]
fn headless_probe_reports_backend() {
    let launch = LaunchConfig::from_args();
    let report = renderer::run_headless_probe(&launch);
    assert!(!report.diagnostics.backend.is_empty() || report.diagnostics.failure_reason.is_some());
}
