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
