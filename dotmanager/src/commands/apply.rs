use crate::{
    config::{GlobalConfig, UnitManifest},
    error::AppError,
    loader::{load_global_config, load_unit_manifest},
    templating::{create_tera_instance, create_template_context, discover_templates, render_template},
};
use anyhow::{Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn run_apply() -> Result<()> {
    println!("Applying configuration...");

    let config_path = Path::new("dotfiles.toml");
    let global_config: GlobalConfig = load_global_config(config_path)
        .map_err(|e| anyhow::anyhow!(e))
        .context(format!("Failed to load global config from {}", config_path.display()))?;
    println!("Loaded global config from {}", config_path.display());

    let root_dir = config_path.parent().unwrap_or_else(|| Path::new("."));
    let units_base_dir_name = global_config.units_base_dir();
    let units_root_path = root_dir.join(units_base_dir_name);
    println!("Units base directory: {}", units_root_path.display());

    for unit_name_or_path_str in &global_config.units {
        println!("\nProcessing unit: {}", unit_name_or_path_str);
        let unit_dir_path = units_root_path.join(unit_name_or_path_str);

        if !unit_dir_path.is_dir() {
            eprintln!("Warning: Unit directory '{}' not found. Skipping.", unit_dir_path.display());
            continue;
        }

        let unit_manifest: UnitManifest = match load_unit_manifest(&unit_dir_path) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("Error loading unit manifest for '{}': {}. Skipping.", unit_name_or_path_str, e);
                continue;
            }
        };
        println!("Loaded unit manifest for '{}'", unit_name_or_path_str);

        let templates_dir_str = unit_manifest.templates_dir();
        let unit_templates_path = unit_dir_path.join(templates_dir_str);

        if !unit_templates_path.exists() || !unit_templates_path.is_dir() {
            println!(
                "Info: Templates directory '{}' not found or not a directory for unit '{}'. No templates to process from this directory.",
                unit_templates_path.display(),
                unit_name_or_path_str
            );
            // Depending on whether a unit *must* have templates, this could be an error or just info.
            // If other file operations were supported (e.g. direct copy), we might continue here.
            // For now, if no templates dir, we skip template processing for this unit.
        }

        // Only proceed with template processing if the templates directory exists
        if unit_templates_path.is_dir() {
            let tera_instance = match create_tera_instance(&unit_templates_path, unit_name_or_path_str) {
                Ok(t) => t,
                Err(e) => {
                    eprintln!(
                        "Error creating Tera instance for unit '{}': {}. Skipping template processing.",
                        unit_name_or_path_str, e
                    );
                    continue;
                }
            };
            println!("Created Tera instance for unit '{}'", unit_name_or_path_str);

            let templates_to_render = match discover_templates(&unit_templates_path) {
                Ok(t) => t,
                Err(e) => {
                    eprintln!(
                        "Error discovering templates for unit '{}': {}. Skipping template processing.",
                        unit_name_or_path_str, e
                    );
                    continue;
                }
            };

            if templates_to_render.is_empty() {
                println!("No templates found in '{}' for unit '{}'.", unit_templates_path.display(), unit_name_or_path_str);
            } else {
                 println!("Found {} template(s) for unit '{}'", templates_to_render.len(), unit_name_or_path_str);
            }


            let template_context = match create_template_context(&unit_manifest, &global_config) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!(
                        "Error creating template context for unit '{}': {}. Skipping template processing.",
                        unit_name_or_path_str, e
                    );
                    continue;
                }
            };
            println!("Created template context for unit '{}'", unit_name_or_path_str);

            let target_dir_base_str = match unit_manifest.target_dir.as_deref() {
                Some(td) => td,
                None => {
                    eprintln!(
                        "Error: 'target_dir' not specified in unit manifest for '{}'. Cannot apply templates. Skipping.",
                        unit_name_or_path_str
                    );
                    continue;
                }
            };

            let expanded_target_base_str = match shellexpand::tilde(target_dir_base_str) {
                Ok(expanded) => expanded.into_owned(),
                Err(e) => {
                    eprintln!(
                        "Error expanding tilde in target_dir '{}' for unit '{}': {}. Skipping.",
                        target_dir_base_str, unit_name_or_path_str, e
                    );
                    continue;
                }
            };
            let expanded_target_base_path = PathBuf::from(expanded_target_base_str);

            for relative_template_path in templates_to_render {
                let template_name_for_tera = match relative_template_path.to_str() {
                    Some(s) => s,
                    None => {
                        eprintln!(
                            "Error: Invalid (non-UTF8) template path '{}' in unit '{}'. Skipping this template.",
                            relative_template_path.display(),
                            unit_name_or_path_str
                        );
                        continue;
                    }
                };
                
                // Remove .template extension for the target file
                let target_file_name = relative_template_path.with_extension("");
                let target_file_path = expanded_target_base_path.join(target_file_name);

                println!("Rendering template '{}' for unit '{}'", template_name_for_tera, unit_name_or_path_str);

                let rendered_content = match render_template(&tera_instance, template_name_for_tera, &template_context) {
                    Ok(content) => content,
                    Err(e) => {
                        eprintln!(
                            "Error rendering template '{}' for unit '{}': {}. Skipping this template.",
                            template_name_for_tera, unit_name_or_path_str, e
                        );
                        continue;
                    }
                };

                if let Some(parent_dir) = target_file_path.parent() {
                    fs::create_dir_all(parent_dir).context(format!(
                        "Failed to create parent directories for {}",
                        target_file_path.display()
                    ))?;
                }

                fs::write(&target_file_path, rendered_content).context(format!(
                    "Failed to write rendered template to {}",
                    target_file_path.display()
                ))?;

                println!(
                    "Applied template '{}' to '{}'",
                    relative_template_path.display(),
                    target_file_path.display()
                );
            }
        }
    }

    println!("\nApply command finished.");
    Ok(())
}
