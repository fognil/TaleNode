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
        let json = serde_json::to_string(&g).unwrap();
        let loaded: NodeGroup = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.id, g.id);
        assert_eq!(loaded.name, "Quest");
        assert_eq!(loaded.node_ids.len(), 2);
    }
}
