use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::connection::Connection;
use super::node::Node;

/// A reusable pattern of nodes and connections.
///
/// Positions are stored relative to the bounding box origin `[0, 0]` so the
/// template can be inserted at any canvas position.  Only connections whose
/// both endpoints are inside the template are stored.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeTemplate {
    pub id: Uuid,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub is_builtin: bool,
    pub nodes: BTreeMap<Uuid, Node>,
    pub connections: Vec<Connection>,
}

/// Persistent collection of templates.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TemplateLibrary {
    pub templates: Vec<NodeTemplate>,
}

impl NodeTemplate {
    #[cfg(test)]
    fn new(name: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            description: String::new(),
            is_builtin: false,
            nodes: BTreeMap::new(),
            connections: Vec::new(),
        }
    }

    /// Number of nodes in this template.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn template_ids_are_unique() {
        let a = NodeTemplate::new("A");
        let b = NodeTemplate::new("B");
        assert_ne!(a.id, b.id);
    }

    #[test]
    fn new_template_defaults() {
        let t = NodeTemplate::new("Test");
        assert_eq!(t.name, "Test");
        assert!(t.description.is_empty());
        assert!(!t.is_builtin);
        assert!(t.nodes.is_empty());
        assert!(t.connections.is_empty());
    }

    #[test]
    fn serialization_roundtrip() {
        let mut t = NodeTemplate::new("Demo");
        t.description = "A demo template".to_string();
        let node = crate::model::node::Node::new_start([10.0, 20.0]);
        t.nodes.insert(node.id, node);

        let json = serde_json::to_string(&t).unwrap();
        let loaded: NodeTemplate = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.id, t.id);
        assert_eq!(loaded.name, "Demo");
        assert_eq!(loaded.node_count(), 1);
    }

    #[test]
    fn library_serialization_roundtrip() {
        let mut lib = TemplateLibrary::default();
        lib.templates.push(NodeTemplate::new("T1"));
        lib.templates.push(NodeTemplate::new("T2"));

        let json = serde_json::to_string(&lib).unwrap();
        let loaded: TemplateLibrary = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.templates.len(), 2);
        assert_eq!(loaded.templates[0].name, "T1");
    }

    #[test]
    fn backward_compat_missing_optional_fields() {
        // Simulate old JSON without description/is_builtin
        let json = r#"{"id":"00000000-0000-0000-0000-000000000001","name":"Old","nodes":{},"connections":[]}"#;
        let loaded: NodeTemplate = serde_json::from_str(json).unwrap();
        assert_eq!(loaded.name, "Old");
        assert!(loaded.description.is_empty());
        assert!(!loaded.is_builtin);
    }
}
