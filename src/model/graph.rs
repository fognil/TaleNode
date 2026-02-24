use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::character::Character;
use super::connection::Connection;
use super::group::NodeGroup;
use super::node::Node;
use super::port::{PortDirection, PortId};
use super::review::{NodeComment, ReviewStatus};
use super::variable::Variable;

/// The central data structure holding the entire dialogue graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueGraph {
    #[serde(default)]
    pub nodes: HashMap<Uuid, Node>,
    #[serde(default)]
    pub connections: Vec<Connection>,
    #[serde(default)]
    pub variables: Vec<Variable>,
    #[serde(default)]
    pub characters: Vec<Character>,
    #[serde(default)]
    pub groups: Vec<NodeGroup>,
    #[serde(default)]
    pub review_statuses: HashMap<Uuid, ReviewStatus>,
    #[serde(default)]
    pub comments: Vec<NodeComment>,
}

impl Default for DialogueGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl DialogueGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            connections: Vec::new(),
            variables: Vec::new(),
            characters: Vec::new(),
            groups: Vec::new(),
            review_statuses: HashMap::new(),
            comments: Vec::new(),
        }
    }

    pub fn add_node(&mut self, node: Node) {
        self.nodes.insert(node.id, node);
    }

    pub fn remove_node(&mut self, node_id: Uuid) -> Option<(Node, Vec<Connection>)> {
        if let Some(node) = self.nodes.remove(&node_id) {
            // Remove all connections involving this node
            let removed: Vec<Connection> = self
                .connections
                .iter()
                .filter(|c| c.from_node == node_id || c.to_node == node_id)
                .cloned()
                .collect();
            self.connections
                .retain(|c| c.from_node != node_id && c.to_node != node_id);
            self.review_statuses.remove(&node_id);
            self.comments.retain(|c| c.node_id != node_id);
            Some((node, removed))
        } else {
            None
        }
    }

    /// Try to add a connection. Returns false if validation fails.
    pub fn add_connection(
        &mut self,
        from_node: Uuid,
        from_port: PortId,
        to_node: Uuid,
        to_port: PortId,
    ) -> bool {
        // No self-loops
        if from_node == to_node {
            return false;
        }

        // Validate ports exist and directions are correct
        let from_valid = self.nodes.get(&from_node).is_some_and(|n| {
            n.outputs
                .iter()
                .any(|p| p.id == from_port && p.direction == PortDirection::Output)
        });
        let to_valid = self.nodes.get(&to_node).is_some_and(|n| {
            n.inputs
                .iter()
                .any(|p| p.id == to_port && p.direction == PortDirection::Input)
        });

        if !from_valid || !to_valid {
            return false;
        }

        // Check if output port already has a connection
        if self.connections.iter().any(|c| c.from_port == from_port) {
            return false;
        }

        // Check if input port already has a connection
        if self.connections.iter().any(|c| c.to_port == to_port) {
            return false;
        }

        let conn = Connection::new(from_node, from_port, to_node, to_port);
        self.connections.push(conn);
        true
    }

    #[allow(dead_code)]
    pub fn remove_connection(&mut self, connection_id: Uuid) -> Option<Connection> {
        if let Some(idx) = self.connections.iter().position(|c| c.id == connection_id) {
            Some(self.connections.remove(idx))
        } else {
            None
        }
    }

    /// Get the review status for a node (defaults to Draft).
    pub fn get_review_status(&self, node_id: Uuid) -> ReviewStatus {
        self.review_statuses
            .get(&node_id)
            .copied()
            .unwrap_or_default()
    }

    /// Set the review status for a node. Removes the entry if Draft (the default).
    pub fn set_review_status(&mut self, node_id: Uuid, status: ReviewStatus) {
        if status == ReviewStatus::Draft {
            self.review_statuses.remove(&node_id);
        } else {
            self.review_statuses.insert(node_id, status);
        }
    }

    /// Find which node and port a given PortId belongs to.
    #[allow(dead_code)]
    pub fn find_port_node(&self, port_id: PortId) -> Option<(Uuid, PortDirection)> {
        for node in self.nodes.values() {
            for p in &node.inputs {
                if p.id == port_id {
                    return Some((node.id, PortDirection::Input));
                }
            }
            for p in &node.outputs {
                if p.id == port_id {
                    return Some((node.id, PortDirection::Output));
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::node::Node;

    #[test]
    fn add_and_remove_node() {
        let mut graph = DialogueGraph::new();
        let node = Node::new_start([0.0, 0.0]);
        let id = node.id;
        graph.add_node(node);
        assert!(graph.nodes.contains_key(&id));

        let (removed, _conns) = graph.remove_node(id).unwrap();
        assert_eq!(removed.id, id);
        assert!(!graph.nodes.contains_key(&id));
    }

    #[test]
    fn connection_validation() {
        let mut graph = DialogueGraph::new();

        let start = Node::new_start([0.0, 0.0]);
        let dlg = Node::new_dialogue([200.0, 0.0]);

        let start_out = start.outputs[0].id;
        let dlg_in = dlg.inputs[0].id;
        let start_id = start.id;
        let dlg_id = dlg.id;

        graph.add_node(start);
        graph.add_node(dlg);

        // Valid connection
        assert!(graph.add_connection(start_id, start_out, dlg_id, dlg_in));

        // Duplicate output port — should fail
        let dlg2 = Node::new_dialogue([400.0, 0.0]);
        let dlg2_in = dlg2.inputs[0].id;
        let dlg2_id = dlg2.id;
        graph.add_node(dlg2);
        assert!(!graph.add_connection(start_id, start_out, dlg2_id, dlg2_in));
    }

    #[test]
    fn no_self_loop() {
        let mut graph = DialogueGraph::new();
        let dlg = Node::new_dialogue([0.0, 0.0]);
        let id = dlg.id;
        let out = dlg.outputs[0].id;
        let inp = dlg.inputs[0].id;
        graph.add_node(dlg);

        assert!(!graph.add_connection(id, out, id, inp));
    }

    #[test]
    fn removing_node_removes_connections() {
        let mut graph = DialogueGraph::new();
        let start = Node::new_start([0.0, 0.0]);
        let dlg = Node::new_dialogue([200.0, 0.0]);

        let start_out = start.outputs[0].id;
        let dlg_in = dlg.inputs[0].id;
        let start_id = start.id;
        let dlg_id = dlg.id;

        graph.add_node(start);
        graph.add_node(dlg);
        graph.add_connection(start_id, start_out, dlg_id, dlg_in);

        assert_eq!(graph.connections.len(), 1);
        let (_node, removed_conns) = graph.remove_node(start_id).unwrap();
        assert_eq!(removed_conns.len(), 1);
        assert!(graph.connections.is_empty());
    }

    #[test]
    fn remove_nonexistent_node_returns_none() {
        let mut graph = DialogueGraph::new();
        assert!(graph.remove_node(Uuid::new_v4()).is_none());
    }

    #[test]
    fn remove_connection_basic() {
        let mut graph = DialogueGraph::new();
        let start = Node::new_start([0.0, 0.0]);
        let end = Node::new_end([200.0, 0.0]);
        let start_out = start.outputs[0].id;
        let end_in = end.inputs[0].id;
        let start_id = start.id;
        let end_id = end.id;
        graph.add_node(start);
        graph.add_node(end);
        graph.add_connection(start_id, start_out, end_id, end_in);

        let conn_id = graph.connections[0].id;
        let removed = graph.remove_connection(conn_id).unwrap();
        assert_eq!(removed.id, conn_id);
        assert!(graph.connections.is_empty());
    }

    #[test]
    fn remove_nonexistent_connection_returns_none() {
        let mut graph = DialogueGraph::new();
        assert!(graph.remove_connection(Uuid::new_v4()).is_none());
    }

    #[test]
    fn find_port_node_input() {
        let mut graph = DialogueGraph::new();
        let dlg = Node::new_dialogue([0.0, 0.0]);
        let dlg_id = dlg.id;
        let in_port = dlg.inputs[0].id;
        graph.add_node(dlg);

        let (node_id, dir) = graph.find_port_node(in_port).unwrap();
        assert_eq!(node_id, dlg_id);
        assert_eq!(dir, PortDirection::Input);
    }

    #[test]
    fn find_port_node_output() {
        let mut graph = DialogueGraph::new();
        let start = Node::new_start([0.0, 0.0]);
        let start_id = start.id;
        let out_port = start.outputs[0].id;
        graph.add_node(start);

        let (node_id, dir) = graph.find_port_node(out_port).unwrap();
        assert_eq!(node_id, start_id);
        assert_eq!(dir, PortDirection::Output);
    }

    #[test]
    fn find_port_node_nonexistent() {
        let graph = DialogueGraph::new();
        assert!(graph.find_port_node(PortId::new()).is_none());
    }

    #[test]
    fn duplicate_input_port_rejected() {
        let mut graph = DialogueGraph::new();
        let start1 = Node::new_start([0.0, 0.0]);
        let start2 = Node::new_start([0.0, 100.0]);
        let dlg = Node::new_dialogue([200.0, 0.0]);

        let s1_out = start1.outputs[0].id;
        let s2_out = start2.outputs[0].id;
        let dlg_in = dlg.inputs[0].id;
        let s1_id = start1.id;
        let s2_id = start2.id;
        let dlg_id = dlg.id;

        graph.add_node(start1);
        graph.add_node(start2);
        graph.add_node(dlg);

        // First connection succeeds
        assert!(graph.add_connection(s1_id, s1_out, dlg_id, dlg_in));
        // Same input port from different source — rejected
        assert!(!graph.add_connection(s2_id, s2_out, dlg_id, dlg_in));
    }

    #[test]
    fn invalid_port_ids_rejected() {
        let mut graph = DialogueGraph::new();
        let start = Node::new_start([0.0, 0.0]);
        let dlg = Node::new_dialogue([200.0, 0.0]);
        let start_id = start.id;
        let dlg_id = dlg.id;
        graph.add_node(start);
        graph.add_node(dlg);

        // Use fake port IDs
        assert!(!graph.add_connection(start_id, PortId::new(), dlg_id, PortId::new()));
    }

    #[test]
    fn connection_with_nonexistent_nodes_rejected() {
        let mut graph = DialogueGraph::new();
        assert!(!graph.add_connection(
            Uuid::new_v4(),
            PortId::new(),
            Uuid::new_v4(),
            PortId::new()
        ));
    }

    #[test]
    fn graph_serialization_roundtrip() {
        let mut graph = DialogueGraph::new();
        graph.add_node(Node::new_start([10.0, 20.0]));
        graph.add_node(Node::new_dialogue([30.0, 40.0]));
        let json = serde_json::to_string(&graph).unwrap();
        let loaded: DialogueGraph = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.nodes.len(), 2);
    }
}
