use std::collections::HashSet;
use uuid::Uuid;

use super::graph::DialogueGraph;

/// Summary of differences between two graph snapshots.
#[derive(Debug, Clone, Default)]
pub struct GraphDiff {
    pub added_nodes: Vec<Uuid>,
    pub removed_nodes: Vec<Uuid>,
    pub modified_nodes: Vec<Uuid>,
    pub added_connections: usize,
    pub removed_connections: usize,
    pub added_variables: Vec<String>,
    pub removed_variables: Vec<String>,
    pub added_characters: Vec<String>,
    pub removed_characters: Vec<String>,
}

impl GraphDiff {
    pub fn is_empty(&self) -> bool {
        self.added_nodes.is_empty()
            && self.removed_nodes.is_empty()
            && self.modified_nodes.is_empty()
            && self.added_connections == 0
            && self.removed_connections == 0
            && self.added_variables.is_empty()
            && self.removed_variables.is_empty()
            && self.added_characters.is_empty()
            && self.removed_characters.is_empty()
    }
}

/// Compare two graphs and return a diff summary.
pub fn diff_graphs(a: &DialogueGraph, b: &DialogueGraph) -> GraphDiff {
    let mut diff = GraphDiff::default();

    // Node diff
    let a_ids: HashSet<Uuid> = a.nodes.keys().copied().collect();
    let b_ids: HashSet<Uuid> = b.nodes.keys().copied().collect();

    diff.added_nodes = b_ids.difference(&a_ids).copied().collect();
    diff.removed_nodes = a_ids.difference(&b_ids).copied().collect();

    for &id in a_ids.intersection(&b_ids) {
        if let (Some(na), Some(nb)) = (a.nodes.get(&id), b.nodes.get(&id)) {
            if nodes_differ(na, nb) {
                diff.modified_nodes.push(id);
            }
        }
    }

    // Connection diff — compare by (from_node, from_port, to_node, to_port) tuples
    let a_conns: HashSet<(Uuid, Uuid, Uuid, Uuid)> = a
        .connections
        .iter()
        .map(|c| (c.from_node, c.from_port.0, c.to_node, c.to_port.0))
        .collect();
    let b_conns: HashSet<(Uuid, Uuid, Uuid, Uuid)> = b
        .connections
        .iter()
        .map(|c| (c.from_node, c.from_port.0, c.to_node, c.to_port.0))
        .collect();
    diff.added_connections = b_conns.difference(&a_conns).count();
    diff.removed_connections = a_conns.difference(&b_conns).count();

    // Variable diff by name
    let a_vars: HashSet<&str> = a.variables.iter().map(|v| v.name.as_str()).collect();
    let b_vars: HashSet<&str> = b.variables.iter().map(|v| v.name.as_str()).collect();
    diff.added_variables = b_vars.difference(&a_vars).map(|s| s.to_string()).collect();
    diff.removed_variables = a_vars.difference(&b_vars).map(|s| s.to_string()).collect();

    // Character diff by name
    let a_chars: HashSet<&str> = a.characters.iter().map(|c| c.name.as_str()).collect();
    let b_chars: HashSet<&str> = b.characters.iter().map(|c| c.name.as_str()).collect();
    diff.added_characters = b_chars.difference(&a_chars).map(|s| s.to_string()).collect();
    diff.removed_characters = a_chars.difference(&b_chars).map(|s| s.to_string()).collect();

    diff
}

/// Check if two nodes differ in type or content (via serde round-trip comparison).
fn nodes_differ(a: &super::node::Node, b: &super::node::Node) -> bool {
    // Compare serialized node_type to detect any content change
    let a_json = serde_json::to_string(&a.node_type).unwrap_or_default();
    let b_json = serde_json::to_string(&b.node_type).unwrap_or_default();
    a_json != b_json
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::character::Character;
    use crate::model::node::Node;
    use crate::model::node_types::{DialogueData, NodeType};
    use crate::model::variable::Variable;

    #[test]
    fn identical_graphs_empty_diff() {
        let g = DialogueGraph::new();
        let diff = diff_graphs(&g, &g);
        assert!(diff.is_empty());
    }

    #[test]
    fn added_node_detected() {
        let a = DialogueGraph::new();
        let mut b = DialogueGraph::new();
        b.add_node(Node::new_start([0.0, 0.0]));
        let diff = diff_graphs(&a, &b);
        assert_eq!(diff.added_nodes.len(), 1);
        assert!(diff.removed_nodes.is_empty());
    }

    #[test]
    fn removed_node_detected() {
        let mut a = DialogueGraph::new();
        a.add_node(Node::new_start([0.0, 0.0]));
        let b = DialogueGraph::new();
        let diff = diff_graphs(&a, &b);
        assert!(diff.added_nodes.is_empty());
        assert_eq!(diff.removed_nodes.len(), 1);
    }

    #[test]
    fn modified_node_detected() {
        let mut a = DialogueGraph::new();
        let node = Node::new_dialogue([0.0, 0.0]);
        let id = node.id;
        a.add_node(node);

        let mut b = a.clone();
        if let Some(n) = b.nodes.get_mut(&id) {
            n.node_type = NodeType::Dialogue(DialogueData {
                text: "Changed text".to_string(),
                ..Default::default()
            });
        }
        let diff = diff_graphs(&a, &b);
        assert!(diff.added_nodes.is_empty());
        assert!(diff.removed_nodes.is_empty());
        assert_eq!(diff.modified_nodes.len(), 1);
        assert_eq!(diff.modified_nodes[0], id);
    }

    #[test]
    fn variable_and_character_diff() {
        let mut a = DialogueGraph::new();
        a.variables.push(Variable::new_bool("old_flag", false));
        a.characters.push(Character::new("OldChar"));

        let mut b = DialogueGraph::new();
        b.variables.push(Variable::new_bool("new_flag", true));
        b.characters.push(Character::new("NewChar"));

        let diff = diff_graphs(&a, &b);
        assert_eq!(diff.added_variables, vec!["new_flag"]);
        assert_eq!(diff.removed_variables, vec!["old_flag"]);
        assert_eq!(diff.added_characters, vec!["NewChar"]);
        assert_eq!(diff.removed_characters, vec!["OldChar"]);
    }

    #[test]
    fn connection_diff() {
        let mut a = DialogueGraph::new();
        let s = Node::new_start([0.0, 0.0]);
        let d = Node::new_dialogue([100.0, 0.0]);
        let s_out = s.outputs[0].id;
        let d_in = d.inputs[0].id;
        let (sid, did) = (s.id, d.id);
        a.add_node(s);
        a.add_node(d);
        a.add_connection(sid, s_out, did, d_in);

        let mut b = a.clone();
        // Remove the connection in b
        b.connections.clear();

        let diff = diff_graphs(&a, &b);
        assert_eq!(diff.removed_connections, 1);
        assert_eq!(diff.added_connections, 0);
    }
}
