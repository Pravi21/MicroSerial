use microserial_gui::renderer::{self, LaunchConfig, RendererKind};

#[test]
fn software_fallback_forced_by_env() {
    let key = "MICROSERIAL_FORCE_SOFTWARE";
    let original = std::env::var(key).ok();
    unsafe {
        std::env::set_var(key, "1");
    }
    let launch = LaunchConfig::from_args();
    let decision = renderer::detect(&launch);
    assert_eq!(decision.kind, RendererKind::Glow);
    assert!(decision.diagnostics.forced_software);
    if let Some(value) = original {
        unsafe {
            std::env::set_var(key, value);
        }
    } else {
        unsafe {
            std::env::remove_var(key);
        }
    }
}

#[test]
fn headless_probe_reports_backend() {
    let launch = LaunchConfig::from_args();
    let report = renderer::run_headless_probe(&launch);
    assert!(!report.diagnostics.backend.is_empty());
}
