use crate::model::graph::DialogueGraph;

use super::json_export_helpers::{build_bark_groups, build_connection_map, build_id_map,
    build_string_table, export_node_data, variable_value_to_json};
use super::json_export_types::*;

/// Export a DialogueGraph to the game-engine JSON format.
/// SubGraph nodes are flattened (inlined) so the output is always a flat node array.
pub fn export_json(graph: &DialogueGraph, name: &str) -> Result<String, serde_json::Error> {
    let flat = super::flatten::flatten_subgraphs(graph);
    let exported = build_export(&flat, name);
    serde_json::to_string_pretty(&exported)
}

fn build_export(graph: &DialogueGraph, name: &str) -> ExportedDialogue {
    let id_map = build_id_map(graph);
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

        let (type_str, data) =
            export_node_data(&node.node_type, &node.outputs, &conn_map, &id_map);

        nodes.push(ExportedNode {
            id: readable_id,
            node_type: type_str.to_string(),
            data,
        });
    }

    let barks = build_bark_groups(graph);
    let has_locales = graph.locale.has_extra_locales();

    ExportedDialogue {
        version: "1.0".to_string(),
        name: name.to_string(),
        default_locale: if has_locales {
            Some(graph.locale.default_locale.clone())
        } else {
            None
        },
        locales: if has_locales {
            let mut all = vec![graph.locale.default_locale.clone()];
            all.extend(graph.locale.extra_locales.clone());
            Some(all)
        } else {
            None
        },
        variables,
        characters,
        nodes,
        strings: if has_locales {
            Some(build_string_table(graph, &id_map))
        } else {
            None
        },
        barks,
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

        let nodes = parsed["nodes"].as_array().unwrap();
        let start_node = nodes.iter().find(|n| n["type"] == "start").unwrap();
        assert!(start_node["next"].as_str().is_some());
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
        use crate::model::node::{CompareOp, NodeType, VariableValue};

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
        use crate::model::node::{EventAction, EventActionType, NodeType, VariableValue};

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
        graph.add_node(Node::new_start([0.0, 0.0]));

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
        let color = chars[0]["color"].as_str().unwrap();
        assert!(color.starts_with('#'));
        assert_eq!(color.len(), 7);
    }

    #[test]
    fn export_with_locales_has_string_table() {
        use crate::model::node::NodeType;

        let mut graph = DialogueGraph::new();
        let mut dlg = Node::new_dialogue([0.0, 100.0]);
        let dlg_uuid8 = dlg.id.to_string()[..8].to_string();
        if let NodeType::Dialogue(ref mut d) = dlg.node_type {
            d.text = "Hello!".to_string();
        }
        graph.add_node(dlg);
        graph.locale.add_locale("fr".to_string());
        graph.locale.set_translation(
            format!("dlg_{dlg_uuid8}"),
            "fr".to_string(),
            "Bonjour!".to_string(),
        );

        let json = export_json(&graph, "test").unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["default_locale"], "en");
        assert!(parsed["locales"].as_array().unwrap().contains(&"fr".into()));
        assert!(parsed["strings"].is_object());
        // Find the string entry (readable id = dlg_1)
        let dlg_entry = &parsed["strings"]["dlg_1"];
        assert_eq!(dlg_entry["en"], "Hello!");
        assert_eq!(dlg_entry["fr"], "Bonjour!");
    }

    #[test]
    fn export_without_locales_omits_string_table() {
        let json = export_json(&DialogueGraph::new(), "test").unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed.get("default_locale").is_none());
        assert!(parsed.get("strings").is_none());
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
