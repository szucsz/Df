use crate::config::{GlobalConfig, UnitManifest};
use crate::error::AppError;
use std::{fs, path::{Path, PathBuf}};
use toml;

pub fn load_global_config(config_path: &Path) -> Result<GlobalConfig, AppError> {
    let content = fs::read_to_string(config_path)
        .map_err(|e| AppError::ConfigLoadError(format!("Failed to read global config file {}: {}", config_path.display(), e)))?;
    toml::from_str(&content)
        .map_err(|e| AppError::ConfigLoadError(format!("Failed to parse global config file {}: {}", config_path.display(), e)))
}

pub fn load_unit_manifest(unit_path: &Path) -> Result<UnitManifest, AppError> {
    let manifest_path = unit_path.join("unit.toml");
    let content = fs::read_to_string(&manifest_path)
        .map_err(|e| AppError::ConfigLoadError(format!("Failed to read unit manifest file {}: {}", manifest_path.display(), e)))?;
    toml::from_str(&content)
        .map_err(|e| AppError::ConfigLoadError(format!("Failed to parse unit manifest file {}: {}", manifest_path.display(), e)))
}

pub fn resolve_path(base_dir: &Path, relative_path: &str) -> PathBuf {
    // For now, a simple join. Shell expansion can be added later if needed.
    base_dir.join(relative_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{GlobalConfig, UnitManifest};
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn load_global_config_success() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("dotfiles.toml");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(
            b"[global_variables]\ngreeting = \"hello\"\nunits = [\"unit1\"]"
        ).unwrap();

        let config = load_global_config(&file_path).unwrap();
        assert_eq!(config.units, vec!["unit1"]);
        assert_eq!(
            config.global_variables.as_ref().unwrap().get("greeting").unwrap().as_str().unwrap(),
            "hello"
        );
    }

    #[test]
    fn load_global_config_file_not_found() {
        let result = load_global_config(Path::new("non_existent_dotfiles.toml"));
        assert!(matches!(result, Err(AppError::ConfigLoadError(_))));
        if let Err(AppError::ConfigLoadError(msg)) = result {
            assert!(msg.contains("Failed to read global config file"));
        }
    }

    #[test]
    fn load_global_config_malformed_toml() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("dotfiles.toml");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"this is not toml content").unwrap();

        let result = load_global_config(&file_path);
        assert!(matches!(result, Err(AppError::ConfigLoadError(_))));
        if let Err(AppError::ConfigLoadError(msg)) = result {
            assert!(msg.contains("Failed to parse global config file"));
        }
    }

    #[test]
    fn load_unit_manifest_success() {
        let dir = tempdir().unwrap(); // This is the unit directory
        let file_path = dir.path().join("unit.toml");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(
            b"name = \"test_unit\"\ntarget_dir = \"~/.config/test\"\n[variables]\nfeature = true"
        ).unwrap();

        let manifest = load_unit_manifest(dir.path()).unwrap();
        assert_eq!(manifest.name.unwrap(), "test_unit");
        assert_eq!(manifest.target_dir.unwrap(), "~/.config/test");
        assert_eq!(
            manifest.variables.as_ref().unwrap().get("feature").unwrap().as_bool().unwrap(),
            true
        );
    }
    
    #[test]
    fn load_unit_manifest_file_not_found() {
        let dir = tempdir().unwrap();
        // unit.toml does not exist in dir
        let result = load_unit_manifest(dir.path());
        assert!(matches!(result, Err(AppError::ConfigLoadError(_))));
         if let Err(AppError::ConfigLoadError(msg)) = result {
            assert!(msg.contains("Failed to read unit manifest file"));
        }
    }

    #[test]
    fn load_unit_manifest_malformed_toml() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("unit.toml");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"this is not toml content for unit").unwrap();
        
        let result = load_unit_manifest(dir.path());
        assert!(matches!(result, Err(AppError::ConfigLoadError(_))));
        if let Err(AppError::ConfigLoadError(msg)) = result {
            assert!(msg.contains("Failed to parse unit manifest file"));
        }
    }


    #[test]
    fn global_config_units_base_dir_default() {
        let config = GlobalConfig {
            units_base_dir: None,
            units: vec![],
            global_variables: None,
        };
        assert_eq!(config.units_base_dir(), "units");
    }

    #[test]
    fn unit_manifest_templates_dir_default() {
        let manifest = UnitManifest {
            name: None,
            path: None,
            templates_dir: None,
            target_dir: None,
            variables: None,
        };
        assert_eq!(manifest.templates_dir(), "templates");
    }

    #[test]
    fn resolve_path_simple_join() {
        let base = Path::new("/tmp/base");
        let relative = "foo/bar";
        assert_eq!(resolve_path(base, relative), PathBuf::from("/tmp/base/foo/bar"));
    }
}
