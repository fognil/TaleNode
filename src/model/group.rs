use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A visual group of nodes with a colored background rectangle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeGroup {
    pub id: Uuid,
    pub name: String,
    /// Node IDs belonging to this group.
    pub node_ids: Vec<Uuid>,
    /// Group background color as [r, g, b, a] (0-255).
    pub color: [u8; 4],
}

impl NodeGroup {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            node_ids: Vec::new(),
            color: [100, 150, 255, 40],
        }
    }
}
