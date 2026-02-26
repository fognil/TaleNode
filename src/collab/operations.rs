use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::model::character::Character;
use crate::model::connection::Connection;
use crate::model::graph::DialogueGraph;
use crate::model::node::Node;
use crate::model::variable::Variable;

/// A single collaborative operation that can be applied to a graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CollabOp {
    AddNode {
        node_json: serde_json::Value,
    },
    RemoveNode {
        node_id: Uuid,
    },
    MoveNode {
        node_id: Uuid,
        position: [f32; 2],
    },
    AddConnection {
        conn_json: serde_json::Value,
    },
    RemoveConnection {
        connection_id: Uuid,
    },
    EditNodeField {
        node_id: Uuid,
        node_json: serde_json::Value,
    },
    AddVariable {
        var_json: serde_json::Value,
    },
    RemoveVariable {
        var_id: Uuid,
    },
    EditVariable {
        var_json: serde_json::Value,
    },
    AddCharacter {
        char_json: serde_json::Value,
    },
    RemoveCharacter {
        char_id: Uuid,
    },
    EditCharacter {
        char_json: serde_json::Value,
    },
}

/// Apply a single operation to the graph. Returns true if applied.
#[allow(dead_code)]
pub fn apply_op(graph: &mut DialogueGraph, op: &CollabOp) -> bool {
    match op {
        CollabOp::AddNode { node_json } => {
            if let Ok(node) = serde_json::from_value::<Node>(node_json.clone()) {
                graph.add_node(node);
                return true;
            }
        }
        CollabOp::RemoveNode { node_id } => {
            graph.remove_node(*node_id);
            return true;
        }
        CollabOp::MoveNode { node_id, position } => {
            if let Some(node) = graph.nodes.get_mut(node_id) {
                node.position = *position;
                return true;
            }
        }
        CollabOp::AddConnection { conn_json } => {
            if let Ok(conn) =
                serde_json::from_value::<Connection>(conn_json.clone())
            {
                graph.connections.push(conn);
                return true;
            }
        }
        CollabOp::RemoveConnection { connection_id } => {
            graph.connections.retain(|c| c.id != *connection_id);
            return true;
        }
        CollabOp::EditNodeField { node_id, node_json } => {
            if let Ok(updated) =
                serde_json::from_value::<Node>(node_json.clone())
            {
                if let Some(existing) = graph.nodes.get_mut(node_id) {
                    existing.node_type = updated.node_type;
                    return true;
                }
            }
        }
        CollabOp::AddVariable { var_json } => {
            if let Ok(var) =
                serde_json::from_value::<Variable>(var_json.clone())
            {
                graph.variables.push(var);
                return true;
            }
        }
        CollabOp::RemoveVariable { var_id } => {
            graph.variables.retain(|v| v.id != *var_id);
            return true;
        }
        CollabOp::EditVariable { var_json } => {
            if let Ok(updated) =
                serde_json::from_value::<Variable>(var_json.clone())
            {
                if let Some(existing) =
                    graph.variables.iter_mut().find(|v| v.id == updated.id)
                {
                    *existing = updated;
                    return true;
                }
            }
        }
        CollabOp::AddCharacter { char_json } => {
            if let Ok(ch) =
                serde_json::from_value::<Character>(char_json.clone())
            {
                graph.characters.push(ch);
                return true;
            }
        }
        CollabOp::RemoveCharacter { char_id } => {
            graph.characters.retain(|c| c.id != *char_id);
            return true;
        }
        CollabOp::EditCharacter { char_json } => {
            if let Ok(updated) =
                serde_json::from_value::<Character>(char_json.clone())
            {
                if let Some(existing) =
                    graph.characters.iter_mut().find(|c| c.id == updated.id)
                {
                    *existing = updated;
                    return true;
                }
            }
        }
    }
    false
}

/// Compute operations needed to transform `old` into `new`.
/// Uses Last-Write-Wins: any changed node emits an EditNodeField op.
#[allow(dead_code)]
pub fn diff_to_ops(old: &DialogueGraph, new: &DialogueGraph) -> Vec<CollabOp> {
    let mut ops = Vec::new();

    // Added nodes
    for (id, node) in &new.nodes {
        if !old.nodes.contains_key(id) {
            if let Ok(json) = serde_json::to_value(node) {
                ops.push(CollabOp::AddNode { node_json: json });
            }
        }
    }
    // Removed nodes
    for id in old.nodes.keys() {
        if !new.nodes.contains_key(id) {
            ops.push(CollabOp::RemoveNode { node_id: *id });
        }
    }
    // Changed nodes (position or type)
    for (id, new_node) in &new.nodes {
        if let Some(old_node) = old.nodes.get(id) {
            if old_node.position != new_node.position {
                ops.push(CollabOp::MoveNode {
                    node_id: *id,
                    position: new_node.position,
                });
            }
            let old_json = serde_json::to_value(&old_node.node_type).ok();
            let new_json = serde_json::to_value(&new_node.node_type).ok();
            if old_json != new_json {
                if let Ok(json) = serde_json::to_value(new_node) {
                    ops.push(CollabOp::EditNodeField {
                        node_id: *id,
                        node_json: json,
                    });
                }
            }
        }
    }

    // Connection diffs
    diff_connections(old, new, &mut ops);

    ops
}

fn diff_connections(
    old: &DialogueGraph,
    new: &DialogueGraph,
    ops: &mut Vec<CollabOp>,
) {
    let old_ids: std::collections::HashSet<Uuid> =
        old.connections.iter().map(|c| c.id).collect();
    let new_ids: std::collections::HashSet<Uuid> =
        new.connections.iter().map(|c| c.id).collect();

    for conn in &new.connections {
        if !old_ids.contains(&conn.id) {
            if let Ok(json) = serde_json::to_value(conn) {
                ops.push(CollabOp::AddConnection { conn_json: json });
            }
        }
    }
    for conn in &old.connections {
        if !new_ids.contains(&conn.id) {
            ops.push(CollabOp::RemoveConnection {
                connection_id: conn.id,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::node::Node;

    #[test]
    fn diff_detects_added_node() {
        let old = DialogueGraph::new();
        let mut new = old.clone();
        new.add_node(Node::new_dialogue([100.0, 100.0]));
        let ops = diff_to_ops(&old, &new);
        assert!(ops.iter().any(|o| matches!(o, CollabOp::AddNode { .. })));
    }

    #[test]
    fn diff_detects_removed_node() {
        let mut old = DialogueGraph::new();
        let node = Node::new_dialogue([0.0, 0.0]);
        let id = node.id;
        old.add_node(node);
        let mut new = old.clone();
        new.remove_node(id);
        let ops = diff_to_ops(&old, &new);
        assert!(ops
            .iter()
            .any(|o| matches!(o, CollabOp::RemoveNode { node_id } if *node_id == id)));
    }

    #[test]
    fn diff_detects_moved_node() {
        let mut old = DialogueGraph::new();
        let node = Node::new_dialogue([0.0, 0.0]);
        let id = node.id;
        old.add_node(node);
        let mut new = old.clone();
        new.nodes.get_mut(&id).unwrap().position = [50.0, 50.0];
        let ops = diff_to_ops(&old, &new);
        assert!(ops.iter().any(|o| matches!(o, CollabOp::MoveNode { .. })));
    }

    #[test]
    fn apply_add_and_remove_node() {
        let mut graph = DialogueGraph::new();
        let node = Node::new_dialogue([0.0, 0.0]);
        let id = node.id;
        let json = serde_json::to_value(&node).unwrap();

        assert!(apply_op(&mut graph, &CollabOp::AddNode { node_json: json }));
        assert!(graph.nodes.contains_key(&id));

        assert!(apply_op(&mut graph, &CollabOp::RemoveNode { node_id: id }));
        assert!(!graph.nodes.contains_key(&id));
    }

    #[test]
    fn apply_move_node() {
        let mut graph = DialogueGraph::new();
        let node = Node::new_dialogue([0.0, 0.0]);
        let id = node.id;
        graph.add_node(node);

        apply_op(
            &mut graph,
            &CollabOp::MoveNode {
                node_id: id,
                position: [100.0, 200.0],
            },
        );
        assert_eq!(graph.nodes.get(&id).unwrap().position, [100.0, 200.0]);
    }
}
