use serde::{Deserialize, Serialize};

/// Theme preset selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ThemePreset {
    #[default]
    Dark,
    Light,
    Custom,
}

impl ThemePreset {
    pub const ALL: [ThemePreset; 3] = [Self::Dark, Self::Light, Self::Custom];

    pub fn label(self) -> &'static str {
        match self {
            Self::Dark => "Dark",
            Self::Light => "Light",
            Self::Custom => "Custom",
        }
    }

    /// Cycle to the next preset.
    pub fn next(self) -> Self {
        match self {
            Self::Dark => Self::Light,
            Self::Light => Self::Custom,
            Self::Custom => Self::Dark,
        }
    }
}

/// User-configurable theme settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    #[serde(default)]
    pub preset: ThemePreset,
    #[serde(default = "default_font_size")]
    pub font_size: u16,
    #[serde(default = "default_accent_color")]
    pub accent_color: [u8; 3],
}

fn default_font_size() -> u16 {
    14
}

fn default_accent_color() -> [u8; 3] {
    [100, 160, 255]
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            preset: ThemePreset::Dark,
            font_size: default_font_size(),
            accent_color: default_accent_color(),
        }
    }
}

/// Build egui Visuals from the theme config.
pub fn build_visuals(config: &ThemeConfig) -> egui::Visuals {
    match config.preset {
        ThemePreset::Dark => {
            let mut v = egui::Visuals::dark();
            apply_accent(&mut v, config.accent_color);
            v
        }
        ThemePreset::Light => {
            let mut v = egui::Visuals::light();
            apply_accent(&mut v, config.accent_color);
            v
        }
        ThemePreset::Custom => {
            let mut v = egui::Visuals::dark();
            v.panel_fill = egui::Color32::from_rgb(30, 30, 40);
            v.window_fill = egui::Color32::from_rgb(35, 35, 45);
            apply_accent(&mut v, config.accent_color);
            v
        }
    }
}

/// Apply font size to the egui context.
pub fn apply_font_size(ctx: &egui::Context, size: u16) {
    let size_f32 = size.clamp(10, 24) as f32;
    ctx.style_mut(|style| {
        for (_text_style, font_id) in style.text_styles.iter_mut() {
            font_id.size = size_f32;
        }
    });
}

fn apply_accent(visuals: &mut egui::Visuals, accent: [u8; 3]) {
    let color = egui::Color32::from_rgb(accent[0], accent[1], accent[2]);
    visuals.selection.bg_fill = color;
    visuals.hyperlink_color = color;
    visuals.widgets.hovered.bg_fill = color.linear_multiply(0.15);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_config_default() {
        let config = ThemeConfig::default();
        assert_eq!(config.preset, ThemePreset::Dark);
        assert_eq!(config.font_size, 14);
        assert_eq!(config.accent_color, [100, 160, 255]);
    }

    #[test]
    fn preset_cycle() {
        assert_eq!(ThemePreset::Dark.next(), ThemePreset::Light);
        assert_eq!(ThemePreset::Light.next(), ThemePreset::Custom);
        assert_eq!(ThemePreset::Custom.next(), ThemePreset::Dark);
    }

    #[test]
    fn theme_config_roundtrip() {
        let config = ThemeConfig {
            preset: ThemePreset::Light,
            font_size: 18,
            accent_color: [255, 100, 50],
        };
        let json = serde_json::to_string(&config).unwrap();
        let loaded: ThemeConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.preset, ThemePreset::Light);
        assert_eq!(loaded.font_size, 18);
        assert_eq!(loaded.accent_color, [255, 100, 50]);
    }

    #[test]
    fn backward_compat_missing_theme() {
        let loaded: ThemeConfig = serde_json::from_str("{}").unwrap();
        assert_eq!(loaded.preset, ThemePreset::Dark);
        assert_eq!(loaded.font_size, 14);
    }
}
