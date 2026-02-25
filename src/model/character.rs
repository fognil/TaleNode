use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    pub id: Uuid,
    pub name: String,
    /// Display color as [r, g, b, a] (0-255).
    #[serde(default = "default_color")]
    pub color: [u8; 4],
    #[serde(default)]
    pub portrait_path: String,
    #[serde(default)]
    pub voice_id: Option<String>,
}

fn default_color() -> [u8; 4] {
    [255, 255, 255, 255]
}

impl Character {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            color: [74, 144, 217, 255], // default blue
            portrait_path: String::new(),
            voice_id: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_character_defaults() {
        let c = Character::new("Guard");
        assert_eq!(c.name, "Guard");
        assert_eq!(c.color, [74, 144, 217, 255]);
        assert!(c.portrait_path.is_empty());
    }

    #[test]
    fn character_ids_are_unique() {
        let a = Character::new("A");
        let b = Character::new("B");
        assert_ne!(a.id, b.id);
    }

    #[test]
    fn character_serialization_roundtrip() {
        let c = Character::new("Princess");
        let json = serde_json::to_string(&c).unwrap();
        let loaded: Character = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.id, c.id);
        assert_eq!(loaded.name, "Princess");
        assert_eq!(loaded.color, [74, 144, 217, 255]);
    }

    #[test]
    fn deserialize_missing_color_uses_default() {
        let json = r#"{"id":"00000000-0000-0000-0000-000000000001","name":"Test"}"#;
        let loaded: Character = serde_json::from_str(json).unwrap();
        assert_eq!(loaded.color, [255, 255, 255, 255]); // serde default
        assert!(loaded.portrait_path.is_empty());
    }
}
