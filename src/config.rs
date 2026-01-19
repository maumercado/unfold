//! Configuration file management for persistent settings.
//!
//! Stores user preferences in ~/.unfold/config.json

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::theme::AppTheme;

/// User configuration that persists between sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// User's preferred theme
    #[serde(default)]
    pub theme: AppTheme,
    /// Whether CLI tool has been installed
    #[serde(default)]
    pub cli_installed: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            theme: AppTheme::Dark,
            cli_installed: false,
        }
    }
}

impl Config {
    /// Get the config directory path (~/.unfold)
    pub fn config_dir() -> Option<PathBuf> {
        dirs::home_dir().map(|home| home.join(".unfold"))
    }

    /// Get the config file path (~/.unfold/config.json)
    pub fn config_path() -> Option<PathBuf> {
        Self::config_dir().map(|dir| dir.join("config.json"))
    }

    /// Load config from file, or return default if not found
    pub fn load() -> Self {
        let Some(path) = Self::config_path() else {
            return Config::default();
        };

        match fs::read_to_string(&path) {
            Ok(contents) => {
                serde_json::from_str(&contents).unwrap_or_default()
            }
            Err(_) => Config::default(),
        }
    }

    /// Save config to file
    pub fn save(&self) -> Result<(), String> {
        let dir = Self::config_dir()
            .ok_or_else(|| "Could not determine home directory".to_string())?;

        // Create config directory if it doesn't exist
        if !dir.exists() {
            fs::create_dir_all(&dir)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }

        let path = Self::config_path()
            .ok_or_else(|| "Could not determine config path".to_string())?;

        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        fs::write(&path, json)
            .map_err(|e| format!("Failed to write config: {}", e))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.theme, AppTheme::Dark);
        assert!(!config.cli_installed);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config {
            theme: AppTheme::Light,
            cli_installed: true,
        };

        let json = serde_json::to_string(&config).unwrap();
        let parsed: Config = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.theme, AppTheme::Light);
        assert!(parsed.cli_installed);
    }
}
