use std::{fs, path::{Path, PathBuf}};
use crate::{
    config::GlobalConfig,
    loader::load_global_config,
};
use anyhow::{Context, Result};

pub fn run_add_unit(unit_name: &str) -> Result<()> {
    println!("Attempting to add new unit: {}", unit_name);

    // Try to load GlobalConfig to determine units_base_dir.
    // If dotfiles.toml doesn't exist or is invalid, default to "units".
    let units_root_path = match load_global_config(Path::new("dotfiles.toml")) {
        Ok(cfg) => PathBuf::from(cfg.units_base_dir()),
        Err(_) => {
            println!("Info: Could not load 'dotfiles.toml' or it's invalid. Defaulting units base directory to 'units'.");
            PathBuf::from("units")
        }
    };

    let new_unit_path = units_root_path.join(unit_name);

    if new_unit_path.exists() {
        eprintln!("Error: Unit '{}' already exists at {}", unit_name, new_unit_path.display());
        // Returning Ok here as the command itself didn't fail, the condition was met.
        // Alternatively, could define a specific error for this.
        return Ok(());
    }

    fs::create_dir_all(&new_unit_path)
        .context(format!("Failed to create unit directory at {}", new_unit_path.display()))?;
    println!("Created unit directory: {}", new_unit_path.display());

    // Create unit.toml
    let unit_toml_path = new_unit_path.join("unit.toml");
    let unit_toml_content = format!(
        r#"# Configuration for the '{}' unit
name = "{}"
# templates_dir = "templates" # Default is "templates"
# target_dir = "~/.config/your_app" # Example: Specify where this unit's files should go

[variables]
# example_var = "example_value"
"#,
        unit_name, unit_name
    );
    fs::write(&unit_toml_path, unit_toml_content)
        .context(format!("Failed to create {}", unit_toml_path.display()))?;
    println!("Created unit manifest: {}", unit_toml_path.display());

    // Create templates directory
    let templates_dir_path = new_unit_path.join("templates");
    fs::create_dir_all(&templates_dir_path)
        .context(format!("Failed to create templates directory at {}", templates_dir_path.display()))?;
    println!("Created templates directory: {}", templates_dir_path.display());

    // Create an example template file
    let example_template_name = format!("{}.conf.template", unit_name);
    let example_template_path = templates_dir_path.join(&example_template_name);
    let example_template_content = format!(
        r#"# Example template for {} unit
# Configured with: {{ example_var | default(value="default_if_not_set") }}
"#,
        unit_name
    );
    fs::write(&example_template_path, example_template_content)
        .context(format!("Failed to create example template {}", example_template_path.display()))?;
    println!("Created example template: {}", example_template_path.display());

    println!("\nSuccessfully created unit '{}' at {}", unit_name, new_unit_path.display());
    println!("Next steps:");
    println!("1. Customize the unit manifest: {}", unit_toml_path.display());
    println!("2. Add your templates to: {}", templates_dir_path.display());
    println!("3. Add the unit name \"{}\" to the 'units' array in 'dotfiles.toml'.", unit_name);

    Ok(())
}
