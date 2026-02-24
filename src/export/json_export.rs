use serde::Serialize;
use std::collections::HashMap;
use uuid::Uuid;

use crate::model::graph::DialogueGraph;
use crate::model::node::{CompareOp, NodeType, VariableValue};

/// Exported dialogue JSON for game engines.
#[derive(Debug, Serialize)]
pub struct ExportedDialogue {
    pub version: String,
    pub name: String,
    pub variables: Vec<ExportedVariable>,
    pub characters: Vec<ExportedCharacter>,
    pub nodes: Vec<ExportedNode>,
}

#[derive(Debug, Serialize)]
pub struct ExportedVariable {
    pub name: String,
    #[serde(rename = "type")]
    pub var_type: String,
    pub default: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct ExportedCharacter {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub portrait: Option<String>,
    pub color: String,
}

#[derive(Debug, Serialize)]
pub struct ExportedNode {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: String,
    #[serde(flatten)]
    pub data: serde_json::Value,
}

/// Export a DialogueGraph to the game-engine JSON format.
pub fn export_json(graph: &DialogueGraph, name: &str) -> Result<String, serde_json::Error> {
    let exported = build_export(graph, name);
    serde_json::to_string_pretty(&exported)
}

fn build_export(graph: &DialogueGraph, name: &str) -> ExportedDialogue {
    // Build readable ID map: Uuid -> "type_N"
    let id_map = build_id_map(graph);

    // Build connection lookup: from_port -> to_node
    let conn_map = build_connection_map(graph);

    let variables = graph
        .variables
        .iter()
        .map(|v| ExportedVariable {
            name: v.name.clone(),
            var_type: format!("{:?}", v.var_type).to_lowercase(),
            default: variable_value_to_json(&v.default_value),
        })
        .collect();

    let characters = graph
        .characters
        .iter()
        .map(|c| ExportedCharacter {
            id: id_map
                .get(&c.id)
                .cloned()
                .unwrap_or_else(|| c.id.to_string()),
            name: c.name.clone(),
            portrait: if c.portrait_path.is_empty() {
                None
            } else {
                Some(c.portrait_path.clone())
            },
            color: format!(
                "#{:02X}{:02X}{:02X}",
                c.color[0], c.color[1], c.color[2]
            ),
        })
        .collect();

    let mut nodes = Vec::new();
    for node in graph.nodes.values() {
        let readable_id = id_map
            .get(&node.id)
            .cloned()
            .unwrap_or_else(|| node.id.to_string());

        let (type_str, data) = match &node.node_type {
            NodeType::Start => {
                let next = find_next_single(&node.outputs, &conn_map, &id_map);
                let data = serde_json::json!({ "next": next });
                ("start", data)
            }
            NodeType::Dialogue(d) => {
                let next = find_next_single(&node.outputs, &conn_map, &id_map);
                let speaker = if let Some(sid) = d.speaker_id {
                    id_map.get(&sid).cloned().unwrap_or_default()
                } else {
                    d.speaker_name.clone()
                };
                let data = serde_json::json!({
                    "speaker": speaker,
                    "text": d.text,
                    "emotion": d.emotion,
                    "portrait": d.portrait_override,
                    "audio": d.audio_clip,
                    "next": next,
                });
                ("dialogue", data)
            }
            NodeType::Choice(d) => {
                let options: Vec<serde_json::Value> = d
                    .choices
                    .iter()
                    .enumerate()
                    .map(|(i, choice)| {
                        let next = node
                            .outputs
                            .get(i)
                            .and_then(|port| conn_map.get(&port.id.0))
                            .and_then(|target| id_map.get(target))
                            .cloned();
                        let mut obj = serde_json::json!({
                            "text": choice.text,
                            "next": next,
                        });
                        if let Some(cond) = &choice.condition {
                            obj["condition"] = serde_json::json!({
                                "variable": cond.variable_name,
                                "operator": compare_op_str(cond.operator),
                                "value": variable_value_to_json(&cond.value),
                            });
                        } else {
                            obj["condition"] = serde_json::Value::Null;
                        }
                        obj
                    })
                    .collect();
                let data = serde_json::json!({
                    "prompt": d.prompt,
                    "options": options,
                });
                ("choice", data)
            }
            NodeType::Condition(d) => {
                let true_next = node
                    .outputs
                    .first()
                    .and_then(|port| conn_map.get(&port.id.0))
                    .and_then(|target| id_map.get(target))
                    .cloned();
                let false_next = node
                    .outputs
                    .get(1)
                    .and_then(|port| conn_map.get(&port.id.0))
                    .and_then(|target| id_map.get(target))
                    .cloned();
                let data = serde_json::json!({
                    "variable": d.variable_name,
                    "operator": compare_op_str(d.operator),
                    "value": variable_value_to_json(&d.value),
                    "true_next": true_next,
                    "false_next": false_next,
                });
                ("condition", data)
            }
            NodeType::Event(d) => {
                let next = find_next_single(&node.outputs, &conn_map, &id_map);
                let actions: Vec<serde_json::Value> = d
                    .actions
                    .iter()
                    .map(|a| {
                        serde_json::json!({
                            "action": format!("{:?}", a.action_type).to_lowercase(),
                            "key": a.key,
                            "value": variable_value_to_json(&a.value),
                        })
                    })
                    .collect();
                let data = serde_json::json!({
                    "actions": actions,
                    "next": next,
                });
                ("event", data)
            }
            NodeType::Random(d) => {
                let branches: Vec<serde_json::Value> = d
                    .branches
                    .iter()
                    .enumerate()
                    .map(|(i, branch)| {
                        let next = node
                            .outputs
                            .get(i)
                            .and_then(|port| conn_map.get(&port.id.0))
                            .and_then(|target| id_map.get(target))
                            .cloned();
                        serde_json::json!({
                            "weight": branch.weight,
                            "next": next,
                        })
                    })
                    .collect();
                let data = serde_json::json!({ "branches": branches });
                ("random", data)
            }
            NodeType::End(d) => {
                let data = serde_json::json!({ "tag": d.tag });
                ("end", data)
            }
        };

        nodes.push(ExportedNode {
            id: readable_id,
            node_type: type_str.to_string(),
            data,
        });
    }

    ExportedDialogue {
        version: "1.0".to_string(),
        name: name.to_string(),
        variables,
        characters,
        nodes,
    }
}

/// Build a map of Uuid -> readable ID like "start_1", "dlg_2", etc.
pub fn build_id_map(graph: &DialogueGraph) -> HashMap<Uuid, String> {
    let mut map = HashMap::new();
    let mut counters: HashMap<&str, usize> = HashMap::new();

    // Sort nodes by position for deterministic ordering
    let mut sorted_nodes: Vec<_> = graph.nodes.values().collect();
    sorted_nodes.sort_by(|a, b| {
        a.position[1]
            .partial_cmp(&b.position[1])
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(
                a.position[0]
                    .partial_cmp(&b.position[0])
                    .unwrap_or(std::cmp::Ordering::Equal),
            )
    });

    for node in &sorted_nodes {
        let prefix = match &node.node_type {
            NodeType::Start => "start",
            NodeType::Dialogue(_) => "dlg",
            NodeType::Choice(_) => "choice",
            NodeType::Condition(_) => "cond",
            NodeType::Event(_) => "evt",
            NodeType::Random(_) => "rand",
            NodeType::End(_) => "end",
        };
        let counter = counters.entry(prefix).or_insert(0);
        *counter += 1;
        map.insert(node.id, format!("{prefix}_{counter}"));
    }

    // Also map character IDs
    for (i, ch) in graph.characters.iter().enumerate() {
        map.insert(ch.id, format!("char_{}", i + 1));
    }

    map
}

/// Build a lookup: output port Uuid -> target node Uuid.
fn build_connection_map(graph: &DialogueGraph) -> HashMap<Uuid, Uuid> {
    graph
        .connections
        .iter()
        .map(|c| (c.from_port.0, c.to_node))
        .collect()
}

fn find_next_single(
    outputs: &[crate::model::port::Port],
    conn_map: &HashMap<Uuid, Uuid>,
    id_map: &HashMap<Uuid, String>,
) -> Option<String> {
    outputs
        .first()
        .and_then(|port| conn_map.get(&port.id.0))
        .and_then(|target| id_map.get(target))
        .cloned()
}

fn compare_op_str(op: CompareOp) -> &'static str {
    match op {
        CompareOp::Eq => "==",
        CompareOp::Neq => "!=",
        CompareOp::Gt => ">",
        CompareOp::Lt => "<",
        CompareOp::Gte => ">=",
        CompareOp::Lte => "<=",
        CompareOp::Contains => "contains",
    }
}

fn variable_value_to_json(val: &VariableValue) -> serde_json::Value {
    match val {
        VariableValue::Bool(b) => serde_json::Value::Bool(*b),
        VariableValue::Int(i) => serde_json::json!(i),
        VariableValue::Float(f) => serde_json::json!(f),
        VariableValue::Text(s) => serde_json::Value::String(s.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::node::Node;

    #[test]
    fn export_simple_graph() {
        let mut graph = DialogueGraph::new();
        let start = Node::new_start([0.0, 0.0]);
        let dlg = Node::new_dialogue([200.0, 0.0]);
        let end = Node::new_end([400.0, 0.0]);

        let start_out = start.outputs[0].id;
        let dlg_in = dlg.inputs[0].id;
        let dlg_out = dlg.outputs[0].id;
        let end_in = end.inputs[0].id;
        let start_id = start.id;
        let dlg_id = dlg.id;
        let end_id = end.id;

        graph.add_node(start);
        graph.add_node(dlg);
        graph.add_node(end);
        graph.add_connection(start_id, start_out, dlg_id, dlg_in);
        graph.add_connection(dlg_id, dlg_out, end_id, end_in);

        let json = export_json(&graph, "test").unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["version"], "1.0");
        assert_eq!(parsed["name"], "test");
        assert_eq!(parsed["nodes"].as_array().unwrap().len(), 3);

        // Check that connections are baked into next fields
        let nodes = parsed["nodes"].as_array().unwrap();
        let start_node = nodes.iter().find(|n| n["type"] == "start").unwrap();
        assert!(start_node["next"].as_str().is_some());
    }

    #[test]
    fn readable_ids_are_deterministic() {
        let mut graph = DialogueGraph::new();
        graph.add_node(Node::new_start([0.0, 0.0]));
        graph.add_node(Node::new_dialogue([0.0, 100.0]));
        graph.add_node(Node::new_dialogue([0.0, 200.0]));

        let id_map = build_id_map(&graph);
        let values: Vec<&String> = id_map.values().collect();
        assert!(values.contains(&&"start_1".to_string()));
        assert!(values.contains(&&"dlg_1".to_string()));
        assert!(values.contains(&&"dlg_2".to_string()));
    }

    #[test]
    fn readable_id_prefixes_all_types() {
        let mut graph = DialogueGraph::new();
        graph.add_node(Node::new_start([0.0, 0.0]));
        graph.add_node(Node::new_dialogue([0.0, 100.0]));
        graph.add_node(Node::new_choice([0.0, 200.0]));
        graph.add_node(Node::new_condition([0.0, 300.0]));
        graph.add_node(Node::new_event([0.0, 400.0]));
        graph.add_node(Node::new_random([0.0, 500.0]));
        graph.add_node(Node::new_end([0.0, 600.0]));

        let id_map = build_id_map(&graph);
        let values: Vec<String> = id_map.values().cloned().collect();
        assert!(values.contains(&"start_1".to_string()));
        assert!(values.contains(&"dlg_1".to_string()));
        assert!(values.contains(&"choice_1".to_string()));
        assert!(values.contains(&"cond_1".to_string()));
        assert!(values.contains(&"evt_1".to_string()));
        assert!(values.contains(&"rand_1".to_string()));
        assert!(values.contains(&"end_1".to_string()));
    }

    #[test]
    fn export_choice_node() {
        let mut graph = DialogueGraph::new();
        let start = Node::new_start([0.0, 0.0]);
        let choice = Node::new_choice([200.0, 0.0]);
        let end1 = Node::new_end([400.0, 0.0]);
        let end2 = Node::new_end([400.0, 100.0]);

        let s_out = start.outputs[0].id;
        let c_in = choice.inputs[0].id;
        let c_out0 = choice.outputs[0].id;
        let c_out1 = choice.outputs[1].id;
        let e1_in = end1.inputs[0].id;
        let e2_in = end2.inputs[0].id;
        let s_id = start.id;
        let c_id = choice.id;
        let e1_id = end1.id;
        let e2_id = end2.id;

        graph.add_node(start);
        graph.add_node(choice);
        graph.add_node(end1);
        graph.add_node(end2);
        graph.add_connection(s_id, s_out, c_id, c_in);
        graph.add_connection(c_id, c_out0, e1_id, e1_in);
        graph.add_connection(c_id, c_out1, e2_id, e2_in);

        let json = export_json(&graph, "test").unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        let nodes = parsed["nodes"].as_array().unwrap();
        let choice_node = nodes.iter().find(|n| n["type"] == "choice").unwrap();
        let options = choice_node["options"].as_array().unwrap();
        assert_eq!(options.len(), 2);
        assert!(options[0]["next"].as_str().is_some());
        assert!(options[1]["next"].as_str().is_some());
    }

    #[test]
    fn export_condition_node() {
        use crate::model::node::NodeType;

        let mut graph = DialogueGraph::new();
        let start = Node::new_start([0.0, 0.0]);
        let mut cond = Node::new_condition([200.0, 0.0]);
        if let NodeType::Condition(ref mut d) = cond.node_type {
            d.variable_name = "gold".to_string();
            d.operator = CompareOp::Gte;
            d.value = VariableValue::Int(100);
        }
        let end_t = Node::new_end([400.0, 0.0]);
        let end_f = Node::new_end([400.0, 100.0]);

        let s_out = start.outputs[0].id;
        let c_in = cond.inputs[0].id;
        let c_true = cond.outputs[0].id;
        let c_false = cond.outputs[1].id;
        let et_in = end_t.inputs[0].id;
        let ef_in = end_f.inputs[0].id;
        let s_id = start.id;
        let c_id = cond.id;
        let et_id = end_t.id;
        let ef_id = end_f.id;

        graph.add_node(start);
        graph.add_node(cond);
        graph.add_node(end_t);
        graph.add_node(end_f);
        graph.add_connection(s_id, s_out, c_id, c_in);
        graph.add_connection(c_id, c_true, et_id, et_in);
        graph.add_connection(c_id, c_false, ef_id, ef_in);

        let json = export_json(&graph, "test").unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        let nodes = parsed["nodes"].as_array().unwrap();
        let cond_node = nodes.iter().find(|n| n["type"] == "condition").unwrap();
        assert_eq!(cond_node["variable"], "gold");
        assert_eq!(cond_node["operator"], ">=");
        assert_eq!(cond_node["value"], 100);
        assert!(cond_node["true_next"].as_str().is_some());
        assert!(cond_node["false_next"].as_str().is_some());
    }

    #[test]
    fn export_event_node() {
        use crate::model::node::{EventAction, EventActionType, NodeType};

        let mut graph = DialogueGraph::new();
        let start = Node::new_start([0.0, 0.0]);
        let mut evt = Node::new_event([200.0, 0.0]);
        if let NodeType::Event(ref mut e) = evt.node_type {
            e.actions.push(EventAction {
                action_type: EventActionType::SetVariable,
                key: "has_key".to_string(),
                value: VariableValue::Bool(true),
            });
        }
        let end = Node::new_end([400.0, 0.0]);

        let s_out = start.outputs[0].id;
        let e_in = evt.inputs[0].id;
        let e_out = evt.outputs[0].id;
        let end_in = end.inputs[0].id;
        let s_id = start.id;
        let e_id = evt.id;
        let end_id = end.id;

        graph.add_node(start);
        graph.add_node(evt);
        graph.add_node(end);
        graph.add_connection(s_id, s_out, e_id, e_in);
        graph.add_connection(e_id, e_out, end_id, end_in);

        let json = export_json(&graph, "test").unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        let nodes = parsed["nodes"].as_array().unwrap();
        let evt_node = nodes.iter().find(|n| n["type"] == "event").unwrap();
        let actions = evt_node["actions"].as_array().unwrap();
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0]["key"], "has_key");
        assert_eq!(actions[0]["value"], true);
    }

    #[test]
    fn export_random_node() {
        let mut graph = DialogueGraph::new();
        let start = Node::new_start([0.0, 0.0]);
        let rand = Node::new_random([200.0, 0.0]);
        let end1 = Node::new_end([400.0, 0.0]);
        let end2 = Node::new_end([400.0, 100.0]);

        let s_out = start.outputs[0].id;
        let r_in = rand.inputs[0].id;
        let r_out0 = rand.outputs[0].id;
        let r_out1 = rand.outputs[1].id;
        let e1_in = end1.inputs[0].id;
        let e2_in = end2.inputs[0].id;
        let s_id = start.id;
        let r_id = rand.id;
        let e1_id = end1.id;
        let e2_id = end2.id;

        graph.add_node(start);
        graph.add_node(rand);
        graph.add_node(end1);
        graph.add_node(end2);
        graph.add_connection(s_id, s_out, r_id, r_in);
        graph.add_connection(r_id, r_out0, e1_id, e1_in);
        graph.add_connection(r_id, r_out1, e2_id, e2_in);

        let json = export_json(&graph, "test").unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        let nodes = parsed["nodes"].as_array().unwrap();
        let rand_node = nodes.iter().find(|n| n["type"] == "random").unwrap();
        let branches = rand_node["branches"].as_array().unwrap();
        assert_eq!(branches.len(), 2);
        assert_eq!(branches[0]["weight"], 0.5);
        assert!(branches[0]["next"].as_str().is_some());
    }

    #[test]
    fn export_unconnected_output_is_null() {
        let mut graph = DialogueGraph::new();
        let start = Node::new_start([0.0, 0.0]);
        graph.add_node(start);

        let json = export_json(&graph, "test").unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        let nodes = parsed["nodes"].as_array().unwrap();
        let start_node = nodes.iter().find(|n| n["type"] == "start").unwrap();
        assert!(start_node["next"].is_null());
    }

    #[test]
    fn export_empty_graph() {
        let graph = DialogueGraph::new();
        let json = export_json(&graph, "empty").unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["version"], "1.0");
        assert_eq!(parsed["name"], "empty");
        assert_eq!(parsed["nodes"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn export_with_variables() {
        use crate::model::variable::Variable;

        let mut graph = DialogueGraph::new();
        graph.variables.push(Variable::new_bool("flag", true));
        graph.variables.push(Variable::new_int("score", 99));

        let json = export_json(&graph, "test").unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        let vars = parsed["variables"].as_array().unwrap();
        assert_eq!(vars.len(), 2);
        assert_eq!(vars[0]["name"], "flag");
        assert_eq!(vars[0]["default"], true);
        assert_eq!(vars[1]["name"], "score");
        assert_eq!(vars[1]["default"], 99);
    }

    #[test]
    fn export_with_characters() {
        use crate::model::character::Character;

        let mut graph = DialogueGraph::new();
        let ch = Character::new("Hero");
        graph.characters.push(ch);

        let json = export_json(&graph, "test").unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        let chars = parsed["characters"].as_array().unwrap();
        assert_eq!(chars.len(), 1);
        assert_eq!(chars[0]["name"], "Hero");
        assert_eq!(chars[0]["id"], "char_1");
        // Color should be hex format
        let color = chars[0]["color"].as_str().unwrap();
        assert!(color.starts_with('#'));
        assert_eq!(color.len(), 7);
    }

    #[test]
    fn export_no_positions_in_output() {
        let mut graph = DialogueGraph::new();
        graph.add_node(Node::new_start([123.0, 456.0]));

        let json = export_json(&graph, "test").unwrap();
        assert!(!json.contains("position"));
        assert!(!json.contains("123"));
        assert!(!json.contains("456"));
    }
}
