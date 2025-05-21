use crate::config::{GlobalConfig, UnitManifest}; // Assuming these are pub in config.rs
use std::{fs, path::Path};
use anyhow::Context;

const DOTFILES_TOML_CONTENT: &str = r#"# Global configuration for the dotfile manager
units_base_dir = "units" # Directory where all units (modules) are stored

# List of unit names to apply. These should correspond to directory names in 'units_base_dir'.
units = [
  "example"
]

# Global variables accessible to all templates
# [global_variables]
# username = "your_username"
# email = "your_email@example.com"
"#;

const EXAMPLE_UNIT_TOML_CONTENT: &str = r#"# Configuration for the 'example' unit
name = "example"
# templates_dir = "templates" # Relative to this unit's directory, defaults to "templates"
# target_dir = "~/.config"  # Base target directory for files from this unit

# Variables specific to this unit's templates
[variables]
feature_enabled = true
message = "Hello from the example unit!"
"#;

const EXAMPLE_TEMPLATE_CONTENT: &str = r#"This is an example configuration file.
It's managed by the dotfile manager.

User message: {{ message }}
Feature status: {% if feature_enabled %}Enabled{% else %}Disabled{% endif %}

{% if global_variables.username %}Global username: {{ global_variables.username }}{% endif %}
"#;

pub fn run_init() -> Result<(), anyhow::Error> {
    let root_dir = Path::new("."); // Assuming current directory

    // Create dotfiles.toml
    let dotfiles_toml_path = root_dir.join("dotfiles.toml");
    fs::write(&dotfiles_toml_path, DOTFILES_TOML_CONTENT)
        .context(format!("Failed to create {}", dotfiles_toml_path.display()))?;
    println!("Created {}", dotfiles_toml_path.display());

    // Parse dotfiles.toml to get units_base_dir (even though we hardcoded it for creation)
    // This is more for a conceptual step, in a real scenario you might deserialize it here
    // For now, we'll just use the hardcoded value "units"
    let units_base_dir_name = "units"; // from DOTFILES_TOML_CONTENT
    let units_base_path = root_dir.join(units_base_dir_name);

    // Create the units base directory
    fs::create_dir_all(&units_base_path)
        .context(format!("Failed to create directory {}", units_base_path.display()))?;
    println!("Created directory {}", units_base_path.display());

    // Create an example unit
    let example_unit_name = "example";
    let example_unit_path = units_base_path.join(example_unit_name);
    fs::create_dir_all(&example_unit_path)
        .context(format!("Failed to create directory {}", example_unit_path.display()))?;
    println!("Created directory {}", example_unit_path.display());

    // Create unit.toml for the example unit
    let unit_toml_path = example_unit_path.join("unit.toml");
    fs::write(&unit_toml_path, EXAMPLE_UNIT_TOML_CONTENT)
        .context(format!("Failed to create {}", unit_toml_path.display()))?;
    println!("Created {}", unit_toml_path.display());

    // Create templates directory for the example unit
    // let unit_manifest_example: UnitManifest = toml::from_str(EXAMPLE_UNIT_TOML_CONTENT)?;
    // let templates_dir_name = unit_manifest_example.templates_dir(); // using default "templates"
    let templates_dir_name = "templates"; // Default from UnitManifest
    let templates_path = example_unit_path.join(templates_dir_name);
    fs::create_dir_all(&templates_path)
        .context(format!("Failed to create directory {}", templates_path.display()))?;
    println!("Created directory {}", templates_path.display());

    // Create example template file
    let example_template_path = templates_path.join("example_config.txt.template");
    fs::write(&example_template_path, EXAMPLE_TEMPLATE_CONTENT)
        .context(format!("Failed to create {}", example_template_path.display()))?;
    println!("Created {}", example_template_path.display());

    println!("Initialized dotfiles repository.");
    Ok(())
}
