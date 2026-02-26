use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::theme::ThemeConfig;
use crate::integrations::ai_writing::{self, AIProvider};

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
    #[serde(default)]
    pub theme: ThemeConfig,
    #[serde(default)]
    pub ai_provider: AIProvider,
    #[serde(default)]
    pub ai_api_key: Option<String>,
    #[serde(default = "ai_writing::default_ai_base_url")]
    pub ai_base_url: String,
    #[serde(default = "ai_writing::default_ai_model")]
    pub ai_model: String,
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
            theme: ThemeConfig::default(),
            ai_provider: AIProvider::default(),
            ai_api_key: None,
            ai_base_url: ai_writing::default_ai_base_url(),
            ai_model: ai_writing::default_ai_model(),
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

/// Action requested by the settings window.
pub enum SettingsAction {
    FetchModels,
    ProviderChanged,
}

/// Draw the settings window. Returns an optional action for the caller to handle.
pub fn show_settings_window(
    ctx: &egui::Context,
    settings: &mut AppSettings,
    open: &mut bool,
    available_models: &[String],
    models_loading: bool,
) -> Option<SettingsAction> {
    let mut action = None;
    egui::Window::new("Settings")
        .open(open)
        .resizable(true)
        .default_width(420.0)
        .show(ctx, |ui| {
            egui::CollapsingHeader::new("Appearance")
                .default_open(true)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Theme:");
                        egui::ComboBox::from_id_salt("theme_preset")
                            .selected_text(settings.theme.preset.label())
                            .show_ui(ui, |ui| {
                                for preset in super::theme::ThemePreset::ALL {
                                    ui.selectable_value(
                                        &mut settings.theme.preset,
                                        preset,
                                        preset.label(),
                                    );
                                }
                            });
                    });
                    ui.horizontal(|ui| {
                        ui.label("Font Size:");
                        let mut size = settings.theme.font_size as i32;
                        if ui
                            .add(egui::Slider::new(&mut size, 10..=24))
                            .changed()
                        {
                            settings.theme.font_size = size as u16;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Accent Color:");
                        ui.color_edit_button_srgb(&mut settings.theme.accent_color);
                    });
                });

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

            egui::CollapsingHeader::new("AI Writing Assistant")
                .default_open(true)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Provider:");
                        let prev = settings.ai_provider;
                        egui::ComboBox::from_id_salt("ai_provider")
                            .selected_text(settings.ai_provider.label())
                            .show_ui(ui, |ui| {
                                for provider in AIProvider::ALL {
                                    ui.selectable_value(
                                        &mut settings.ai_provider,
                                        provider,
                                        provider.label(),
                                    );
                                }
                            });
                        if settings.ai_provider != prev {
                            action = Some(SettingsAction::ProviderChanged);
                            match settings.ai_provider {
                                AIProvider::OpenAI => {
                                    settings.ai_base_url = ai_writing::default_ai_base_url();
                                    settings.ai_model = ai_writing::default_ai_model();
                                }
                                AIProvider::Anthropic => {
                                    settings.ai_base_url = String::new();
                                    settings.ai_model = "claude-sonnet-4-6".to_string();
                                }
                                AIProvider::Gemini => {
                                    settings.ai_base_url = ai_writing::default_gemini_base_url();
                                    settings.ai_model = ai_writing::default_gemini_model();
                                }
                            }
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("API Key:");
                        let mut key = settings.ai_api_key.clone().unwrap_or_default();
                        if ui.add(
                            egui::TextEdit::singleline(&mut key)
                                .password(true)
                                .desired_width(250.0),
                        ).changed() {
                            settings.ai_api_key =
                                if key.is_empty() { None } else { Some(key) };
                        }
                    });
                    if matches!(settings.ai_provider, AIProvider::OpenAI | AIProvider::Gemini) {
                        ui.horizontal(|ui| {
                            ui.label("Base URL:");
                            ui.text_edit_singleline(&mut settings.ai_base_url);
                        });
                    }
                    ui.horizontal(|ui| {
                        ui.label("Model:");
                        if available_models.is_empty() {
                            ui.text_edit_singleline(&mut settings.ai_model);
                        } else {
                            egui::ComboBox::from_id_salt("ai_model_select")
                                .selected_text(&settings.ai_model)
                                .show_ui(ui, |ui| {
                                    for model in available_models {
                                        ui.selectable_value(
                                            &mut settings.ai_model,
                                            model.clone(),
                                            model,
                                        );
                                    }
                                });
                        }
                    });
                    ui.horizontal(|ui| {
                        let has_key = settings
                            .ai_api_key
                            .as_ref()
                            .is_some_and(|k| !k.is_empty());
                        let enabled = has_key && !models_loading;
                        if ui
                            .add_enabled(enabled, egui::Button::new("Fetch Models"))
                            .clicked()
                        {
                            action = Some(SettingsAction::FetchModels);
                        }
                        if models_loading {
                            ui.spinner();
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
    action
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
        assert_eq!(s.ai_provider, AIProvider::OpenAI);
        assert!(s.ai_api_key.is_none());
        assert_eq!(s.ai_base_url, "https://api.openai.com/v1");
        assert_eq!(s.ai_model, "gpt-4o");
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

    #[test]
    fn write_and_read_settings_file() {
        let dir = std::env::temp_dir().join("talenode_test_settings");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("settings.json");

        let mut s = AppSettings::default();
        s.deepl_api_key = Some("dk_test_123".to_string());
        s.elevenlabs_api_key = Some("el_test_456".to_string());
        s.collab_username = "TestUser".to_string();
        s.collab_default_port = 8080;

        let json = serde_json::to_string_pretty(&s).unwrap();
        std::fs::write(&path, &json).unwrap();

        let data = std::fs::read_to_string(&path).unwrap();
        let loaded: AppSettings = serde_json::from_str(&data).unwrap();
        assert_eq!(loaded.deepl_api_key.as_deref(), Some("dk_test_123"));
        assert_eq!(loaded.elevenlabs_api_key.as_deref(), Some("el_test_456"));
        assert_eq!(loaded.collab_username, "TestUser");
        assert_eq!(loaded.collab_default_port, 8080);
        assert!(!loaded.deepl_use_pro);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn all_fields_survive_roundtrip() {
        let s = AppSettings {
            deepl_api_key: Some("key1".to_string()),
            deepl_use_pro: true,
            elevenlabs_api_key: Some("key2".to_string()),
            collab_username: "Bob".to_string(),
            collab_default_port: 12345,
            theme: ThemeConfig::default(),
            ai_provider: AIProvider::Anthropic,
            ai_api_key: Some("sk-ant-test".to_string()),
            ai_base_url: "https://custom.api.com/v1".to_string(),
            ai_model: "claude-sonnet-4-20250514".to_string(),
        };
        let json = serde_json::to_string(&s).unwrap();
        let loaded: AppSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.deepl_api_key, s.deepl_api_key);
        assert_eq!(loaded.deepl_use_pro, true);
        assert_eq!(loaded.elevenlabs_api_key, s.elevenlabs_api_key);
        assert_eq!(loaded.collab_username, "Bob");
        assert_eq!(loaded.collab_default_port, 12345);
        assert_eq!(loaded.ai_provider, AIProvider::Anthropic);
        assert_eq!(loaded.ai_api_key.as_deref(), Some("sk-ant-test"));
        assert_eq!(loaded.ai_model, "claude-sonnet-4-20250514");

        // Verify Gemini variant roundtrips
        let mut s2 = AppSettings::default();
        s2.ai_provider = AIProvider::Gemini;
        s2.ai_base_url = ai_writing::default_gemini_base_url();
        s2.ai_model = ai_writing::default_gemini_model();
        let json2 = serde_json::to_string(&s2).unwrap();
        let loaded2: AppSettings = serde_json::from_str(&json2).unwrap();
        assert_eq!(loaded2.ai_provider, AIProvider::Gemini);
        assert_eq!(loaded2.ai_model, "gemini-2.0-flash");
    }
}
