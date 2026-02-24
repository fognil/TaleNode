use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::character::Character;
use super::connection::Connection;
use super::node::Node;
use super::port::{PortDirection, PortId};
use super::variable::Variable;

/// The central data structure holding the entire dialogue graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueGraph {
    pub nodes: HashMap<Uuid, Node>,
    pub connections: Vec<Connection>,
    #[serde(default)]
    pub variables: Vec<Variable>,
    #[serde(default)]
    pub characters: Vec<Character>,
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

    pub fn remove_connection(&mut self, connection_id: Uuid) -> Option<Connection> {
        if let Some(idx) = self.connections.iter().position(|c| c.id == connection_id) {
            Some(self.connections.remove(idx))
        } else {
            None
        }
    }

    /// Find which node and port a given PortId belongs to.
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
}
