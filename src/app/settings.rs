use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Persistent application settings stored at ~/.talenode/settings.json.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    #[serde(default)]
    pub deepl_api_key: Option<String>,
    #[serde(default)]
    pub deepl_use_pro: bool,
    #[serde(default)]
    pub elevenlabs_api_key: Option<String>,
    #[serde(default)]
    pub collab_username: String,
    #[serde(default = "default_collab_port")]
    pub collab_default_port: u16,
}

fn default_collab_port() -> u16 {
    9847
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            deepl_api_key: None,
            deepl_use_pro: false,
            elevenlabs_api_key: None,
            collab_username: whoami().unwrap_or_else(|| "User".to_string()),
            collab_default_port: default_collab_port(),
        }
    }
}

fn whoami() -> Option<String> {
    std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .ok()
}

fn settings_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("talenode").join("settings.json"))
}

impl AppSettings {
    /// Load settings from disk, returning defaults on any error.
    pub fn load() -> Self {
        let Some(path) = settings_path() else {
            return Self::default();
        };
        let Ok(data) = std::fs::read_to_string(&path) else {
            return Self::default();
        };
        serde_json::from_str(&data).unwrap_or_default()
    }

    /// Save settings to disk. Silently ignores errors.
    pub fn save(&self) {
        let Some(path) = settings_path() else {
            return;
        };
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(&path, json);
        }
    }
}

/// Draw the settings window. Returns `true` if the window is still open.
pub fn show_settings_window(
    ctx: &egui::Context,
    settings: &mut AppSettings,
    open: &mut bool,
) {
    egui::Window::new("Settings")
        .open(open)
        .resizable(true)
        .default_width(420.0)
        .show(ctx, |ui| {
            egui::CollapsingHeader::new("DeepL Translation")
                .default_open(true)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("API Key:");
                        let mut key = settings.deepl_api_key.clone().unwrap_or_default();
                        if ui.add(
                            egui::TextEdit::singleline(&mut key)
                                .password(true)
                                .desired_width(250.0),
                        ).changed() {
                            settings.deepl_api_key = if key.is_empty() { None } else { Some(key) };
                        }
                    });
                    ui.checkbox(&mut settings.deepl_use_pro, "Use DeepL Pro API");
                });

            egui::CollapsingHeader::new("ElevenLabs Voice")
                .default_open(true)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("API Key:");
                        let mut key = settings.elevenlabs_api_key.clone().unwrap_or_default();
                        if ui.add(
                            egui::TextEdit::singleline(&mut key)
                                .password(true)
                                .desired_width(250.0),
                        ).changed() {
                            settings.elevenlabs_api_key =
                                if key.is_empty() { None } else { Some(key) };
                        }
                    });
                });

            egui::CollapsingHeader::new("Collaboration")
                .default_open(true)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Username:");
                        ui.text_edit_singleline(&mut settings.collab_username);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Default Port:");
                        ui.add(
                            egui::DragValue::new(&mut settings.collab_default_port)
                                .range(1024..=65535),
                        );
                    });
                });

            ui.add_space(8.0);
            if ui.button("Save Settings").clicked() {
                settings.save();
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_settings() {
        let s = AppSettings::default();
        assert!(s.deepl_api_key.is_none());
        assert!(!s.deepl_use_pro);
        assert!(s.elevenlabs_api_key.is_none());
        assert_eq!(s.collab_default_port, 9847);
    }

    #[test]
    fn serialization_roundtrip() {
        let mut s = AppSettings::default();
        s.deepl_api_key = Some("test-key".to_string());
        s.collab_username = "Alice".to_string();
        let json = serde_json::to_string(&s).unwrap();
        let loaded: AppSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.deepl_api_key.as_deref(), Some("test-key"));
        assert_eq!(loaded.collab_username, "Alice");
        assert_eq!(loaded.collab_default_port, 9847);
    }

    #[test]
    fn deserialize_empty_json_uses_defaults() {
        let loaded: AppSettings = serde_json::from_str("{}").unwrap();
        assert!(loaded.deepl_api_key.is_none());
        assert_eq!(loaded.collab_default_port, 9847);
    }
}
