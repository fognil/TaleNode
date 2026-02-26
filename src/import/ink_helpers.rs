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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::graph::DialogueGraph;
    use crate::model::node::Node;

    #[test]
    fn parse_speaker_text_with_speaker() {
        let (speaker, text) = parse_speaker_text("Guard: Stay back!");
        assert_eq!(speaker, Some("Guard".to_string()));
        assert_eq!(text, "Stay back!");
    }

    #[test]
    fn parse_speaker_text_without_colon() {
        let (speaker, text) = parse_speaker_text("Hello there");
        assert!(speaker.is_none());
        assert_eq!(text, "Hello there");
    }

    #[test]
    fn parse_speaker_text_special_chars_rejected() {
        let (speaker, text) = parse_speaker_text("foo@bar: text");
        assert!(speaker.is_none());
        assert_eq!(text, "foo@bar: text");
    }

    #[test]
    fn parse_speaker_text_empty_speaker() {
        let (speaker, text) = parse_speaker_text(": text");
        assert!(speaker.is_none());
        assert_eq!(text, ": text");
    }

    #[test]
    fn guess_value_bool_true() {
        assert!(matches!(guess_value("true"), VariableValue::Bool(true)));
    }

    #[test]
    fn guess_value_bool_false() {
        assert!(matches!(guess_value("false"), VariableValue::Bool(false)));
    }

    #[test]
    fn guess_value_int() {
        assert!(matches!(guess_value("42"), VariableValue::Int(42)));
    }

    #[test]
    fn guess_value_float() {
        if let VariableValue::Float(f) = guess_value("3.14") {
            assert!((f - 3.14).abs() < f64::EPSILON);
        } else {
            panic!("Expected Float variant");
        }
    }

    #[test]
    fn guess_value_text() {
        assert!(matches!(guess_value("hello"), VariableValue::Text(ref s) if s == "hello"));
    }

    #[test]
    fn guess_value_quoted_text() {
        assert!(matches!(guess_value("\"hello\""), VariableValue::Text(ref s) if s == "hello"));
    }

    #[test]
    fn connect_nodes_basic() {
        let mut graph = DialogueGraph::new();
        let n1 = Node::new_dialogue([0.0, 0.0]);
        let n2 = Node::new_dialogue([100.0, 0.0]);
        let id1 = n1.id;
        let id2 = n2.id;
        graph.nodes.insert(id1, n1);
        graph.nodes.insert(id2, n2);

        connect_nodes(&mut graph, id1, 0, id2);
        assert_eq!(graph.connections.len(), 1);
        assert_eq!(graph.connections[0].from_node, id1);
        assert_eq!(graph.connections[0].to_node, id2);
    }

    #[test]
    fn connect_nodes_invalid_node() {
        let mut graph = DialogueGraph::new();
        let fake_id1 = Uuid::new_v4();
        let fake_id2 = Uuid::new_v4();

        // Should not panic with nonexistent nodes
        connect_nodes(&mut graph, fake_id1, 0, fake_id2);
        assert!(graph.connections.is_empty());
    }

    #[test]
    fn collect_info_extracts_speakers() {
        let lines = vec![
            InkLine::Dialogue {
                text: "Guard: Halt!".to_string(),
                tags: Vec::new(),
            },
            InkLine::Dialogue {
                text: "Just narration.".to_string(),
                tags: Vec::new(),
            },
        ];
        let mut speakers = HashSet::new();
        let mut var_names = HashSet::new();

        collect_info(&lines, &mut speakers, &mut var_names);
        assert!(speakers.contains("Guard"));
        assert_eq!(speakers.len(), 1);
    }
}
