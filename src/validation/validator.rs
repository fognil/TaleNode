use std::collections::{HashMap, HashSet, VecDeque};
use uuid::Uuid;

use crate::model::graph::DialogueGraph;
use crate::model::node::NodeType;

#[derive(Debug, Clone)]
pub struct ValidationWarning {
    pub node_id: Option<Uuid>,
    pub message: String,
    pub severity: Severity,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Severity {
    Error,
    Warning,
}

/// Validate the dialogue graph and return a list of warnings/errors.
pub fn validate(graph: &DialogueGraph) -> Vec<ValidationWarning> {
    let mut warnings = Vec::new();

    check_start_nodes(graph, &mut warnings);
    check_disconnected_outputs(graph, &mut warnings);
    check_unreachable_nodes(graph, &mut warnings);
    check_empty_dialogue(graph, &mut warnings);
    check_dead_ends(graph, &mut warnings);

    warnings
}

/// Must have exactly one Start node.
fn check_start_nodes(graph: &DialogueGraph, warnings: &mut Vec<ValidationWarning>) {
    let start_count = graph
        .nodes
        .values()
        .filter(|n| matches!(n.node_type, NodeType::Start))
        .count();

    if start_count == 0 {
        warnings.push(ValidationWarning {
            node_id: None,
            message: "No Start node found".to_string(),
            severity: Severity::Error,
        });
    } else if start_count > 1 {
        warnings.push(ValidationWarning {
            node_id: None,
            message: format!("Multiple Start nodes found ({start_count})"),
            severity: Severity::Warning,
        });
    }
}

/// Check for output ports that have no connection (except End nodes).
fn check_disconnected_outputs(graph: &DialogueGraph, warnings: &mut Vec<ValidationWarning>) {
    let connected_outputs: HashSet<_> = graph
        .connections
        .iter()
        .map(|c| c.from_port)
        .collect();

    for node in graph.nodes.values() {
        if matches!(node.node_type, NodeType::End(_)) {
            continue;
        }
        for port in &node.outputs {
            if !connected_outputs.contains(&port.id) {
                warnings.push(ValidationWarning {
                    node_id: Some(node.id),
                    message: format!(
                        "'{}' has unconnected output",
                        node.title()
                    ),
                    severity: Severity::Warning,
                });
                break; // One warning per node is enough
            }
        }
    }
}

/// Check for nodes not reachable from any Start node.
fn check_unreachable_nodes(graph: &DialogueGraph, warnings: &mut Vec<ValidationWarning>) {
    // BFS from all Start nodes
    let mut reachable = HashSet::new();
    let mut queue = VecDeque::new();

    // Build adjacency: node -> [connected nodes via outputs]
    let mut adjacency: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
    for conn in &graph.connections {
        adjacency
            .entry(conn.from_node)
            .or_default()
            .push(conn.to_node);
    }

    for node in graph.nodes.values() {
        if matches!(node.node_type, NodeType::Start) {
            queue.push_back(node.id);
            reachable.insert(node.id);
        }
    }

    while let Some(id) = queue.pop_front() {
        if let Some(neighbors) = adjacency.get(&id) {
            for &next in neighbors {
                if reachable.insert(next) {
                    queue.push_back(next);
                }
            }
        }
    }

    for node in graph.nodes.values() {
        if !reachable.contains(&node.id) && !matches!(node.node_type, NodeType::Start) {
            warnings.push(ValidationWarning {
                node_id: Some(node.id),
                message: format!("'{}' is unreachable from Start", node.title()),
                severity: Severity::Warning,
            });
        }
    }
}

/// Check for Dialogue nodes with empty text.
fn check_empty_dialogue(graph: &DialogueGraph, warnings: &mut Vec<ValidationWarning>) {
    for node in graph.nodes.values() {
        if let NodeType::Dialogue(data) = &node.node_type {
            if data.text.trim().is_empty() {
                warnings.push(ValidationWarning {
                    node_id: Some(node.id),
                    message: format!("'{}' has empty text", node.title()),
                    severity: Severity::Warning,
                });
            }
        }
    }
}

/// Check for non-End nodes that have no outgoing connections and should have.
fn check_dead_ends(graph: &DialogueGraph, warnings: &mut Vec<ValidationWarning>) {
    let nodes_with_output: HashSet<_> = graph
        .connections
        .iter()
        .map(|c| c.from_node)
        .collect();

    for node in graph.nodes.values() {
        if matches!(node.node_type, NodeType::End(_)) {
            continue;
        }
        if node.outputs.is_empty() {
            continue;
        }
        if !nodes_with_output.contains(&node.id) {
            warnings.push(ValidationWarning {
                node_id: Some(node.id),
                message: format!("'{}' is a dead end (no outgoing connections)", node.title()),
                severity: Severity::Warning,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::node::Node;

    #[test]
    fn no_start_node_error() {
        let graph = DialogueGraph::new();
        let warnings = validate(&graph);
        assert!(warnings.iter().any(|w| w.message.contains("No Start")));
    }

    #[test]
    fn valid_simple_graph() {
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

        let warnings = validate(&graph);
        let errors: Vec<_> = warnings.iter().filter(|w| w.severity == Severity::Error).collect();
        assert!(errors.is_empty());
    }

    #[test]
    fn unreachable_node_warning() {
        let mut graph = DialogueGraph::new();
        graph.add_node(Node::new_start([0.0, 0.0]));
        graph.add_node(Node::new_dialogue([200.0, 200.0])); // not connected

        let warnings = validate(&graph);
        assert!(warnings.iter().any(|w| w.message.contains("unreachable")));
    }

    #[test]
    fn dead_end_warning() {
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

        let warnings = validate(&graph);
        assert!(warnings.iter().any(|w| w.message.contains("dead end")));
    }

    #[test]
    fn multiple_start_nodes_warning() {
        let mut graph = DialogueGraph::new();
        graph.add_node(Node::new_start([0.0, 0.0]));
        graph.add_node(Node::new_start([200.0, 0.0]));

        let warnings = validate(&graph);
        assert!(warnings
            .iter()
            .any(|w| w.message.contains("Multiple Start") && w.severity == Severity::Warning));
    }

    #[test]
    fn empty_dialogue_warning() {
        let mut graph = DialogueGraph::new();
        let start = Node::new_start([0.0, 0.0]);
        let dlg = Node::new_dialogue([200.0, 0.0]); // empty text
        let end = Node::new_end([400.0, 0.0]);

        let s_out = start.outputs[0].id;
        let d_in = dlg.inputs[0].id;
        let d_out = dlg.outputs[0].id;
        let e_in = end.inputs[0].id;
        let s_id = start.id;
        let d_id = dlg.id;
        let e_id = end.id;

        graph.add_node(start);
        graph.add_node(dlg);
        graph.add_node(end);
        graph.add_connection(s_id, s_out, d_id, d_in);
        graph.add_connection(d_id, d_out, e_id, e_in);

        let warnings = validate(&graph);
        assert!(warnings.iter().any(|w| w.message.contains("empty text")));
    }

    #[test]
    fn whitespace_only_dialogue_warning() {
        let mut graph = DialogueGraph::new();
        let mut dlg = Node::new_dialogue([200.0, 0.0]);
        if let NodeType::Dialogue(ref mut d) = dlg.node_type {
            d.text = "   \n  ".to_string();
        }
        graph.add_node(Node::new_start([0.0, 0.0]));
        graph.add_node(dlg);

        let warnings = validate(&graph);
        assert!(warnings.iter().any(|w| w.message.contains("empty text")));
    }

    #[test]
    fn disconnected_output_warning() {
        let mut graph = DialogueGraph::new();
        let start = Node::new_start([0.0, 0.0]);
        let choice = Node::new_choice([200.0, 0.0]);
        let end = Node::new_end([400.0, 0.0]);

        let s_out = start.outputs[0].id;
        let c_in = choice.inputs[0].id;
        let c_out0 = choice.outputs[0].id;
        // c_out1 is NOT connected
        let e_in = end.inputs[0].id;
        let s_id = start.id;
        let c_id = choice.id;
        let e_id = end.id;

        graph.add_node(start);
        graph.add_node(choice);
        graph.add_node(end);
        graph.add_connection(s_id, s_out, c_id, c_in);
        graph.add_connection(c_id, c_out0, e_id, e_in);

        let warnings = validate(&graph);
        assert!(warnings
            .iter()
            .any(|w| w.message.contains("unconnected output")));
    }

    #[test]
    fn fully_connected_graph_no_errors() {
        let mut graph = DialogueGraph::new();
        let start = Node::new_start([0.0, 0.0]);
        let mut dlg = Node::new_dialogue([200.0, 0.0]);
        if let NodeType::Dialogue(ref mut d) = dlg.node_type {
            d.text = "Hello!".to_string();
        }
        let end = Node::new_end([400.0, 0.0]);

        let s_out = start.outputs[0].id;
        let d_in = dlg.inputs[0].id;
        let d_out = dlg.outputs[0].id;
        let e_in = end.inputs[0].id;
        let s_id = start.id;
        let d_id = dlg.id;
        let e_id = end.id;

        graph.add_node(start);
        graph.add_node(dlg);
        graph.add_node(end);
        graph.add_connection(s_id, s_out, d_id, d_in);
        graph.add_connection(d_id, d_out, e_id, e_in);

        let warnings = validate(&graph);
        assert!(warnings.is_empty(), "Expected no warnings, got: {warnings:?}");
    }

    #[test]
    fn end_node_no_dead_end_or_disconnected_warning() {
        let mut graph = DialogueGraph::new();
        let start = Node::new_start([0.0, 0.0]);
        let end = Node::new_end([200.0, 0.0]);
        let s_out = start.outputs[0].id;
        let e_in = end.inputs[0].id;
        let s_id = start.id;
        let e_id = end.id;
        graph.add_node(start);
        graph.add_node(end);
        graph.add_connection(s_id, s_out, e_id, e_in);

        let warnings = validate(&graph);
        // End node should not trigger "dead end" or "disconnected output"
        assert!(!warnings
            .iter()
            .any(|w| w.message.contains("dead end") || w.message.contains("unconnected")));
    }
}
