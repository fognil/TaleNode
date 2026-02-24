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
        }
    }
}
