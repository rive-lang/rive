//! Tests for configuration handling.

use rive_utils::Config;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_config_new() {
    let config = Config::new("test-project");
    assert_eq!(config.package.name, "test-project");
    assert_eq!(config.package.version, "0.1.0");
    assert_eq!(config.package.edition, "2025");
}

#[test]
fn test_config_save_and_load() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("rive.toml");

    let config = Config::new("test-project");
    config.save(&config_path).unwrap();

    let loaded = Config::load(&config_path).unwrap();
    assert_eq!(loaded.package.name, "test-project");
    assert_eq!(loaded.package.version, "0.1.0");
    assert_eq!(loaded.package.edition, "2025");
}

#[test]
fn test_config_find() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = temp_dir.path();

    // Create a rive.toml in the temp directory
    let config = Config::new("find-test");
    config.save(project_dir.join("rive.toml")).unwrap();

    // Create a subdirectory
    let sub_dir = project_dir.join("src");
    fs::create_dir(&sub_dir).unwrap();

    // Change to the subdirectory and test find
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&sub_dir).unwrap();

    let result = Config::find();

    // Restore original directory
    std::env::set_current_dir(original_dir).unwrap();

    // Verify we found the config
    assert!(result.is_ok());
    let (found_config, found_dir) = result.unwrap();
    assert_eq!(found_config.package.name, "find-test");
    assert_eq!(found_dir, project_dir);
}
