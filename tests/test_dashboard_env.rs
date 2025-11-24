use otlp_arrow_library::config::ConfigLoader;
use std::env;
use std::path::PathBuf;

#[test]
fn test_dashboard_config_from_env_disabled() {
    env::set_var("OTLP_DASHBOARD_ENABLED", "false");
    env::remove_var("OTLP_DASHBOARD_PORT");
    env::remove_var("OTLP_DASHBOARD_STATIC_DIR");

    let config = ConfigLoader::from_env().unwrap();

    assert!(!config.dashboard.enabled);
    assert_eq!(config.dashboard.port, 8080); // Default

    env::remove_var("OTLP_DASHBOARD_ENABLED");
}

#[test]
fn test_dashboard_config_from_env_enabled() {
    let temp_dir = tempfile::tempdir().unwrap();
    let static_dir = temp_dir.path().join("dashboard").join("dist");
    std::fs::create_dir_all(&static_dir).unwrap();

    env::set_var("OTLP_DASHBOARD_ENABLED", "true");
    env::set_var("OTLP_DASHBOARD_PORT", "9000");
    env::set_var("OTLP_DASHBOARD_STATIC_DIR", static_dir.to_str().unwrap());

    let config = ConfigLoader::from_env().unwrap();

    assert!(config.dashboard.enabled);
    assert_eq!(config.dashboard.port, 9000);
    assert_eq!(config.dashboard.static_dir, static_dir);

    env::remove_var("OTLP_DASHBOARD_ENABLED");
    env::remove_var("OTLP_DASHBOARD_PORT");
    env::remove_var("OTLP_DASHBOARD_STATIC_DIR");
}

#[test]
fn test_dashboard_config_from_env_defaults() {
    env::remove_var("OTLP_DASHBOARD_ENABLED");
    env::remove_var("OTLP_DASHBOARD_PORT");
    env::remove_var("OTLP_DASHBOARD_STATIC_DIR");

    let config = ConfigLoader::from_env().unwrap();

    // Dashboard should default to disabled
    assert!(!config.dashboard.enabled);
    assert_eq!(config.dashboard.port, 8080);
    assert_eq!(
        config.dashboard.static_dir,
        PathBuf::from("./dashboard/dist")
    );
}

#[test]
fn test_dashboard_config_from_env_partial() {
    let temp_dir = tempfile::tempdir().unwrap();
    let static_dir = temp_dir.path().join("dashboard").join("dist");
    std::fs::create_dir_all(&static_dir).unwrap();

    env::set_var("OTLP_DASHBOARD_ENABLED", "true");
    env::set_var("OTLP_DASHBOARD_STATIC_DIR", static_dir.to_str().unwrap());
    env::remove_var("OTLP_DASHBOARD_PORT");

    let config = ConfigLoader::from_env().unwrap();

    assert!(config.dashboard.enabled);
    assert_eq!(config.dashboard.port, 8080); // Default port
    assert_eq!(config.dashboard.static_dir, static_dir);

    env::remove_var("OTLP_DASHBOARD_ENABLED");
    env::remove_var("OTLP_DASHBOARD_STATIC_DIR");
}

#[test]
fn test_dashboard_config_env_overrides_yaml() {
    let temp_dir = tempfile::tempdir().unwrap();
    let static_dir = temp_dir.path().join("dashboard").join("dist");
    std::fs::create_dir_all(&static_dir).unwrap();

    let yaml = r#"
output_dir: ./output
dashboard:
  enabled: false
  port: 8080
"#;

    let temp_file = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(temp_file.path(), yaml).unwrap();

    env::set_var("OTLP_DASHBOARD_ENABLED", "true");
    env::set_var("OTLP_DASHBOARD_PORT", "9000");
    env::set_var("OTLP_DASHBOARD_STATIC_DIR", static_dir.to_str().unwrap());

    let config = ConfigLoader::from_yaml(temp_file.path()).unwrap();

    // Environment variables should override YAML
    assert!(config.dashboard.enabled); // Overridden by env
    assert_eq!(config.dashboard.port, 9000); // Overridden by env
    assert_eq!(config.dashboard.static_dir, static_dir); // Overridden by env

    env::remove_var("OTLP_DASHBOARD_ENABLED");
    env::remove_var("OTLP_DASHBOARD_PORT");
    env::remove_var("OTLP_DASHBOARD_STATIC_DIR");
}
