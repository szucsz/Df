use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command; // Used for Command::new()

use assert_cmd::prelude::*; // For Command methods
use predicates::prelude::*; // For predicate::str::contains
use tempfile::tempdir;

// Helper function to get the path to the compiled binary
fn get_binary_path() -> PathBuf {
    assert_cmd::cargo::cargo_bin("dotmanager")
}

#[test]
fn init_command_success() {
    let temp_dir = tempdir().unwrap();
    let cmd = Command::new(get_binary_path())
        .current_dir(temp_dir.path())
        .arg("init")
        .assert()
        .success();

    // Verify output contains expected messages
    cmd.stdout(predicate::str::contains("Created dotfiles.toml"))
       .stdout(predicate::str::contains("Created directory units"))
       .stdout(predicate::str::contains("Created directory units/example"))
       .stdout(predicate::str::contains("Created units/example/unit.toml"))
       .stdout(predicate::str::contains("Created directory units/example/templates"))
       .stdout(predicate::str::contains("Created units/example/templates/example_config.txt.template"))
       .stdout(predicate::str::contains("Initialized dotfiles repository."));


    assert!(temp_dir.path().join("dotfiles.toml").exists());
    assert!(temp_dir.path().join("units/example/unit.toml").exists());
    assert!(temp_dir.path().join("units/example/templates/example_config.txt.template").exists());

    // Optionally verify content of dotfiles.toml
    let dotfiles_content = fs::read_to_string(temp_dir.path().join("dotfiles.toml")).unwrap();
    assert!(dotfiles_content.contains("units_base_dir = \"units\""));
    assert!(dotfiles_content.contains("units = [\n  \"example\"\n]"));
}

#[test]
fn add_unit_command_success() {
    let temp_dir = tempdir().unwrap();

    // Run init first
    Command::new(get_binary_path())
        .current_dir(temp_dir.path())
        .arg("init")
        .assert()
        .success();

    // Run add-unit
    Command::new(get_binary_path())
        .current_dir(temp_dir.path())
        .arg("add-unit")
        .arg("mytestunit")
        .assert()
        .success()
        .stdout(predicate::str::contains("Successfully created unit 'mytestunit'"))
        .stdout(predicate::str::contains("3. Add the unit name \"mytestunit\" to the 'units' array in 'dotfiles.toml'."));


    assert!(temp_dir.path().join("units/mytestunit/unit.toml").exists());
    assert!(temp_dir.path().join("units/mytestunit/templates/mytestunit.conf.template").exists());
}

#[test]
fn validate_command_success_after_init() {
    let temp_dir = tempdir().unwrap();

    Command::new(get_binary_path())
        .current_dir(temp_dir.path())
        .arg("init")
        .assert()
        .success();

    Command::new(get_binary_path())
        .current_dir(temp_dir.path())
        .arg("validate")
        .assert()
        .success()
        .stdout(predicate::str::contains("Configuration validated successfully."));
}

#[test]
fn validate_command_fail_missing_unit_dir() {
    let temp_dir = tempdir().unwrap();
    let root_path = temp_dir.path();

    Command::new(get_binary_path())
        .current_dir(root_path)
        .arg("init")
        .assert()
        .success();

    // Manually edit dotfiles.toml to include a nonexistent unit
    let dotfiles_path = root_path.join("dotfiles.toml");
    let mut dotfiles_content = fs::read_to_string(&dotfiles_path).unwrap();
    dotfiles_content = dotfiles_content.replace(
        "units = [\n  \"example\"\n]",
        "units = [\n  \"example\",\n  \"nonexistentunit\"\n]"
    );
    fs::write(&dotfiles_path, dotfiles_content).unwrap();

    Command::new(get_binary_path())
        .current_dir(root_path)
        .arg("validate")
        .assert()
        .failure() // Expect the command to fail
        .stderr(predicate::str::contains("Error: Unit directory")) // Check stderr
        .stderr(predicate::str::contains("units/nonexistentunit' not found"));
}


#[test]
fn apply_command_simple_case() {
    let temp_dir = tempdir().unwrap();
    let root_dir = temp_dir.path();

    // Run init
    Command::new(get_binary_path())
        .current_dir(root_dir)
        .arg("init")
        .assert()
        .success();

    // Modify units/example/unit.toml
    let unit_toml_path = root_dir.join("units/example/unit.toml");
    let unit_toml_content = r#"
name = "example"
target_dir = "./output" # Relative to root_dir for test simplicity
templates_dir = "templates"

[variables]
feature_enabled = false
message = "Applied by test"
"#;
    fs::write(&unit_toml_path, unit_toml_content).unwrap();

    // Modify dotfiles.toml
    let dotfiles_path = root_dir.join("dotfiles.toml");
    let dotfiles_content_orig = fs::read_to_string(&dotfiles_path).unwrap();
    let dotfiles_content_new = format!(
        "{}\n\n[global_variables]\nusername = \"testuser\"",
        dotfiles_content_orig
    );
    fs::write(&dotfiles_path, dotfiles_content_new).unwrap();
    
    // Run apply
    Command::new(get_binary_path())
        .current_dir(root_dir)
        .arg("apply")
        .assert()
        .success();

    let target_file_path = root_dir.join("output/example_config.txt");
    assert!(target_file_path.exists());

    let target_content = fs::read_to_string(target_file_path).unwrap();
    assert!(target_content.contains("Applied by test"));
    assert!(target_content.contains("Feature status: Disabled")); // Based on feature_enabled = false
    assert!(target_content.contains("Global username: testuser"));
}

#[test]
fn apply_command_target_dir_tilde_expansion_mocked_home() {
    let root_dir_temp = tempdir().unwrap(); // Main directory for dotfiles.toml, units, etc.
    let root_dir = root_dir_temp.path();

    let mock_home_temp = tempdir().unwrap(); // This will be our mocked HOME
    let mock_home = mock_home_temp.path();

    // It's good practice to create .config if the target_dir expects it
    // fs::create_dir_all(mock_home.join(".config")).unwrap(); // Not strictly necessary if target_dir creates parents

    // Run init in root_dir
    Command::new(get_binary_path())
        .current_dir(root_dir)
        .arg("init")
        .assert()
        .success();

    // Modify units/example/unit.toml
    let unit_toml_path = root_dir.join("units/example/unit.toml");
    let unit_toml_content = r#"
name = "example_tilde"
target_dir = "~/.config/testapp_dotmanager" # Using a unique app name
templates_dir = "templates"

[variables]
message = "Tilde expansion test"
feature_enabled = true
"#;
    fs::write(&unit_toml_path, unit_toml_content).unwrap();

    // No need to change global_variables for this test specifically unless template uses them.
    // The default example template does use global_variables.username, so let's add it.
    let dotfiles_path = root_dir.join("dotfiles.toml");
    let dotfiles_content_orig = fs::read_to_string(&dotfiles_path).unwrap();
    let dotfiles_content_new = format!(
        "{}\n\n[global_variables]\nusername = \"tilde_user\"",
        dotfiles_content_orig
    );
    fs::write(&dotfiles_path, dotfiles_content_new).unwrap();

    // Run apply with HOME env var set
    Command::new(get_binary_path())
        .current_dir(root_dir)
        .env("HOME", mock_home) // Set HOME to our mock_home
        .arg("apply")
        .assert()
        .success();

    let target_file_path = mock_home.join(".config/testapp_dotmanager/example_config.txt");
    assert!(target_file_path.exists(), "Target file does not exist at {}", target_file_path.display());

    let target_content = fs::read_to_string(target_file_path).unwrap();
    assert!(target_content.contains("Tilde expansion test"));
    assert!(target_content.contains("Feature status: Enabled"));
    assert!(target_content.contains("Global username: tilde_user"));
}
