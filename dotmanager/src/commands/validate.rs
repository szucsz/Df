use crate::{
    config::{GlobalConfig, UnitManifest},
    error::AppError,
    loader::{load_global_config, load_unit_manifest},
};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub fn run_validate() -> Result<()> {
    println!("Validating configuration...");

    let config_path = Path::new("dotfiles.toml");
    let global_config: GlobalConfig = load_global_config(config_path)
        .map_err(|e| anyhow::anyhow!(e)) // Convert AppError to anyhow::Error for this command
        .context(format!("Failed to load global config: {}", config_path.display()))?;

    println!("Global config loaded successfully from {}", config_path.display());

    let root_dir = config_path.parent().unwrap_or_else(|| Path::new("."));
    let units_base_dir_name = global_config.units_base_dir();
    let units_base_path = root_dir.join(units_base_dir_name);

    if !units_base_path.exists() {
        return Err(anyhow::anyhow!(AppError::InvalidPath(format!(
            "Units base directory '{}' not found",
            units_base_path.display()
        ))));
    }
    println!("Units base directory: {}", units_base_path.display());

    let mut all_valid = true;

    for unit_name in &global_config.units {
        println!("\nValidating unit: {}", unit_name);
        let unit_dir = units_base_path.join(unit_name);

        if !unit_dir.is_dir() {
            eprintln!("Error: Unit directory '{}' not found.", unit_dir.display());
            all_valid = false;
            continue;
        }
        println!("Found unit directory: {}", unit_dir.display());

        let unit_manifest_path = unit_dir.join("unit.toml");
        if !unit_manifest_path.is_file() {
            eprintln!("Error: Unit manifest '{}' not found.", unit_manifest_path.display());
            all_valid = false;
            continue;
        }
        println!("Found unit manifest: {}", unit_manifest_path.display());

        match load_unit_manifest(&unit_dir) {
            Ok(unit_manifest) => {
                println!("Successfully loaded unit manifest for '{}'", unit_name);

                // Validate templates_dir
                let templates_dir_name = unit_manifest.templates_dir();
                let templates_path = unit_dir.join(templates_dir_name);

                if !templates_path.is_dir() {
                    eprintln!(
                        "Warning: Templates directory '{}' not found for unit '{}'.",
                        templates_path.display(),
                        unit_name
                    );
                    // Depending on strictness, you might set all_valid = false here
                } else {
                    println!("Found templates directory: {}", templates_path.display());
                }
            }
            Err(e) => {
                eprintln!("Error loading unit manifest for '{}': {}", unit_name, e);
                all_valid = false;
            }
        }
    }

    if all_valid {
        println!("\nConfiguration validated successfully.");
        Ok(())
    } else {
        Err(anyhow::anyhow!("Configuration validation failed."))
    }
}
