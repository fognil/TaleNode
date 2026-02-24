use serde::{Deserialize, Serialize};

use super::graph::DialogueGraph;

/// The full project file format (.talenode).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub version: String,
    pub name: String,
    pub graph: DialogueGraph,
}

impl Default for Project {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            name: "Untitled".to_string(),
            graph: DialogueGraph::new(),
        }
    }
}

impl Project {
    pub fn save_to_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn load_from_string(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::node::Node;

    #[test]
    fn save_load_roundtrip() {
        let mut project = Project::default();
        project.name = "Test Project".to_string();
        project.graph.add_node(Node::new_start([100.0, 200.0]));
        project.graph.add_node(Node::new_dialogue([300.0, 200.0]));

        let json = project.save_to_string().unwrap();
        let loaded = Project::load_from_string(&json).unwrap();

        assert_eq!(loaded.name, "Test Project");
        assert_eq!(loaded.graph.nodes.len(), 2);
    }
}
