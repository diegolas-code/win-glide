//! Configuration management.
//!
//! Handles loading and saving the application settings (physics and hotkeys)
//! from/to a `config.json` file.

use crate::physics::PhysicsConfig;
use serde::{Deserialize, Serialize};
use std::fs;

/// Root configuration structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Physics parameters (acceleration, friction, top speed).
    pub physics: PhysicsConfig,
    /// Resize speed in pixels per second.
    #[serde(default = "default_resize_speed")]
    pub resize_speed: f32,
    /// Global hotkey configuration.
    pub hotkey: HotkeyConfig,
    /// Global hotkey configuration for centering the window.
    pub center_hotkey: HotkeyConfig,
    /// Optional custom physics parameters for resizing.
    pub resize_physics: Option<PhysicsConfig>,
}

fn default_resize_speed() -> f32 {
    600.0
}

/// Structure for defining the activation hotkey.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    /// Modifier keys (e.g., MOD_CONTROL, MOD_ALT).
    pub modifiers: u32,
    /// Virtual key code (e.g., VK_F10).
    pub vk: u32,
}

impl std::fmt::Display for HotkeyConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut parts = Vec::new();
        // Check modifiers
        // MOD_WIN = 0x0008, MOD_CONTROL = 0x0002, MOD_SHIFT = 0x0004, MOD_ALT = 0x0001
        if (self.modifiers & 0x0008) != 0 {
            parts.push("Win");
        }
        if (self.modifiers & 0x0002) != 0 {
            parts.push("Ctrl");
        }
        if (self.modifiers & 0x0004) != 0 {
            parts.push("Shift");
        }
        if (self.modifiers & 0x0001) != 0 {
            parts.push("Alt");
        }

        let key_name = match self.vk {
            // Function keys
            0x70 => "F1".to_string(),
            0x71 => "F2".to_string(),
            0x72 => "F3".to_string(),
            0x73 => "F4".to_string(),
            0x74 => "F5".to_string(),
            0x75 => "F6".to_string(),
            0x76 => "F7".to_string(),
            0x77 => "F8".to_string(),
            0x78 => "F9".to_string(),
            0x79 => "F10".to_string(),
            0x7A => "F11".to_string(),
            0x7B => "F12".to_string(),
            // Alpha-numeric
            vk @ 0x30..=0x39 => ((vk as u8) as char).to_string(),
            vk @ 0x41..=0x5A => ((vk as u8) as char).to_string(),
            // Arrow keys
            0x25 => "Left".to_string(),
            0x26 => "Up".to_string(),
            0x27 => "Right".to_string(),
            0x28 => "Down".to_string(),
            // Other common keys
            0x08 => "Backspace".to_string(),
            0x09 => "Tab".to_string(),
            0x0D => "Enter".to_string(),
            0x1B => "Escape".to_string(),
            0x20 => "Space".to_string(),
            0x2E => "Delete".to_string(),
            // Fallback
            vk => format!("0x{:X}", vk),
        };

        parts.push(&key_name);
        write!(f, "{}", parts.join("+"))
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            physics: PhysicsConfig::default(),
            resize_speed: 600.0,
            hotkey: HotkeyConfig {
                // Default to Ctrl + Alt + F10
                modifiers: 0x0002 | 0x0001, // MOD_CONTROL | MOD_ALT
                vk: 0x79,                   // F10
            },
            center_hotkey: HotkeyConfig {
                // Default to Win + Alt + C
                modifiers: 0x0008 | 0x0001, // MOD_WIN | MOD_ALT
                vk: 0x43,                   // C
            },
            resize_physics: None,
        }
    }
}

fn get_config_path() -> std::path::PathBuf {
    std::env::current_exe()
        .map(|p| {
            p.parent()
                .map(|parent| parent.join("config.json"))
                .unwrap_or_else(|| std::path::PathBuf::from("config.json"))
        })
        .unwrap_or_else(|_| std::path::PathBuf::from("config.json"))
}

impl Config {
    /// Loads the configuration from `config.json`.
    ///
    /// If the file does not exist, it creates it with default values.
    /// If parsing fails, it returns the default configuration.
    pub fn load() -> Self {
        let path = get_config_path();
        if path.exists() {
            match fs::read_to_string(&path) {
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
        let path = get_config_path();
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_deserialization_with_center_hotkey() {
        let json_data = r#"{
            "physics": {
                "acceleration": 4000.0,
                "friction": 10.0,
                "thrust_friction": 0.5,
                "top_speed": 4000.0
            },
            "hotkey": {
                "modifiers": 3,
                "vk": 121
            },
            "center_hotkey": {
                "modifiers": 9,
                "vk": 67
            }
        }"#;
        let config: Result<Config, _> = serde_json::from_str(json_data);
        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(config.center_hotkey.modifiers, 9);
        assert_eq!(config.center_hotkey.vk, 67);
    }

    #[test]
    fn test_hotkey_config_display() {
        let hk = HotkeyConfig {
            modifiers: 0x0002 | 0x0001, // MOD_CONTROL | MOD_ALT
            vk: 0x79,                   // F10
        };
        assert_eq!(hk.to_string(), "Ctrl+Alt+F10");

        let hk_center = HotkeyConfig {
            modifiers: 0x0008 | 0x0001, // MOD_WIN | MOD_ALT
            vk: 0x43,                   // C
        };
        assert_eq!(hk_center.to_string(), "Win+Alt+C");
    }

    #[test]
    fn test_config_deserialization_with_resize_speed() {
        let json_data = r#"{
            "physics": {
                "acceleration": 4000.0,
                "friction": 10.0,
                "thrust_friction": 0.5,
                "top_speed": 4000.0
            },
            "resize_speed": 750.0,
            "hotkey": {
                "modifiers": 3,
                "vk": 121
            },
            "center_hotkey": {
                "modifiers": 9,
                "vk": 67
            }
        }"#;
        let config: Result<Config, _> = serde_json::from_str(json_data);
        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(config.resize_speed, 750.0);
    }

    #[test]
    fn test_config_deserialization_with_resize_physics() {
        let json_data = r#"{
            "physics": {
                "acceleration": 4000.0,
                "friction": 10.0,
                "thrust_friction": 0.5,
                "top_speed": 4000.0
            },
            "resize_speed": 600.0,
            "resize_physics": {
                "acceleration": 2000.0,
                "friction": 25.0,
                "thrust_friction": 0.3,
                "top_speed": 1200.0
            },
            "hotkey": {
                "modifiers": 3,
                "vk": 121
            },
            "center_hotkey": {
                "modifiers": 9,
                "vk": 67
            }
        }"#;
        let config: Result<Config, _> = serde_json::from_str(json_data);
        assert!(config.is_ok());
        let config = config.unwrap();
        assert!(config.resize_physics.is_some());
        let resize_phys = config.resize_physics.unwrap();
        assert_eq!(resize_phys.acceleration, 2000.0);
        assert_eq!(resize_phys.friction, 25.0);
        assert_eq!(resize_phys.thrust_friction, 0.3);
        assert_eq!(resize_phys.top_speed, 1200.0);
    }
}
