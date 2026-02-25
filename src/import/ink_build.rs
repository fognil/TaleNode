use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use crate::model::character::Character;
use crate::model::graph::DialogueGraph;
use crate::model::node::*;
use crate::model::node::Node;
use crate::model::port::Port;
use crate::model::variable::{Variable, VariableType};

use super::ink_helpers::{collect_info, connect_nodes, guess_value, link_prev, parse_speaker_text};
use super::ink_parse::{InkKnot, InkLine};

/// Build a DialogueGraph from parsed Ink knots and global variables.
/// Two-pass: (1) collect all knot names as entry points, (2) emit nodes + wires.
pub(super) fn build_ink_graph(
    global_vars: Vec<(String, String)>,
    knots: Vec<InkKnot>,
) -> Result<DialogueGraph, String> {
    if knots.is_empty() {
        return Err("No content found in Ink file".to_string());
    }

    let mut graph = DialogueGraph::new();
    let mut speakers: HashSet<String> = HashSet::new();
    let mut var_names: HashSet<String> = HashSet::new();

    for knot in &knots {
        collect_info(&knot.lines, &mut speakers, &mut var_names);
    }
    for (name, _) in &global_vars {
        var_names.insert(name.clone());
    }

    let mut char_map: HashMap<String, Uuid> = HashMap::new();
    for name in &speakers {
        let ch = Character::new(name.clone());
        char_map.insert(name.clone(), ch.id);
        graph.characters.push(ch);
    }

    for var_name in &var_names {
        let default = global_vars
            .iter()
            .find(|(n, _)| n == var_name)
            .map(|(_, v)| guess_value(v))
            .unwrap_or(VariableValue::Text(String::new()));
        let var_type = match &default {
            VariableValue::Bool(_) => VariableType::Bool,
            VariableValue::Int(_) => VariableType::Int,
            VariableValue::Float(_) => VariableType::Float,
            VariableValue::Text(_) => VariableType::Text,
        };
        graph.variables.push(Variable {
            id: Uuid::new_v4(),
            name: var_name.clone(),
            var_type,
            default_value: default,
        });
    }

    let mut entry_points: HashMap<String, Uuid> = HashMap::new();
    let mut pending_diverts: Vec<(Uuid, usize, String)> = Vec::new();

    for (knot_idx, knot) in knots.iter().enumerate() {
        let base_x = knot_idx as f32 * 400.0;
        let mut y_offset: f32 = 0.0;

        if knot_idx == 0 {
            let start = Node::new_start([base_x, y_offset]);
            let start_id = start.id;
            graph.add_node(start);
            y_offset += 200.0;

            let (first_id, _) = emit_ink_lines(
                &knot.lines, &mut graph, &char_map, base_x, &mut y_offset,
                &mut pending_diverts, &knot.tags,
            );
            if let Some(fid) = first_id {
                connect_nodes(&mut graph, start_id, 0, fid);
            }
            entry_points.insert(knot.name.clone(), start_id);
        } else {
            let (first_id, _) = emit_ink_lines(
                &knot.lines, &mut graph, &char_map, base_x, &mut y_offset,
                &mut pending_diverts, &knot.tags,
            );
            if let Some(fid) = first_id {
                entry_points.insert(knot.name.clone(), fid);
            }
        }
    }

    for (from_node, from_port_idx, target) in pending_diverts {
        if let Some(&target_id) = entry_points.get(&target) {
            connect_nodes(&mut graph, from_node, from_port_idx, target_id);
        }
    }

    Ok(graph)
}

pub(super) fn emit_ink_lines(
    lines: &[InkLine],
    graph: &mut DialogueGraph,
    char_map: &HashMap<String, Uuid>,
    base_x: f32,
    y_offset: &mut f32,
    pending_diverts: &mut Vec<(Uuid, usize, String)>,
    _knot_tags: &[String],
) -> (Option<Uuid>, Option<Uuid>) {
    let mut first_id: Option<Uuid> = None;
    let mut prev_id: Option<Uuid> = None;
    let mut i = 0;

    while i < lines.len() {
        match &lines[i] {
            InkLine::Dialogue { text, tags } => {
                let (speaker, dialogue_text) = parse_speaker_text(text);
                let mut node = Node::new_dialogue([base_x, *y_offset]);
                if let NodeType::Dialogue(ref mut data) = node.node_type {
                    data.text = dialogue_text;
                    if let Some(ref sp) = speaker {
                        data.speaker_name = sp.clone();
                        data.speaker_id = char_map.get(sp).copied();
                    }
                    if let Some(tag) = tags.first() {
                        data.emotion = tag.clone();
                    }
                }
                let node_id = node.id;
                graph.add_node(node);
                link_prev(&mut first_id, &mut prev_id, node_id, graph);
                *y_offset += 200.0;
                i += 1;
            }
            InkLine::Choice { .. } => {
                let mut options = Vec::new();
                while i < lines.len() {
                    if let InkLine::Choice { text, body, condition, .. } = &lines[i] {
                        options.push((text.clone(), condition.clone(), body));
                        i += 1;
                    } else {
                        break;
                    }
                }
                let choice_id = emit_choice_group(
                    &options, graph, char_map, base_x, y_offset, pending_diverts,
                );
                link_prev(&mut first_id, &mut prev_id, choice_id, graph);
            }
            InkLine::Divert { target } => {
                if let Some(pid) = prev_id {
                    pending_diverts.push((pid, 0, target.clone()));
                    prev_id = None;
                }
                i += 1;
            }
            InkLine::VarDecl { name, value } | InkLine::VarSet { name, value } => {
                let mut node = Node::new_event([base_x, *y_offset]);
                if let NodeType::Event(ref mut data) = node.node_type {
                    data.actions.push(EventAction {
                        action_type: EventActionType::SetVariable,
                        key: name.clone(),
                        value: guess_value(value),
                    });
                }
                let node_id = node.id;
                graph.add_node(node);
                link_prev(&mut first_id, &mut prev_id, node_id, graph);
                *y_offset += 200.0;
                i += 1;
            }
            InkLine::Condition { expression, true_body, false_body } => {
                emit_condition(
                    expression, true_body, false_body,
                    graph, char_map, base_x, y_offset, pending_diverts,
                    &mut first_id, &mut prev_id,
                );
                i += 1;
            }
            InkLine::Gather { text, tags } => {
                if !text.is_empty() {
                    let (speaker, dialogue_text) = parse_speaker_text(text);
                    let mut node = Node::new_dialogue([base_x, *y_offset]);
                    if let NodeType::Dialogue(ref mut data) = node.node_type {
                        data.text = dialogue_text;
                        if let Some(ref sp) = speaker {
                            data.speaker_name = sp.clone();
                            data.speaker_id = char_map.get(sp).copied();
                        }
                        if let Some(tag) = tags.first() {
                            data.emotion = tag.clone();
                        }
                    }
                    let node_id = node.id;
                    graph.add_node(node);
                    link_prev(&mut first_id, &mut prev_id, node_id, graph);
                    *y_offset += 200.0;
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

#[allow(clippy::too_many_arguments)]
fn emit_condition(
    expression: &str,
    true_body: &[InkLine],
    false_body: &[InkLine],
    graph: &mut DialogueGraph,
    char_map: &HashMap<String, Uuid>,
    base_x: f32,
    y_offset: &mut f32,
    pending_diverts: &mut Vec<(Uuid, usize, String)>,
    first_id: &mut Option<Uuid>,
    prev_id: &mut Option<Uuid>,
) {
    let cond_node = Node {
        id: Uuid::new_v4(),
        node_type: NodeType::Condition(ConditionData {
            variable_name: expression.to_string(),
            operator: CompareOp::Eq,
            value: VariableValue::Bool(true),
        }),
        position: [base_x, *y_offset],
        collapsed: false,
        inputs: vec![Port::input()],
        outputs: vec![
            Port::output_with_label("True"),
            Port::output_with_label("False"),
        ],
    };
    let cond_id = cond_node.id;
    graph.add_node(cond_node);
    link_prev(first_id, prev_id, cond_id, graph);
    *y_offset += 200.0;

    if !true_body.is_empty() {
        let mut branch_y = *y_offset;
        let (bf, _) = emit_ink_lines(
            true_body, graph, char_map, base_x, &mut branch_y, pending_diverts, &[],
        );
        if let Some(bf_id) = bf {
            connect_nodes(graph, cond_id, 0, bf_id);
        }
        if branch_y > *y_offset {
            *y_offset = branch_y;
        }
    }
    if !false_body.is_empty() {
        let branch_x = base_x + 300.0;
        let mut branch_y = *y_offset;
        let (bf, _) = emit_ink_lines(
            false_body, graph, char_map, branch_x, &mut branch_y, pending_diverts, &[],
        );
        if let Some(bf_id) = bf {
            connect_nodes(graph, cond_id, 1, bf_id);
        }
        if branch_y > *y_offset {
            *y_offset = branch_y;
        }
    }
    *prev_id = Some(cond_id);
}

fn emit_choice_group(
    options: &[(String, Option<String>, &Vec<InkLine>)],
    graph: &mut DialogueGraph,
    char_map: &HashMap<String, Uuid>,
    base_x: f32,
    y_offset: &mut f32,
    pending_diverts: &mut Vec<(Uuid, usize, String)>,
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
        let (branch_first, _) = emit_ink_lines(
            body, graph, char_map, branch_x, &mut branch_y, pending_diverts, &[],
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::import::ink_parse::parse_ink;

    #[test]
    fn build_simple_dialogue() {
        let (vars, knots) = parse_ink("=== start ===\nGuard: Hello!\nGuard: Welcome.\n");
        let graph = build_ink_graph(vars, knots).expect("should build");
        assert!(graph.nodes.len() >= 3);
        assert_eq!(graph.characters.len(), 1);
    }

    #[test]
    fn build_with_choices() {
        let ink = "=== start ===\nWhat do you want?\n* Buy\n  Here you go.\n* Leave\n  Bye.\n";
        let (vars, knots) = parse_ink(ink);
        let graph = build_ink_graph(vars, knots).expect("should build");
        let choice_count = graph
            .nodes
            .values()
            .filter(|n| matches!(n.node_type, NodeType::Choice(_)))
            .count();
        assert!(choice_count >= 1);
    }

    #[test]
    fn build_with_variables() {
        let ink = "VAR gold = 100\n=== start ===\n~ gold = 50\nDone.\n";
        let (vars, knots) = parse_ink(ink);
        let graph = build_ink_graph(vars, knots).expect("should build");
        assert_eq!(graph.variables.len(), 1);
        assert_eq!(graph.variables[0].name, "gold");
    }

    #[test]
    fn build_with_diverts() {
        let ink = "=== start ===\nHello!\n-> shop\n=== shop ===\nWelcome to the shop!\n";
        let (vars, knots) = parse_ink(ink);
        let graph = build_ink_graph(vars, knots).expect("should build");
        assert!(graph.connections.len() >= 2);
    }

    #[test]
    fn build_empty_returns_error() {
        let result = build_ink_graph(Vec::new(), Vec::new());
        assert!(result.is_err());
    }
}
