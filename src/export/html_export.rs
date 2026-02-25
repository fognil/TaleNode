use std::collections::HashMap;
use uuid::Uuid;

use crate::model::graph::DialogueGraph;

use super::flatten::flatten_subgraphs;
use super::html_template::{CSS, JS_ENGINE};
use super::json_export_helpers::{
    build_connection_map, build_id_map, export_node_data, variable_value_to_json,
};

/// Export a self-contained HTML file that plays the dialogue in a browser.
pub fn export_html(graph: &DialogueGraph, name: &str) -> String {
    let flat = flatten_subgraphs(graph);
    let id_map = build_id_map(&flat);
    let conn_map = build_connection_map(&flat);

    let graph_json = build_graph_json(&flat, &id_map, &conn_map);
    let characters_json = build_characters_json(&flat);
    let variables_json = build_variables_json(&flat);

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>{title} — TaleNode Playable</title>
{css}
</head>
<body>
<div id="app">
  <header><h1>{title}</h1><button id="restart-btn">Restart</button></header>
  <div id="main">
    <div id="chat-log"></div>
    <div id="choices"></div>
  </div>
  <aside id="sidebar">
    <h3 onclick="toggleSidebar()">Variables ▾</h3>
    <div id="var-list"></div>
  </aside>
</div>
<script>
const GRAPH = {graph_json};
const CHARACTERS = {characters_json};
const INIT_VARS = {variables_json};
{js}
</script>
</body>
</html>"#,
        title = html_escape(name),
        css = CSS,
        graph_json = graph_json,
        characters_json = characters_json,
        variables_json = variables_json,
        js = JS_ENGINE,
    )
}

fn build_graph_json(
    graph: &DialogueGraph,
    id_map: &HashMap<Uuid, String>,
    conn_map: &HashMap<Uuid, Uuid>,
) -> String {
    let mut nodes = serde_json::Map::new();

    let mut sorted: Vec<_> = graph.nodes.values().collect();
    sorted.sort_by(|a, b| {
        a.position[1]
            .partial_cmp(&b.position[1])
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    for node in sorted {
        let Some(readable_id) = id_map.get(&node.id) else {
            continue;
        };
        let (type_str, data) =
            export_node_data(&node.node_type, &node.outputs, conn_map, id_map);
        let mut obj = serde_json::Map::new();
        obj.insert("type".to_string(), serde_json::json!(type_str));
        if let serde_json::Value::Object(map) = data {
            for (k, v) in map {
                obj.insert(k, v);
            }
        }
        nodes.insert(readable_id.clone(), serde_json::Value::Object(obj));
    }

    serde_json::to_string(&serde_json::Value::Object(nodes)).unwrap_or_default()
}

fn build_characters_json(graph: &DialogueGraph) -> String {
    let mut chars = serde_json::Map::new();
    for ch in &graph.characters {
        let color = format!(
            "rgb({},{},{})",
            ch.color[0], ch.color[1], ch.color[2]
        );
        chars.insert(ch.name.clone(), serde_json::json!({ "color": color }));
    }
    serde_json::to_string(&serde_json::Value::Object(chars)).unwrap_or_default()
}

fn build_variables_json(graph: &DialogueGraph) -> String {
    let mut vars = serde_json::Map::new();
    for v in &graph.variables {
        vars.insert(v.name.clone(), variable_value_to_json(&v.default_value));
    }
    serde_json::to_string(&serde_json::Value::Object(vars)).unwrap_or_default()
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::node::{Node, NodeType};

    #[test]
    fn html_export_has_structure() {
        let mut graph = DialogueGraph::new();
        graph.add_node(Node::new_start([0.0, 0.0]));

        let html = export_html(&graph, "Test Dialogue");
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("Test Dialogue"));
        assert!(html.contains("const GRAPH"));
        assert!(html.contains("function start()"));
    }

    #[test]
    fn html_export_contains_nodes() {
        let mut graph = DialogueGraph::new();
        let start = Node::new_start([0.0, 0.0]);
        let s_id = start.id;
        let s_out = start.outputs[0].id;

        let mut dlg = Node::new_dialogue([0.0, 200.0]);
        if let NodeType::Dialogue(ref mut data) = dlg.node_type {
            data.speaker_name = "Guard".to_string();
            data.text = "Hello there!".to_string();
        }
        let d_id = dlg.id;
        let d_in = dlg.inputs[0].id;

        graph.add_node(start);
        graph.add_node(dlg);
        graph.add_connection(s_id, s_out, d_id, d_in);

        let html = export_html(&graph, "Test");
        assert!(html.contains("Guard"));
        assert!(html.contains("Hello there!"));
    }

    #[test]
    fn html_export_escapes_title() {
        let graph = DialogueGraph::new();
        let html = export_html(&graph, "Test <script>alert(1)</script>");
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn html_export_includes_variables() {
        let mut graph = DialogueGraph::new();
        graph.add_node(Node::new_start([0.0, 0.0]));
        graph.variables.push(crate::model::variable::Variable {
            id: uuid::Uuid::new_v4(),
            name: "gold".to_string(),
            var_type: crate::model::variable::VariableType::Int,
            default_value: crate::model::node::VariableValue::Int(100),
        });

        let html = export_html(&graph, "Test");
        assert!(html.contains("\"gold\""));
        assert!(html.contains("100"));
    }

    #[test]
    fn html_export_empty_graph() {
        let graph = DialogueGraph::new();
        let html = export_html(&graph, "Empty");
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("const GRAPH"));
    }
}
