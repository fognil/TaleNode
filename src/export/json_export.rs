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
fn build_id_map(graph: &DialogueGraph) -> HashMap<Uuid, String> {
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
}
