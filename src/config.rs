//! Configuration management.
//!
//! Handles loading and saving the application settings (physics and hotkeys)
//! from/to a `config.json` file.

use crate::physics::PhysicsConfig;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Root configuration structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Physics parameters (acceleration, friction, top speed).
    pub physics: PhysicsConfig,
    /// Global hotkey configuration.
    pub hotkey: HotkeyConfig,
}

/// Structure for defining the activation hotkey.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    /// Modifier keys (e.g., MOD_CONTROL, MOD_ALT).
    pub modifiers: u32,
    /// Virtual key code (e.g., VK_F10).
    pub vk: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            physics: PhysicsConfig::default(),
            hotkey: HotkeyConfig {
                // Default to Ctrl + Alt + F10
                modifiers: 0x0002 | 0x0001, // MOD_CONTROL | MOD_ALT
                vk: 0x79,                   // F10
            },
        }
    }
}

impl Config {
    /// Loads the configuration from `config.json`.
    ///
    /// If the file does not exist, it creates it with default values.
    /// If parsing fails, it returns the default configuration.
    pub fn load() -> Self {
        let path = Path::new("config.json");
        if path.exists() {
            match fs::read_to_string(path) {
                Ok(content) => match serde_json::from_str::<Self>(&content) {
                    Ok(config) => return config,
                    Err(e) => eprintln!("Error parsing config.json: {:?}. Using defaults.", e),
                },
                Err(e) => eprintln!("Error reading config.json: {:?}. Using defaults.", e),
            }
        }

        // If file missing or error occurs, fallback to defaults and save them.
        let default_config = Self::default();
        if let Err(e) = default_config.save() {
            eprintln!("Error saving default config: {:?}", e);
        }
        default_config
    }

    /// Persists the current configuration to `config.json`.
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write("config.json", content)?;
        Ok(())
    }
}
