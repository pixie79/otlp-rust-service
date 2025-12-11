use otlp_arrow_library::config::ConfigLoader;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_dashboard_config_from_yaml_disabled() {
    let yaml = r#"
output_dir: ./output
dashboard:
  enabled: false
"#;

    let temp_file = tempfile::NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), yaml).unwrap();

    let config = ConfigLoader::from_yaml(temp_file.path()).unwrap();

    assert!(!config.dashboard.enabled);
    assert_eq!(config.dashboard.port, 8080); // Default port
}

#[test]
fn test_dashboard_config_from_yaml_enabled() {
    let temp_dir = tempfile::tempdir().unwrap();
    let static_dir = temp_dir.path().join("dashboard").join("dist");
    fs::create_dir_all(&static_dir).unwrap();

    let yaml = format!(
        r#"
output_dir: ./output
dashboard:
  enabled: true
  port: 9000
  static_dir: {}
"#,
        static_dir.display()
    );

    let temp_file = tempfile::NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), yaml).unwrap();

    let config = ConfigLoader::from_yaml(temp_file.path()).unwrap();

    assert!(config.dashboard.enabled);
    assert_eq!(config.dashboard.port, 9000);
    assert_eq!(config.dashboard.static_dir, static_dir);
}

#[test]
fn test_dashboard_config_from_yaml_defaults() {
    let yaml = r#"
output_dir: ./output
"#;

    let temp_file = tempfile::NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), yaml).unwrap();

    let config = ConfigLoader::from_yaml(temp_file.path()).unwrap();

    // Dashboard should default to disabled
    assert!(!config.dashboard.enabled);
    assert_eq!(config.dashboard.port, 8080);
    assert_eq!(config.dashboard.static_dir, PathBuf::from("./dashboard/dist"));
}

#[test]
fn test_dashboard_config_from_yaml_partial() {
    let temp_dir = tempfile::tempdir().unwrap();
    let static_dir = temp_dir.path().join("dashboard").join("dist");
    fs::create_dir_all(&static_dir).unwrap();

    let yaml = format!(
        r#"
output_dir: ./output
dashboard:
  enabled: true
  static_dir: {}
"#,
        static_dir.display()
    );

    let temp_file = tempfile::NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), yaml).unwrap();

    let config = ConfigLoader::from_yaml(temp_file.path()).unwrap();

    assert!(config.dashboard.enabled);
    assert_eq!(config.dashboard.port, 8080); // Default port
    assert_eq!(config.dashboard.static_dir, static_dir);
}

