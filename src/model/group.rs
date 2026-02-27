use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A visual group of nodes with a colored background rectangle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeGroup {
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    #[serde(default)]
    pub name: String,
    /// Node IDs belonging to this group.
    #[serde(default)]
    pub node_ids: Vec<Uuid>,
    /// Group background color as [r, g, b, a] (0-255).
    #[serde(default = "NodeGroup::default_color")]
    pub color: [u8; 4],
    /// Whether the group is collapsed (hides contained nodes).
    #[serde(default)]
    pub collapsed: bool,
}

impl NodeGroup {
    fn default_color() -> [u8; 4] {
        [100, 150, 255, 40]
    }
}

impl NodeGroup {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            node_ids: Vec::new(),
            color: [100, 150, 255, 40],
            collapsed: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_group_defaults() {
        let g = NodeGroup::new("Act 1");
        assert_eq!(g.name, "Act 1");
        assert!(g.node_ids.is_empty());
        assert_eq!(g.color, [100, 150, 255, 40]);
    }

    #[test]
    fn group_ids_are_unique() {
        let a = NodeGroup::new("A");
        let b = NodeGroup::new("B");
        assert_ne!(a.id, b.id);
    }

    #[test]
    fn group_serialization_roundtrip() {
        let mut g = NodeGroup::new("Quest");
        g.node_ids.push(Uuid::new_v4());
        g.node_ids.push(Uuid::new_v4());
        g.collapsed = true;
        let json = serde_json::to_string(&g).unwrap();
        let loaded: NodeGroup = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.id, g.id);
        assert_eq!(loaded.name, "Quest");
        assert_eq!(loaded.node_ids.len(), 2);
        assert!(loaded.collapsed);
    }

    #[test]
    fn collapsed_field_defaults_false_on_old_data() {
        let json = r#"{"id":"00000000-0000-0000-0000-000000000001","name":"Old","node_ids":[],"color":[100,150,255,40]}"#;
        let loaded: NodeGroup = serde_json::from_str(json).unwrap();
        assert!(!loaded.collapsed);
    }
}
