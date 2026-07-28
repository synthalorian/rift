use assert_cmd::Command;
use predicates::prelude::*;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper: build the rift binary path
fn rift_bin() -> Command {
    Command::cargo_bin("rift").expect("rift binary not found")
}

/// Helper: create a Command that runs in the given directory
fn rift_at(root: &PathBuf) -> Command {
    let mut cmd = rift_bin();
    cmd.current_dir(root);
    cmd
}

/// Helper: create a temporary directory with raw-assets/ subdirectory
fn with_temp_pipeline() -> (TempDir, PathBuf) {
    let dir = TempDir::new().unwrap();
    let root = dir.path().to_path_buf();
    std::fs::create_dir_all(root.join("raw-assets")).unwrap();
    (dir, root)
}

#[test]
fn test_init_creates_config() {
    let (_dir, root) = with_temp_pipeline();

    rift_at(&root)
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialized rift pipeline"));

    assert!(root.join("rift.yml").exists());
    assert!(root.join("raw-assets").exists());
    assert!(root.join(".rift").exists());
}

#[test]
fn test_init_unity_engine() {
    let (_dir, root) = with_temp_pipeline();

    rift_at(&root)
        .args(["init", "--engine", "unity"])
        .assert()
        .success();

    let config = std::fs::read_to_string(root.join("rift.yml")).unwrap();
    assert!(config.contains("unity"));
    assert!(config.contains("Assets/Rift"));
}

#[test]
fn test_init_godot_engine() {
    let (_dir, root) = with_temp_pipeline();

    rift_at(&root)
        .args(["init", "--engine", "godot"])
        .assert()
        .success();

    let config = std::fs::read_to_string(root.join("rift.yml")).unwrap();
    assert!(config.contains("godot"));
    assert!(config.contains("assets/rift"));
}

#[test]
fn test_init_unreal_engine() {
    let (_dir, root) = with_temp_pipeline();

    rift_at(&root)
        .args(["init", "--engine", "unreal"])
        .assert()
        .success();

    let config = std::fs::read_to_string(root.join("rift.yml")).unwrap();
    assert!(config.contains("unreal"));
    assert!(config.contains("Content/Rift"));
}

#[test]
fn test_init_rejects_existing_config() {
    let (_dir, root) = with_temp_pipeline();

    // First init succeeds
    rift_at(&root).arg("init").assert().success();

    // Second init fails
    rift_at(&root)
        .arg("init")
        .assert()
        .failure()
        .stderr(predicate::str::contains("rift.yml already exists"));
}

#[test]
fn test_run_fails_without_config() {
    let dir = TempDir::new().unwrap();
    let root = dir.path().to_path_buf();

    rift_at(&root)
        .arg("run")
        .assert()
        .failure()
        .stderr(predicate::str::contains("No rift.yml found"));
}

#[test]
fn test_run_fails_without_source_root() {
    let (_dir, root) = with_temp_pipeline();

    // Create rift.yml pointing to a non-existent source
    let config = r#"
pipeline:
  name: "test"
  version: 1

source:
  root: "nonexistent"
  watch: false

targets:
  - engine: unity
    project: "out/unity"
    textures:
      format: png
      max_width: 256
      max_height: 256
    audio:
      format: ogg
      bitrate: "128k"

rules: []
"#;
    std::fs::write(root.join("rift.yml"), config).unwrap();

    rift_at(&root)
        .arg("run")
        .assert()
        .failure()
        .stderr(predicate::str::contains("does not exist"));
}

#[test]
fn test_status_reports_no_database() {
    let (_dir, root) = with_temp_pipeline();
    // Init creates .rift/ dir, then we remove it so status fails
    rift_at(&root).arg("init").assert().success();
    std::fs::remove_dir_all(root.join(".rift")).unwrap();

    rift_at(&root)
        .arg("status")
        .assert()
        .failure()
        .stderr(predicate::str::contains("No rift database found"));
}

#[test]
fn test_completions_bash() {
    rift_bin()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("_rift"));
}

#[test]
fn test_completions_zsh() {
    rift_bin()
        .args(["completions", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("#compdef"));
}

#[test]
fn test_completions_fish() {
    rift_bin()
        .args(["completions", "fish"])
        .assert()
        .success()
        .stdout(predicate::str::contains("complete"));
}

#[test]
fn test_upgrade_parses() {
    // Just verify the upgrade command parses without crashing
    rift_bin().arg("upgrade").assert().success();
}

#[test]
fn test_help_shows_all_commands() {
    rift_bin()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("init"))
        .stdout(predicate::str::contains("run"))
        .stdout(predicate::str::contains("watch"))
        .stdout(predicate::str::contains("status"))
        .stdout(predicate::str::contains("completions"))
        .stdout(predicate::str::contains("upgrade"));
}

#[test]
fn test_full_pipeline_roundtrip() {
    let (_dir, root) = with_temp_pipeline();

    // Create a small test PNG using the image crate directly
    let img_path = root.join("raw-assets").join("test.png");
    let img = image::RgbaImage::from_fn(64, 64, |_, _| image::Rgba([255, 0, 0, 255]));
    img.save(&img_path).unwrap();

    // Init and run
    rift_at(&root).arg("init").assert().success();
    rift_at(&root).arg("run").assert().success();

    // Status should show the asset
    rift_at(&root)
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("Total:"));

    // Status with --assets should list the file
    rift_at(&root)
        .args(["status", "--assets"])
        .assert()
        .success()
        .stdout(predicate::str::contains("test.png"));
}
