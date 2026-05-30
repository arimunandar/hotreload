use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub project: ProjectConfig,
    pub injection: InjectionConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectConfig {
    /// Path to the .xcodeproj or .xcworkspace
    pub project_path: PathBuf,
    /// Active scheme
    pub scheme: String,
    /// Swift module name (must match the app's module for symbol interposition)
    pub module_name: Option<String>,
    /// DerivedData directory for the project
    pub derived_data_path: Option<PathBuf>,
    /// Target platform (e.g., "arm64-apple-ios-simulator")
    pub target: Option<String>,
    /// SDK path override (auto-detected if empty)
    pub sdk_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InjectionConfig {
    /// Port for the dylib HTTP server on the host
    pub http_port: u16,
    /// TCP port on the iOS app for injection commands
    pub app_port: u16,
    /// Simulator host IP
    pub app_host: String,
    /// Directories/patterns to watch for changes
    pub watch_paths: Vec<String>,
    /// File extensions to watch
    pub watch_extensions: Vec<String>,
    /// Debounce delay in milliseconds
    pub debounce_ms: u64,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            project: ProjectConfig {
                project_path: PathBuf::new(),
                scheme: String::new(),
                module_name: None,
                derived_data_path: None,
                target: None,
                sdk_path: None,
            },
            injection: InjectionConfig {
                http_port: 9876,
                app_port: 8899,
                app_host: "127.0.0.1".to_string(),
                watch_paths: vec!["Sources".to_string()],
                watch_extensions: vec!["swift".to_string()],
                debounce_ms: 150,
            },
        }
    }
}

impl Config {
    pub fn config_dir(project_root: &Path) -> PathBuf {
        project_root.join(".hotreload")
    }

    pub fn config_path(project_root: &Path) -> PathBuf {
        Self::config_dir(project_root).join("config.toml")
    }

    /// Load config from project root's .hotreload/config.toml
    pub fn load(project_root: &Path) -> anyhow::Result<Self> {
        let path = Self::config_path(project_root);
        let content = std::fs::read_to_string(&path).map_err(|e| {
            anyhow::anyhow!(
                "Failed to read config at {}: {}",
                path.display(),
                e
            )
        })?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Save config to project root's .hotreload/config.toml
    pub fn save(&self, project_root: &Path) -> anyhow::Result<()> {
        let dir = Self::config_dir(project_root);
        std::fs::create_dir_all(&dir)?;
        let path = Self::config_path(project_root);
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        tracing::info!("Config written to {}", path.display());
        Ok(())
    }
}
