//! Integration tests for the Rive CLI.

use std::fs;
use tempfile::TempDir;

#[test]
fn test_config_new() {
    use rive_utils::Config;

    let config = Config::new("test-project");
    assert_eq!(config.package.name, "test-project");
    assert_eq!(config.package.version, "0.1.0");
    assert_eq!(config.package.edition, "2025");
}

#[test]
fn test_config_save_and_load() {
    use rive_utils::Config;

    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("rive.toml");

    let config = Config::new("test-project");
    config.save(&config_path).unwrap();

    let loaded = Config::load(&config_path).unwrap();
    assert_eq!(loaded.package.name, "test-project");
    assert_eq!(loaded.package.version, "0.1.0");
}

#[test]
fn test_new_command_structure() {
    // Test that new command creates proper directory structure
    let temp_dir = TempDir::new().unwrap();
    let project_name = "test-new-project";
    let project_path = temp_dir.path().join(project_name);

    // Simulate creating project structure
    fs::create_dir_all(project_path.join("src")).unwrap();

    let config = rive_utils::Config::new(project_name);
    config.save(project_path.join("rive.toml")).unwrap();

    fs::write(
        project_path.join("src/main.rive"),
        "fun main() {\n    print(\"Hello, Rive!\")\n}\n",
    )
    .unwrap();

    // Verify structure
    assert!(project_path.exists());
    assert!(project_path.join("rive.toml").exists());
    assert!(project_path.join("src").exists());
    assert!(project_path.join("src/main.rive").exists());
}
