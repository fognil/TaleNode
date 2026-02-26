use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use crate::model::character::Character;
use crate::model::graph::DialogueGraph;
use crate::model::node::*;
use crate::model::node::Node;
use crate::model::port::Port;
use crate::model::variable::{Variable, VariableType};

use super::yarn_import::YarnLine;

pub(super) struct YarnNode {
    pub title: String,
    pub lines: Vec<YarnLine>,
}

pub(super) fn build_graph(yarn_nodes: Vec<YarnNode>) -> Result<DialogueGraph, String> {
    let mut graph = DialogueGraph::new();
    let mut speakers: HashSet<String> = HashSet::new();
    let mut variables: HashSet<String> = HashSet::new();

    for yn in &yarn_nodes {
        collect_speakers_and_vars(&yn.lines, &mut speakers, &mut variables);
    }

    let mut char_map: HashMap<String, Uuid> = HashMap::new();
    for name in &speakers {
        let ch = Character::new(name.clone());
        char_map.insert(name.clone(), ch.id);
        graph.characters.push(ch);
    }

    for var_name in &variables {
        let var = Variable {
            id: Uuid::new_v4(),
            name: var_name.clone(),
            var_type: VariableType::Text,
            default_value: VariableValue::Text(String::new()),
        };
        graph.variables.push(var);
    }

    let mut entry_points: HashMap<String, Uuid> = HashMap::new();
    let mut pending_jumps: Vec<(Uuid, usize, String)> = Vec::new();

    for (yn_idx, yn) in yarn_nodes.iter().enumerate() {
        let base_x = yn_idx as f32 * 400.0;
        let mut y_offset: f32 = 0.0;

        if yn_idx == 0 {
            let start = Node::new_start([base_x, y_offset]);
            let start_id = start.id;
            graph.add_node(start);
            y_offset += 200.0;

            let (first_id, _) = emit_lines(
                &yn.lines, &mut graph, &char_map, base_x, &mut y_offset,
                &mut pending_jumps,
            );
            if let Some(fid) = first_id {
                connect_nodes(&mut graph, start_id, 0, fid);
            }
            entry_points.insert(yn.title.clone(), start_id);
        } else {
            let (first_id, _) = emit_lines(
                &yn.lines, &mut graph, &char_map, base_x, &mut y_offset,
                &mut pending_jumps,
            );
            if let Some(fid) = first_id {
                entry_points.insert(yn.title.clone(), fid);
            }
        }
    }

    for (from_node, from_port_idx, target_title) in pending_jumps {
        if let Some(&target_id) = entry_points.get(&target_title) {
            connect_nodes(&mut graph, from_node, from_port_idx, target_id);
        }
    }

    Ok(graph)
}

fn emit_lines(
    lines: &[YarnLine],
    graph: &mut DialogueGraph,
    char_map: &HashMap<String, Uuid>,
    base_x: f32,
    y_offset: &mut f32,
    pending_jumps: &mut Vec<(Uuid, usize, String)>,
) -> (Option<Uuid>, Option<Uuid>) {
    let mut first_id: Option<Uuid> = None;
    let mut prev_id: Option<Uuid> = None;

    let mut i = 0;
    while i < lines.len() {
        match &lines[i] {
            YarnLine::Dialogue { speaker, text } => {
                let mut node = Node::new_dialogue([base_x, *y_offset]);
                if let NodeType::Dialogue(ref mut data) = node.node_type {
                    data.text = text.clone();
                    if let Some(sp) = speaker {
                        data.speaker_name = sp.clone();
                        data.speaker_id = char_map.get(sp).copied();
                    }
                }
                let node_id = node.id;
                graph.add_node(node);
                if first_id.is_none() { first_id = Some(node_id); }
                if let Some(pid) = prev_id {
                    connect_nodes(graph, pid, 0, node_id);
                }
                prev_id = Some(node_id);
                *y_offset += 200.0;
                i += 1;
            }
            YarnLine::ShortcutOption { .. } => {
                let mut options = Vec::new();
                while i < lines.len() {
                    if let YarnLine::ShortcutOption { text, condition, body } = &lines[i] {
                        options.push((text.clone(), condition.clone(), body));
                        i += 1;
                    } else {
                        break;
                    }
                }
                let choice_id = emit_choice_group(
                    &options, graph, char_map, base_x, y_offset,
                    pending_jumps,
                );
                if first_id.is_none() { first_id = Some(choice_id); }
                if let Some(pid) = prev_id {
                    connect_nodes(graph, pid, 0, choice_id);
                }
                prev_id = Some(choice_id);
            }
            YarnLine::SetCommand { variable, value } => {
                let mut node = Node::new_event([base_x, *y_offset]);
                if let NodeType::Event(ref mut data) = node.node_type {
                    data.actions.push(EventAction {
                        action_type: EventActionType::SetVariable,
                        key: variable.clone(),
                        value: guess_variable_value(value),
                    });
                }
                let node_id = node.id;
                graph.add_node(node);
                if first_id.is_none() { first_id = Some(node_id); }
                if let Some(pid) = prev_id {
                    connect_nodes(graph, pid, 0, node_id);
                }
                prev_id = Some(node_id);
                *y_offset += 200.0;
                i += 1;
            }
            YarnLine::JumpCommand { target } => {
                if let Some(pid) = prev_id {
                    pending_jumps.push((pid, 0, target.clone()));
                    prev_id = None;
                }
                i += 1;
            }
            YarnLine::Link { target } => {
                if let Some(pid) = prev_id {
                    pending_jumps.push((pid, 0, target.clone()));
                    prev_id = None;
                }
                i += 1;
            }
        }
    }

    if let Some(pid) = prev_id {
        let end = Node::new_end([base_x, *y_offset]);
        let end_id = end.id;
        graph.add_node(end);
        connect_nodes(graph, pid, 0, end_id);
        *y_offset += 200.0;
        return (first_id, Some(end_id));
    }

    (first_id, prev_id)
}

fn emit_choice_group(
    options: &[(String, Option<String>, &Vec<YarnLine>)],
    graph: &mut DialogueGraph,
    char_map: &HashMap<String, Uuid>,
    base_x: f32,
    y_offset: &mut f32,
    pending_jumps: &mut Vec<(Uuid, usize, String)>,
) -> Uuid {
    let mut choice_node = Node {
        id: Uuid::new_v4(),
        node_type: NodeType::Choice(ChoiceData {
            prompt: String::new(),
            choices: Vec::new(),
        }),
        position: [base_x, *y_offset],
        collapsed: false,
        inputs: vec![Port::input()],
        outputs: Vec::new(),
    };

    for (text, condition, _) in options {
        let cond_expr = condition.as_ref().map(|c| ConditionExpr {
            variable_name: c.clone(),
            operator: CompareOp::Eq,
            value: VariableValue::Bool(true),
        });
        if let NodeType::Choice(ref mut data) = choice_node.node_type {
            data.choices.push(ChoiceOption {
                id: Uuid::new_v4(),
                text: text.clone(),
                condition: cond_expr,
            });
        }
        choice_node.outputs.push(Port::output_with_label(text));
    }

    let choice_id = choice_node.id;
    graph.add_node(choice_node);
    *y_offset += 200.0;

    for (port_idx, (_, _, body)) in options.iter().enumerate() {
        if body.is_empty() {
            continue;
        }
        let branch_x = base_x + (port_idx as f32) * 300.0;
        let mut branch_y = *y_offset;
        let (branch_first, _) = emit_lines(
            body, graph, char_map, branch_x, &mut branch_y,
            pending_jumps,
        );
        if let Some(bf) = branch_first {
            connect_nodes(graph, choice_id, port_idx, bf);
        }
        if branch_y > *y_offset {
            *y_offset = branch_y;
        }
    }

    choice_id
}

fn connect_nodes(graph: &mut DialogueGraph, from_id: Uuid, from_port_idx: usize, to_id: Uuid) {
    let from_port = {
        let node = match graph.nodes.get(&from_id) {
            Some(n) => n,
            None => return,
        };
        match node.outputs.get(from_port_idx) {
            Some(p) => p.id,
            None => return,
        }
    };
    let to_port = {
        let node = match graph.nodes.get(&to_id) {
            Some(n) => n,
            None => return,
        };
        match node.inputs.first() {
            Some(p) => p.id,
            None => return,
        }
    };
    graph.add_connection(from_id, from_port, to_id, to_port);
}

fn guess_variable_value(s: &str) -> VariableValue {
    if s == "true" {
        return VariableValue::Bool(true);
    }
    if s == "false" {
        return VariableValue::Bool(false);
    }
    if let Ok(n) = s.parse::<i64>() {
        return VariableValue::Int(n);
    }
    VariableValue::Text(s.trim_matches('"').to_string())
}

fn collect_speakers_and_vars(
    lines: &[YarnLine],
    speakers: &mut HashSet<String>,
    variables: &mut HashSet<String>,
) {
    for line in lines {
        match line {
            YarnLine::Dialogue { speaker: Some(s), .. } => {
                speakers.insert(s.clone());
            }
            YarnLine::ShortcutOption { body, .. } => {
                collect_speakers_and_vars(body, speakers, variables);
            }
            YarnLine::SetCommand { variable, .. } => {
                variables.insert(variable.clone());
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::import::yarn_import::YarnLine;

    fn dlg(speaker: Option<&str>, text: &str) -> YarnLine {
        YarnLine::Dialogue { speaker: speaker.map(String::from), text: text.into() }
    }

    #[test]
    fn guess_variable_value_bool() {
        assert!(matches!(guess_variable_value("true"), VariableValue::Bool(true)));
        assert!(matches!(guess_variable_value("false"), VariableValue::Bool(false)));
    }

    #[test]
    fn guess_variable_value_int() {
        assert!(matches!(guess_variable_value("42"), VariableValue::Int(42)));
        assert!(matches!(guess_variable_value("-7"), VariableValue::Int(-7)));
    }

    #[test]
    fn guess_variable_value_text() {
        assert_eq!(guess_variable_value("hello"), VariableValue::Text("hello".into()));
        assert_eq!(guess_variable_value("\"quoted\""), VariableValue::Text("quoted".into()));
    }

    #[test]
    fn build_graph_simple_dialogue() {
        let nodes = vec![YarnNode {
            title: "Start".into(),
            lines: vec![dlg(None, "Hello"), dlg(None, "World")],
        }];
        let g = build_graph(nodes).unwrap();
        // Start + 2 Dialogue + End = 4 nodes
        assert_eq!(g.nodes.len(), 4);
        let starts = g.nodes.values().filter(|n| matches!(n.node_type, NodeType::Start)).count();
        let dlgs = g.nodes.values().filter(|n| matches!(n.node_type, NodeType::Dialogue(_))).count();
        let ends = g.nodes.values().filter(|n| matches!(n.node_type, NodeType::End(_))).count();
        assert_eq!((starts, dlgs, ends), (1, 2, 1));
    }

    #[test]
    fn build_graph_creates_characters() {
        let nodes = vec![YarnNode {
            title: "Start".into(),
            lines: vec![dlg(Some("Guard"), "Halt!")],
        }];
        let g = build_graph(nodes).unwrap();
        assert!(g.characters.iter().any(|c| c.name == "Guard"));
    }

    #[test]
    fn build_graph_creates_variables() {
        let nodes = vec![YarnNode {
            title: "Start".into(),
            lines: vec![YarnLine::SetCommand { variable: "gold".into(), value: "10".into() }],
        }];
        let g = build_graph(nodes).unwrap();
        assert!(g.variables.iter().any(|v| v.name == "gold"));
    }

    #[test]
    fn build_graph_with_choices() {
        let nodes = vec![YarnNode {
            title: "Start".into(),
            lines: vec![
                YarnLine::ShortcutOption { text: "Yes".into(), condition: None, body: vec![] },
                YarnLine::ShortcutOption { text: "No".into(), condition: None, body: vec![] },
            ],
        }];
        let g = build_graph(nodes).unwrap();
        let choices = g.nodes.values().filter(|n| matches!(n.node_type, NodeType::Choice(_))).count();
        assert_eq!(choices, 1);
    }
}
