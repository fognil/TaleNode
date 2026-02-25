use std::collections::HashSet;
use uuid::Uuid;

use crate::model::graph::DialogueGraph;
use crate::model::node::VariableValue;

use super::ink_parse::InkLine;

pub(super) fn connect_nodes(
    graph: &mut DialogueGraph,
    from_id: Uuid,
    from_port_idx: usize,
    to_id: Uuid,
) {
    let from_port = {
        let Some(node) = graph.nodes.get(&from_id) else {
            return;
        };
        let Some(p) = node.outputs.get(from_port_idx) else {
            return;
        };
        p.id
    };
    let to_port = {
        let Some(node) = graph.nodes.get(&to_id) else {
            return;
        };
        let Some(p) = node.inputs.first() else {
            return;
        };
        p.id
    };
    graph.add_connection(from_id, from_port, to_id, to_port);
}

pub(super) fn link_prev(
    first_id: &mut Option<Uuid>,
    prev_id: &mut Option<Uuid>,
    node_id: Uuid,
    graph: &mut DialogueGraph,
) {
    if first_id.is_none() {
        *first_id = Some(node_id);
    }
    if let Some(pid) = *prev_id {
        connect_nodes(graph, pid, 0, node_id);
    }
    *prev_id = Some(node_id);
}

pub(super) fn parse_speaker_text(text: &str) -> (Option<String>, String) {
    if let Some(colon_pos) = text.find(':') {
        let candidate = text[..colon_pos].trim();
        if !candidate.is_empty()
            && candidate
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == ' ')
        {
            let dialogue = text[colon_pos + 1..].trim().to_string();
            return (Some(candidate.to_string()), dialogue);
        }
    }
    (None, text.to_string())
}

pub(super) fn guess_value(s: &str) -> VariableValue {
    let s = s.trim();
    if s == "true" {
        return VariableValue::Bool(true);
    }
    if s == "false" {
        return VariableValue::Bool(false);
    }
    if let Ok(n) = s.parse::<i64>() {
        return VariableValue::Int(n);
    }
    if let Ok(f) = s.parse::<f64>() {
        return VariableValue::Float(f);
    }
    VariableValue::Text(s.trim_matches('"').to_string())
}

pub(super) fn collect_info(
    lines: &[InkLine],
    speakers: &mut HashSet<String>,
    var_names: &mut HashSet<String>,
) {
    for line in lines {
        match line {
            InkLine::Dialogue { text, .. } | InkLine::Gather { text, .. } => {
                if let (Some(sp), _) = parse_speaker_text(text) {
                    speakers.insert(sp);
                }
            }
            InkLine::Choice { body, .. } => {
                collect_info(body, speakers, var_names);
            }
            InkLine::VarDecl { name, .. } | InkLine::VarSet { name, .. } => {
                var_names.insert(name.clone());
            }
            InkLine::Condition {
                true_body,
                false_body,
                ..
            } => {
                collect_info(true_body, speakers, var_names);
                collect_info(false_body, speakers, var_names);
            }
            InkLine::Divert { .. } => {}
        }
    }
}
