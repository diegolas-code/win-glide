use crate::physics::PhysicsConfig;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub physics: PhysicsConfig,
    pub hotkey: HotkeyConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    pub modifiers: u32,
    pub vk: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            physics: PhysicsConfig::default(),
            hotkey: HotkeyConfig {
                modifiers: 0x0002 | 0x0001, // MOD_CONTROL | MOD_ALT
                vk: 0x79, // F10
            },
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let path = Path::new("config.json");
        if path.exists() {
            match fs::read_to_string(path) {
                Ok(content) => {
                    match serde_json::from_str::<Self>(&content) {
                        Ok(config) => return config,
                        Err(e) => eprintln!("Error parsing config.json: {:?}. Using defaults.", e),
                    }
                }
                Err(e) => eprintln!("Error reading config.json: {:?}. Using defaults.", e),
            }
        }
        
        let default_config = Self::default();
        if let Err(e) = default_config.save() {
            eprintln!("Error saving default config: {:?}", e);
        }
        default_config
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write("config.json", content)?;
        Ok(())
    }
}
