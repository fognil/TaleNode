use std::collections::HashMap;
use uuid::Uuid;

use crate::model::graph::DialogueGraph;
use crate::model::locale::collect_translatable_strings;
use crate::model::node::{CompareOp, NodeType, VariableValue};

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
            NodeType::SubGraph(_) => "sub",
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
pub fn build_connection_map(graph: &DialogueGraph) -> HashMap<Uuid, Uuid> {
    graph
        .connections
        .iter()
        .map(|c| (c.from_port.0, c.to_node))
        .collect()
}

pub fn find_next_single(
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

pub fn compare_op_str(op: CompareOp) -> &'static str {
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

pub fn variable_value_to_json(val: &VariableValue) -> serde_json::Value {
    match val {
        VariableValue::Bool(b) => serde_json::Value::Bool(*b),
        VariableValue::Int(i) => serde_json::json!(i),
        VariableValue::Float(f) => serde_json::json!(f),
        VariableValue::Text(s) => serde_json::Value::String(s.clone()),
    }
}

pub fn export_node_data(
    node_type: &NodeType,
    outputs: &[crate::model::port::Port],
    conn_map: &HashMap<Uuid, Uuid>,
    id_map: &HashMap<Uuid, String>,
) -> (&'static str, serde_json::Value) {
    match node_type {
        NodeType::Start => {
            let next = find_next_single(outputs, conn_map, id_map);
            ("start", serde_json::json!({ "next": next }))
        }
        NodeType::Dialogue(d) => {
            let next = find_next_single(outputs, conn_map, id_map);
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
                    let next = outputs
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
            let true_next = outputs
                .first()
                .and_then(|port| conn_map.get(&port.id.0))
                .and_then(|target| id_map.get(target))
                .cloned();
            let false_next = outputs
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
            let next = find_next_single(outputs, conn_map, id_map);
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
                    let next = outputs
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
            ("random", serde_json::json!({ "branches": branches }))
        }
        NodeType::End(d) => ("end", serde_json::json!({ "tag": &d.tag })),
        NodeType::SubGraph(d) => {
            let next = find_next_single(outputs, conn_map, id_map);
            ("subgraph", serde_json::json!({
                "name": d.name,
                "next": next,
            }))
        }
    }
}

/// Build the string table for the exported JSON.
/// Maps readable IDs to { locale -> text } for all translatable strings.
pub fn build_string_table(
    graph: &DialogueGraph,
    id_map: &HashMap<Uuid, String>,
) -> serde_json::Value {
    let locale = &graph.locale;
    let strings = collect_translatable_strings(graph);
    let mut table = serde_json::Map::new();

    for ts in &strings {
        // Map UUID-based key to readable key using node's readable id
        let readable_prefix = id_map
            .get(&ts.node_id)
            .cloned()
            .unwrap_or_else(|| ts.node_id.to_string());
        let readable_key = match ts.string_type {
            crate::model::locale::StringType::Dialogue => readable_prefix,
            crate::model::locale::StringType::ChoicePrompt => {
                format!("{readable_prefix}_prompt")
            }
            crate::model::locale::StringType::ChoiceOption => {
                // Extract the index suffix from the UUID-based key (e.g. "opt_abcd1234_0" -> "0")
                let suffix = ts.key.rsplit('_').next().unwrap_or("0");
                format!("{readable_prefix}_opt_{suffix}")
            }
        };

        let mut locale_map = serde_json::Map::new();
        // Default locale text
        locale_map.insert(
            locale.default_locale.clone(),
            serde_json::Value::String(ts.default_text.clone()),
        );
        // Extra locale translations
        for loc in &locale.extra_locales {
            let text = locale
                .get_translation(&ts.key, loc)
                .unwrap_or("")
                .to_string();
            locale_map.insert(loc.clone(), serde_json::Value::String(text));
        }
        table.insert(readable_key, serde_json::Value::Object(locale_map));
    }

    serde_json::Value::Object(table)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::node::Node;

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
}
