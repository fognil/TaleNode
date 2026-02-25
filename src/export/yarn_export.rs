use std::collections::HashSet;

use uuid::Uuid;

use crate::model::graph::DialogueGraph;
use crate::model::node::NodeType;

use super::json_export_helpers::compare_op_str;

/// Export a DialogueGraph to Yarn-like text format.
/// SubGraph nodes are flattened first so nested dialogues are included.
pub fn export_yarn(graph: &DialogueGraph) -> String {
    let flat = super::flatten::flatten_subgraphs(graph);
    let mut out = String::with_capacity(2048);
    out.push_str("title: root\n---\n");

    if let Some(start_id) = find_start(&flat) {
        if let Some(next) = follow_output(&flat, start_id, 0) {
            let mut visited = HashSet::new();
            emit_node(&flat, next, &mut visited, &mut out, 0);
        }
    }

    out.push_str("===\n");
    out
}

fn find_start(graph: &DialogueGraph) -> Option<Uuid> {
    graph
        .nodes
        .values()
        .find(|n| matches!(n.node_type, NodeType::Start))
        .map(|n| n.id)
}

/// Follow the Nth output port of a node to find the connected target node.
fn follow_output(graph: &DialogueGraph, node_id: Uuid, port_index: usize) -> Option<Uuid> {
    let node = graph.nodes.get(&node_id)?;
    let port = node.outputs.get(port_index)?;
    graph
        .connections
        .iter()
        .find(|c| c.from_port == port.id)
        .map(|c| c.to_node)
}

fn emit_node(
    graph: &DialogueGraph,
    node_id: Uuid,
    visited: &mut HashSet<Uuid>,
    out: &mut String,
    indent: usize,
) {
    if visited.contains(&node_id) {
        return;
    }
    visited.insert(node_id);

    let Some(node) = graph.nodes.get(&node_id) else {
        return;
    };
    let prefix = "    ".repeat(indent);

    match &node.node_type {
        NodeType::Start => {
            if let Some(next) = follow_output(graph, node_id, 0) {
                emit_node(graph, next, visited, out, indent);
            }
        }
        NodeType::Dialogue(d) => {
            emit_dialogue(d, &prefix, out, graph);
            if let Some(next) = follow_output(graph, node_id, 0) {
                emit_node(graph, next, visited, out, indent);
            }
        }
        NodeType::Choice(d) => {
            if !d.prompt.is_empty() {
                out.push_str(&format!("{prefix}{}\n", d.prompt));
            }
            for (i, choice) in d.choices.iter().enumerate() {
                out.push_str(&format!("{prefix}-> {}\n", choice.text));
                if let Some(next) = follow_output(graph, node_id, i) {
                    emit_node(graph, next, visited, out, indent + 1);
                }
            }
        }
        NodeType::Condition(d) => {
            emit_condition(d, graph, node_id, visited, out, &prefix, indent);
        }
        NodeType::Event(d) => {
            for action in &d.actions {
                let val = variable_value_str(&action.value);
                out.push_str(&format!("{prefix}<<set ${} to {val}>>\n", action.key));
            }
            if let Some(next) = follow_output(graph, node_id, 0) {
                emit_node(graph, next, visited, out, indent);
            }
        }
        NodeType::Random(d) => {
            out.push_str(&format!(
                "{prefix}# [RANDOM: {} branches]\n",
                d.branches.len()
            ));
        }
        NodeType::End(_) => {
            // End nodes are implicit — the section terminator === is added by caller.
        }
        NodeType::SubGraph(d) => {
            let name = if d.name.is_empty() { "unnamed" } else { &d.name };
            out.push_str(&format!("{prefix}# [SUBGRAPH: {name}]\n"));
            if let Some(next) = follow_output(graph, node_id, 0) {
                emit_node(graph, next, visited, out, indent);
            }
        }
    }
}

fn emit_dialogue(
    d: &crate::model::node::DialogueData,
    prefix: &str,
    out: &mut String,
    graph: &DialogueGraph,
) {
    let speaker = resolve_speaker(d, graph);
    if speaker.is_empty() {
        out.push_str(&format!("{prefix}{}\n", d.text));
    } else {
        out.push_str(&format!("{prefix}{speaker}: {}\n", d.text));
    }
    if d.emotion != "neutral" && !d.emotion.is_empty() {
        out.push_str(&format!("{prefix}# emotion: {}\n", d.emotion));
    }
    if let Some(ref audio) = d.audio_clip {
        if !audio.is_empty() {
            out.push_str(&format!("{prefix}# audio: {audio}\n"));
        }
    }
}

fn resolve_speaker(d: &crate::model::node::DialogueData, graph: &DialogueGraph) -> String {
    if let Some(sid) = d.speaker_id {
        graph
            .characters
            .iter()
            .find(|c| c.id == sid)
            .map(|c| c.name.clone())
            .unwrap_or_else(|| d.speaker_name.clone())
    } else {
        d.speaker_name.clone()
    }
}

fn emit_condition(
    d: &crate::model::node::ConditionData,
    graph: &DialogueGraph,
    node_id: Uuid,
    visited: &mut HashSet<Uuid>,
    out: &mut String,
    prefix: &str,
    indent: usize,
) {
    let op = compare_op_str(d.operator);
    let val = variable_value_str(&d.value);
    out.push_str(&format!("{prefix}<<if ${} {op} {val}>>\n", d.variable_name));
    if let Some(true_next) = follow_output(graph, node_id, 0) {
        emit_node(graph, true_next, visited, out, indent + 1);
    }
    if let Some(false_next) = follow_output(graph, node_id, 1) {
        out.push_str(&format!("{prefix}<<else>>\n"));
        emit_node(graph, false_next, visited, out, indent + 1);
    }
    out.push_str(&format!("{prefix}<<endif>>\n"));
}

fn variable_value_str(val: &crate::model::node::VariableValue) -> String {
    match val {
        crate::model::node::VariableValue::Bool(b) => b.to_string(),
        crate::model::node::VariableValue::Int(i) => i.to_string(),
        crate::model::node::VariableValue::Float(f) => f.to_string(),
        crate::model::node::VariableValue::Text(s) => format!("\"{s}\""),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::node::Node;

    #[test]
    fn empty_graph_produces_header_only() {
        let graph = DialogueGraph::new();
        let text = export_yarn(&graph);
        assert!(text.contains("title: root"));
        assert!(text.contains("==="));
    }

    #[test]
    fn simple_dialogue_chain() {
        let mut graph = DialogueGraph::new();
        let start = Node::new_start([0.0, 0.0]);
        let mut dlg = Node::new_dialogue([0.0, 100.0]);
        if let NodeType::Dialogue(ref mut d) = dlg.node_type {
            d.speaker_name = "Guard".to_string();
            d.text = "Halt!".to_string();
        }
        let end = Node::new_end([0.0, 200.0]);

        let s_out = start.outputs[0].id;
        let d_in = dlg.inputs[0].id;
        let d_out = dlg.outputs[0].id;
        let e_in = end.inputs[0].id;
        let (sid, did, eid) = (start.id, dlg.id, end.id);

        graph.add_node(start);
        graph.add_node(dlg);
        graph.add_node(end);
        graph.add_connection(sid, s_out, did, d_in);
        graph.add_connection(did, d_out, eid, e_in);

        let text = export_yarn(&graph);
        assert!(text.contains("Guard: Halt!"));
    }

    #[test]
    fn choice_node_emits_options() {
        let mut graph = DialogueGraph::new();
        let start = Node::new_start([0.0, 0.0]);
        let choice = Node::new_choice([0.0, 100.0]);
        let end = Node::new_end([0.0, 200.0]);

        let s_out = start.outputs[0].id;
        let ch_in = choice.inputs[0].id;
        let ch_out0 = choice.outputs[0].id;
        let e_in = end.inputs[0].id;
        let (sid, chid, eid) = (start.id, choice.id, end.id);

        graph.add_node(start);
        graph.add_node(choice);
        graph.add_node(end);
        graph.add_connection(sid, s_out, chid, ch_in);
        graph.add_connection(chid, ch_out0, eid, e_in);

        let text = export_yarn(&graph);
        assert!(text.contains("-> Option 1"));
    }

    #[test]
    fn round_trip_preserves_structure() {
        let mut graph = DialogueGraph::new();
        let start = Node::new_start([0.0, 0.0]);
        let mut dlg = Node::new_dialogue([0.0, 100.0]);
        if let NodeType::Dialogue(ref mut d) = dlg.node_type {
            d.speaker_name = "NPC".to_string();
            d.text = "Hello there!".to_string();
        }
        let end = Node::new_end([0.0, 200.0]);

        let s_out = start.outputs[0].id;
        let d_in = dlg.inputs[0].id;
        let d_out = dlg.outputs[0].id;
        let e_in = end.inputs[0].id;
        let (sid, did, eid) = (start.id, dlg.id, end.id);

        graph.add_node(start);
        graph.add_node(dlg);
        graph.add_node(end);
        graph.add_connection(sid, s_out, did, d_in);
        graph.add_connection(did, d_out, eid, e_in);

        let text = export_yarn(&graph);
        let reimported = crate::import::yarn_import::import_yarn(&text);
        assert!(reimported.is_ok());
        let g2 = reimported.unwrap();
        // Should have Start, Dialogue, End nodes
        assert_eq!(g2.nodes.len(), 3);
        // Should have at least one dialogue with "Hello there!"
        let has_hello = g2.nodes.values().any(|n| {
            if let NodeType::Dialogue(ref d) = n.node_type {
                d.text == "Hello there!"
            } else {
                false
            }
        });
        assert!(has_hello);
    }
}
