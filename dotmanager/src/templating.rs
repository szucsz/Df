use crate::{
    config::{GlobalConfig, UnitManifest},
    error::AppError,
};
use std::path::{Path, PathBuf};
use tera::{Context, Tera, Value};
use walkdir::WalkDir;
use anyhow::Context as AnyhowContext; // Alias to avoid conflict with tera::Context


// Helper function to convert toml::Value to tera::Value
fn toml_to_tera_value(toml_val: &toml::Value) -> Result<Value, AppError> {
    // Easiest way is often via JSON as an intermediary if direct conversion is complex
    let json_val = serde_json::to_value(toml_val).map_err(|e| {
        AppError::TemplateError(format!( // Changed from ConfigLoadError
            "Failed to convert TOML value to intermediate JSON for Tera context: {}",
            e
        ))
    })?;
    Value::try_from(json_val).map_err(|e| {
        AppError::TemplateError(format!("Failed to convert intermediate JSON to Tera Value for context: {}", e))
    })
}

pub fn create_tera_instance(
    unit_templates_path: &Path,
    _unit_name: &str, // unit_name might be used later for namespacing if needed
) -> Result<Tera, AppError> {
    let mut tera = Tera::default();
    let glob_path = unit_templates_path.join("**/*.template");

    let glob_str = glob_path.to_str().ok_or_else(|| {
        AppError::InvalidPath(format!(
            "Failed to convert path to string for Tera glob: {}",
            glob_path.display()
        ))
    })?;

    tera.add_template_files(glob_str)
        .map_err(|e| AppError::TemplateError(format!("Tera template loading error: {}", e)))?;
    Ok(tera)
}

pub fn create_template_context(
    unit_manifest: &UnitManifest,
    global_config: &GlobalConfig,
) -> Result<Context, AppError> {
    let mut context = Context::new();

    // Add global variables
    if let Some(ref global_vars_toml) = global_config.global_variables {
        let global_vars_tera = toml_to_tera_value(global_vars_toml)?;
        // It's generally better to insert global variables under a specific key,
        // e.g., "global", to avoid unintended clashes with unit variables.
        context.insert("global_variables", &global_vars_tera);
    }

    // Add unit-specific variables
    if let Some(ref unit_vars_toml) = unit_manifest.variables {
        let unit_vars_tera = toml_to_tera_value(unit_vars_toml)?;
        // If unit_vars_tera is a map, iterate and insert its keys at the root.
        // Otherwise, insert it under a key like "unit".
        // For now, let's assume unit_vars_toml is always a table/map at its root.
        if let Value::Object(map) = unit_vars_tera {
            for (key, value) in map {
                context.insert(&key, &value);
            }
        } else {
            // If it's not an object, it's a single value. Insert it under a specific key or error.
            // For simplicity, we'll assume unit variables are always a TOML table.
            // If it could be other TOML types, this needs more robust handling.
             return Err(AppError::TemplateError( // Changed from ConfigLoadError
                "Unit variables for template context must be a TOML table (map).".to_string()
            ));
        }
    }

    Ok(context)
}

pub fn render_template(
    tera: &Tera,
    template_name: &str,
    context: &Context,
) -> Result<String, AppError> {
    tera.render(template_name, context)
        .map_err(|e| AppError::TemplateError(format!("Tera rendering error for template '{}': {}", template_name, e)))
}

pub fn discover_templates(templates_dir: &Path) -> Result<Vec<PathBuf>, AppError> {
    let mut template_files = Vec::new();

    if !templates_dir.exists() {
        // It's not an error if the templates directory doesn't exist for a unit
        // Or it could be, depending on desired strictness. For now, return empty.
        return Ok(template_files);
    }
    if !templates_dir.is_dir() {
        return Err(AppError::InvalidPath(format!(
            "Templates path '{}' is not a directory.",
            templates_dir.display()
        )));
    }

    for entry in WalkDir::new(templates_dir)
        .into_iter()
        .filter_map(|e| e.ok()) // Filter out errors during walk
    {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "template" {
                    // We need the path relative to templates_dir for Tera to use it as a name
                    let relative_path = path.strip_prefix(templates_dir).map_err(|e| {
                        AppError::InvalidPath(format!(
                            "Failed to strip prefix from template path {}: {}",
                            path.display(),
                            e
                        ))
                    })?;
                    template_files.push(relative_path.to_path_buf());
                }
            }
        }
    }
    Ok(template_files)
}
