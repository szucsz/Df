use serde::Deserialize;
use toml;

#[derive(Debug, Deserialize)]
pub struct UnitManifest {
    pub name: Option<String>,
    pub path: Option<String>,
    pub templates_dir: Option<String>,
    pub target_dir: Option<String>,
    pub variables: Option<toml::Value>,
}

impl UnitManifest {
    pub fn templates_dir(&self) -> &str {
        self.templates_dir.as_deref().unwrap_or("templates")
    }
}

#[derive(Debug, Deserialize)]
pub struct GlobalConfig {
    pub units_base_dir: Option<String>,
    pub units: Vec<String>,
    pub global_variables: Option<toml::Value>,
}

impl GlobalConfig {
    pub fn units_base_dir(&self) -> &str {
        self.units_base_dir.as_deref().unwrap_or("units")
    }
}
