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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{GlobalConfig, UnitManifest};
    use tera::{Map, Value as TeraValue}; // Note: TeraValue is already Value, just aliasing for clarity if needed
    use toml::value::Value as TomlValue; // Explicitly TomlValue
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::tempdir;
    use std::path::PathBuf;

    #[test]
    fn toml_to_tera_value_conversion_string() {
        let toml_val = TomlValue::String("test".to_string());
        let tera_val = toml_to_tera_value(&toml_val).unwrap();
        assert_eq!(tera_val, TeraValue::String("test".to_string()));
    }

    #[test]
    fn toml_to_tera_value_conversion_table() {
        let mut toml_map = toml::map::Map::new();
        toml_map.insert("key".to_string(), TomlValue::String("value".to_string()));
        toml_map.insert("number".to_string(), TomlValue::Integer(123));
        let toml_val = TomlValue::Table(toml_map);

        let tera_val = toml_to_tera_value(&toml_val).unwrap();
        
        let mut expected_map = Map::new();
        expected_map.insert("key".to_string(), TeraValue::String("value".to_string()));
        expected_map.insert("number".to_string(), TeraValue::from(123));
        assert_eq!(tera_val, TeraValue::Object(expected_map));
    }

    #[test]
    fn create_template_context_merging() {
        let mut global_vars_map = toml::map::Map::new();
        global_vars_map.insert("global_var".to_string(), TomlValue::String("global_val".to_string()));
        global_vars_map.insert("shared_var".to_string(), TomlValue::String("global_shared".to_string()));
        
        let global_config = GlobalConfig {
            units_base_dir: None,
            units: vec![],
            global_variables: Some(TomlValue::Table(global_vars_map)),
        };

        let mut unit_vars_map = toml::map::Map::new();
        unit_vars_map.insert("unit_var".to_string(), TomlValue::String("unit_val".to_string()));
        unit_vars_map.insert("shared_var".to_string(), TomlValue::String("unit_shared_override".to_string()));
        
        let unit_manifest = UnitManifest {
            name: Some("test_unit".to_string()),
            path: None,
            templates_dir: None,
            target_dir: None,
            variables: Some(TomlValue::Table(unit_vars_map)),
        };

        let context = create_template_context(&unit_manifest, &global_config).unwrap();

        // Check unit variable (at root)
        assert_eq!(context.get("unit_var").unwrap().as_str().unwrap(), "unit_val");
        
        // Check unit variable overriding what would be a global if not namespaced (at root)
        assert_eq!(context.get("shared_var").unwrap().as_str().unwrap(), "unit_shared_override");

        // Check global variable (namespaced under "global_variables")
        let globals_in_context = context.get("global_variables").unwrap();
        assert_eq!(globals_in_context.get("global_var").unwrap().as_str().unwrap(), "global_val");
        assert_eq!(globals_in_context.get("shared_var").unwrap().as_str().unwrap(), "global_shared");
    }
    
    #[test]
    fn create_template_context_unit_vars_not_table() {
        let global_config = GlobalConfig { // Empty global config
            units_base_dir: None, units: vec![], global_variables: None,
        };
        let unit_manifest = UnitManifest {
            name: Some("test_unit".to_string()), path: None, templates_dir: None, target_dir: None,
            variables: Some(TomlValue::String("this is not a table".to_string())), // Invalid unit var type
        };

        let result = create_template_context(&unit_manifest, &global_config);
        assert!(matches!(result, Err(AppError::TemplateError(_))));
        if let Err(AppError::TemplateError(msg)) = result {
            assert!(msg.contains("Unit variables for template context must be a TOML table (map)"));
        }
    }


    #[test]
    fn discover_templates_basic() {
        let unit_dir = tempdir().unwrap();
        let templates_path = unit_dir.path().join("templates");
        fs::create_dir_all(&templates_path).unwrap();
        fs::create_dir_all(templates_path.join("subdir")).unwrap();

        File::create(templates_path.join("file1.template")).unwrap();
        File::create(templates_path.join("subdir").join("file2.template")).unwrap();
        File::create(templates_path.join("not_a_template.txt")).unwrap(); // Should be ignored

        let discovered = discover_templates(&templates_path).unwrap();
        
        assert_eq!(discovered.len(), 2);
        // Convert to Vec<String> for easier comparison, as order from WalkDir is not guaranteed
        let mut discovered_paths_str: Vec<String> = discovered.into_iter()
            .map(|p| p.to_str().unwrap().replace("\\", "/")) // Normalize slashes for Windows
            .collect();
        discovered_paths_str.sort(); // Sort for consistent order

        let mut expected_paths_str = vec![
            "file1.template".to_string(),
            "subdir/file2.template".to_string(),
        ];
        expected_paths_str.sort();
        
        assert_eq!(discovered_paths_str, expected_paths_str);
    }

    #[test]
    fn discover_templates_empty_dir() {
        let unit_dir = tempdir().unwrap();
        let templates_path = unit_dir.path().join("templates");
        fs::create_dir_all(&templates_path).unwrap();

        let discovered = discover_templates(&templates_path).unwrap();
        assert!(discovered.is_empty());
    }

    #[test]
    fn discover_templates_non_existent_dir() {
        let unit_dir = tempdir().unwrap();
        let templates_path = unit_dir.path().join("non_existent_templates");
        // Do not create the directory
        let discovered = discover_templates(&templates_path).unwrap();
        assert!(discovered.is_empty()); // Should return Ok with empty vec
    }
    
    #[test]
    fn discover_templates_path_is_file() {
        let unit_dir = tempdir().unwrap();
        let file_path = unit_dir.path().join("templates_file.txt");
        File::create(&file_path).unwrap();

        let result = discover_templates(&file_path);
        assert!(matches!(result, Err(AppError::InvalidPath(_))));
        if let Err(AppError::InvalidPath(msg)) = result {
            assert!(msg.contains("is not a directory"));
        }
    }

    #[test]
    fn create_tera_instance_and_render() {
        let unit_dir = tempdir().unwrap();
        let templates_path = unit_dir.path().join("unit_A").join("templates");
        fs::create_dir_all(&templates_path).unwrap();

        let mut template_file = File::create(templates_path.join("hello.txt.template")).unwrap();
        template_file.write_all(b"Hello {{ name }}! Globals: {{ global_variables.planet }}").unwrap();
        
        // Test create_tera_instance
        let tera = create_tera_instance(&templates_path, "unit_A").unwrap();

        // Test render_template
        let mut context = Context::new();
        context.insert("name", "World");
        
        // Create a dummy global_variables structure for the context
        let mut global_vars_map = Map::new();
        global_vars_map.insert("planet".to_string(), TeraValue::String("Earth".to_string()));
        context.insert("global_variables", &TeraValue::Object(global_vars_map));


        let rendered = render_template(&tera, "hello.txt.template", &context).unwrap();
        assert_eq!(rendered, "Hello World! Globals: Earth");
    }
    
    #[test]
    fn create_tera_instance_no_templates_found() {
        let unit_dir = tempdir().unwrap();
        let templates_path = unit_dir.path().join("empty_templates");
        fs::create_dir_all(&templates_path).unwrap();

        // Tera::new with a glob that finds no files is not an error for Tera itself.
        // It successfully creates an instance with no templates.
        let tera_result = create_tera_instance(&templates_path, "empty_unit");
        assert!(tera_result.is_ok());
        let tera = tera_result.unwrap();
        assert_eq!(tera.get_template_names().count(), 0);
    }

    #[test]
    fn render_template_not_found_in_instance() {
        let tera = Tera::default(); // Empty Tera instance
        let context = Context::new();
        let result = render_template(&tera, "non_existent.template", &context);
        assert!(matches!(result, Err(AppError::TemplateError(_))));
        if let Err(AppError::TemplateError(msg)) = result {
            // Message comes from Tera, may vary slightly but should indicate template not found
            assert!(msg.contains("Template 'non_existent.template' not found"));
        }
    }
}
