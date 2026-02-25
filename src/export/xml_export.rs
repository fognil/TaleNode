use crate::model::graph::DialogueGraph;
use crate::model::node::{NodeType, VariableValue};

use super::json_export_helpers::{
    build_connection_map, build_id_map, compare_op_str, find_next_single,
};

/// Escape special XML characters.
fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn variable_value_str(val: &VariableValue) -> String {
    match val {
        VariableValue::Bool(b) => b.to_string(),
        VariableValue::Int(i) => i.to_string(),
        VariableValue::Float(f) => f.to_string(),
        VariableValue::Text(s) => xml_escape(s),
    }
}

fn variable_type_str(val: &VariableValue) -> &'static str {
    match val {
        VariableValue::Bool(_) => "bool",
        VariableValue::Int(_) => "int",
        VariableValue::Float(_) => "float",
        VariableValue::Text(_) => "string",
    }
}

/// Export a DialogueGraph to XML format.
/// SubGraph nodes are flattened (inlined) before export.
pub fn export_xml(graph: &DialogueGraph, name: &str) -> Result<String, String> {
    let flat = super::flatten::flatten_subgraphs(graph);
    let id_map = build_id_map(&flat);
    let conn_map = build_connection_map(&flat);
    let graph = &flat;

    let mut xml = String::with_capacity(4096);
    xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str(&format!("<dialogue version=\"1.0\" name=\"{}\">\n", xml_escape(name)));

    write_variables(graph, &mut xml);
    write_characters(graph, &id_map, &mut xml);
    if flat.locale.has_extra_locales() {
        write_localization(&flat, &id_map, &mut xml);
    }
    write_nodes(graph, &id_map, &conn_map, &mut xml);

    xml.push_str("</dialogue>\n");
    Ok(xml)
}

fn write_variables(graph: &DialogueGraph, xml: &mut String) {
    if graph.variables.is_empty() {
        xml.push_str("  <variables/>\n");
        return;
    }
    xml.push_str("  <variables>\n");
    for var in &graph.variables {
        let vtype = format!("{:?}", var.var_type).to_lowercase();
        xml.push_str(&format!(
            "    <variable name=\"{}\" type=\"{}\" default=\"{}\"/>\n",
            xml_escape(&var.name),
            vtype,
            variable_value_str(&var.default_value),
        ));
    }
    xml.push_str("  </variables>\n");
}

fn write_characters(
    graph: &DialogueGraph,
    id_map: &std::collections::HashMap<uuid::Uuid, String>,
    xml: &mut String,
) {
    if graph.characters.is_empty() {
        xml.push_str("  <characters/>\n");
        return;
    }
    xml.push_str("  <characters>\n");
    for ch in &graph.characters {
        let readable_id = id_map
            .get(&ch.id)
            .cloned()
            .unwrap_or_else(|| ch.id.to_string());
        let color = format!("#{:02X}{:02X}{:02X}", ch.color[0], ch.color[1], ch.color[2]);
        xml.push_str(&format!(
            "    <character id=\"{}\" name=\"{}\" color=\"{}\"",
            readable_id,
            xml_escape(&ch.name),
            color,
        ));
        if ch.portrait_path.is_empty() {
            xml.push_str("/>\n");
        } else {
            xml.push_str(&format!(
                " portrait=\"{}\"/>\n",
                xml_escape(&ch.portrait_path)
            ));
        }
    }
    xml.push_str("  </characters>\n");
}

fn write_localization(
    graph: &DialogueGraph,
    id_map: &std::collections::HashMap<uuid::Uuid, String>,
    xml: &mut String,
) {
    let strings = crate::model::locale::collect_translatable_strings(graph);
    let dl = &graph.locale.default_locale;
    xml.push_str(&format!("  <localization default=\"{}\">\n", xml_escape(dl)));
    for loc in &graph.locale.extra_locales {
        xml.push_str(&format!("    <locale code=\"{}\"/>\n", xml_escape(loc)));
    }
    for ts in &strings {
        let rk = super::json_export_helpers::readable_key_for_locale(&ts.key, ts.node_id, id_map);
        xml.push_str(&format!("    <string key=\"{}\">\n", xml_escape(&rk)));
        xml.push_str(&format!("      <text locale=\"{}\">{}</text>\n", xml_escape(dl), xml_escape(&ts.default_text)));
        for loc in &graph.locale.extra_locales {
            let t = graph.locale.get_translation(&ts.key, loc).unwrap_or("");
            xml.push_str(&format!("      <text locale=\"{}\">{}</text>\n", xml_escape(loc), xml_escape(t)));
        }
        xml.push_str("    </string>\n");
    }
    xml.push_str("  </localization>\n");
}

fn write_nodes(
    graph: &DialogueGraph,
    id_map: &std::collections::HashMap<uuid::Uuid, String>,
    conn_map: &std::collections::HashMap<uuid::Uuid, uuid::Uuid>,
    xml: &mut String,
) {
    if graph.nodes.is_empty() {
        xml.push_str("  <nodes/>\n");
        return;
    }
    xml.push_str("  <nodes>\n");
    // Sort for deterministic output
    let mut sorted: Vec<_> = graph.nodes.values().collect();
    sorted.sort_by(|a, b| {
        a.position[1]
            .partial_cmp(&b.position[1])
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(
                a.position[0]
                    .partial_cmp(&b.position[0])
                    .unwrap_or(std::cmp::Ordering::Equal),
            )
    });

    for node in sorted {
        let rid = id_map.get(&node.id).cloned().unwrap_or_default();
        write_single_node(node, &rid, id_map, conn_map, xml);
    }
    xml.push_str("  </nodes>\n");
}

fn write_single_node(
    node: &crate::model::node::Node,
    rid: &str,
    id_map: &std::collections::HashMap<uuid::Uuid, String>,
    conn_map: &std::collections::HashMap<uuid::Uuid, uuid::Uuid>,
    xml: &mut String,
) {
    match &node.node_type {
        NodeType::Start => {
            let next = find_next_single(&node.outputs, conn_map, id_map);
            let next_attr = next.as_deref().unwrap_or("");
            xml.push_str(&format!(
                "    <node id=\"{rid}\" type=\"start\" next=\"{next_attr}\"/>\n"
            ));
        }
        NodeType::Dialogue(d) => write_dialogue_node(rid, d, node, id_map, conn_map, xml),
        NodeType::Choice(d) => write_choice_node(rid, d, node, id_map, conn_map, xml),
        NodeType::Condition(d) => write_condition_node(rid, d, node, id_map, conn_map, xml),
        NodeType::Event(d) => write_event_node(rid, d, node, id_map, conn_map, xml),
        NodeType::Random(d) => write_random_node(rid, d, node, id_map, conn_map, xml),
        NodeType::End(d) => {
            xml.push_str(&format!(
                "    <node id=\"{rid}\" type=\"end\" tag=\"{}\"/>\n",
                xml_escape(&d.tag)
            ));
        }
        NodeType::SubGraph(d) => {
            let next = find_next_single(&node.outputs, conn_map, id_map);
            let next_attr = next.as_deref().unwrap_or("");
            xml.push_str(&format!(
                "    <node id=\"{rid}\" type=\"subgraph\" name=\"{}\" next=\"{next_attr}\"/>\n",
                xml_escape(&d.name)
            ));
        }
    }
}

fn write_dialogue_node(
    rid: &str,
    d: &crate::model::node::DialogueData,
    node: &crate::model::node::Node,
    id_map: &std::collections::HashMap<uuid::Uuid, String>,
    conn_map: &std::collections::HashMap<uuid::Uuid, uuid::Uuid>,
    xml: &mut String,
) {
    let next = find_next_single(&node.outputs, conn_map, id_map);
    let speaker = if let Some(sid) = d.speaker_id {
        id_map.get(&sid).cloned().unwrap_or_default()
    } else {
        xml_escape(&d.speaker_name)
    };
    xml.push_str(&format!(
        "    <node id=\"{rid}\" type=\"dialogue\" speaker=\"{}\" emotion=\"{}\"",
        speaker,
        xml_escape(&d.emotion),
    ));
    if let Some(ref audio) = d.audio_clip {
        if !audio.is_empty() {
            xml.push_str(&format!(" audio=\"{}\"", xml_escape(audio)));
        }
    }
    if let Some(ref n) = next {
        xml.push_str(&format!(" next=\"{n}\""));
    }
    xml.push_str(&format!(">{}</node>\n", xml_escape(&d.text)));
}

fn write_choice_node(
    rid: &str,
    d: &crate::model::node::ChoiceData,
    node: &crate::model::node::Node,
    id_map: &std::collections::HashMap<uuid::Uuid, String>,
    conn_map: &std::collections::HashMap<uuid::Uuid, uuid::Uuid>,
    xml: &mut String,
) {
    xml.push_str(&format!(
        "    <node id=\"{rid}\" type=\"choice\" prompt=\"{}\">\n",
        xml_escape(&d.prompt),
    ));
    for (i, choice) in d.choices.iter().enumerate() {
        let next = node
            .outputs
            .get(i)
            .and_then(|port| conn_map.get(&port.id.0))
            .and_then(|target| id_map.get(target))
            .cloned();
        let next_attr = next.as_deref().unwrap_or("");
        xml.push_str(&format!(
            "      <option next=\"{next_attr}\"",
        ));
        if let Some(cond) = &choice.condition {
            xml.push_str(&format!(
                " condition-var=\"{}\" condition-op=\"{}\" condition-val=\"{}\"",
                xml_escape(&cond.variable_name),
                compare_op_str(cond.operator),
                variable_value_str(&cond.value),
            ));
        }
        xml.push_str(&format!(">{}</option>\n", xml_escape(&choice.text)));
    }
    xml.push_str("    </node>\n");
}

fn write_condition_node(
    rid: &str,
    d: &crate::model::node::ConditionData,
    node: &crate::model::node::Node,
    id_map: &std::collections::HashMap<uuid::Uuid, String>,
    conn_map: &std::collections::HashMap<uuid::Uuid, uuid::Uuid>,
    xml: &mut String,
) {
    let true_next = node
        .outputs
        .first()
        .and_then(|p| conn_map.get(&p.id.0))
        .and_then(|t| id_map.get(t))
        .cloned();
    let false_next = node
        .outputs
        .get(1)
        .and_then(|p| conn_map.get(&p.id.0))
        .and_then(|t| id_map.get(t))
        .cloned();
    xml.push_str(&format!(
        "    <node id=\"{rid}\" type=\"condition\" variable=\"{}\" operator=\"{}\" value=\"{}\" \
         value-type=\"{}\" true-next=\"{}\" false-next=\"{}\"/>\n",
        xml_escape(&d.variable_name),
        compare_op_str(d.operator),
        variable_value_str(&d.value),
        variable_type_str(&d.value),
        true_next.as_deref().unwrap_or(""),
        false_next.as_deref().unwrap_or(""),
    ));
}

fn write_event_node(
    rid: &str,
    d: &crate::model::node::EventData,
    node: &crate::model::node::Node,
    id_map: &std::collections::HashMap<uuid::Uuid, String>,
    conn_map: &std::collections::HashMap<uuid::Uuid, uuid::Uuid>,
    xml: &mut String,
) {
    let next = find_next_single(&node.outputs, conn_map, id_map);
    let next_attr = next.as_deref().unwrap_or("");
    xml.push_str(&format!(
        "    <node id=\"{rid}\" type=\"event\" next=\"{next_attr}\">\n"
    ));
    for action in &d.actions {
        xml.push_str(&format!(
            "      <action type=\"{}\" key=\"{}\" value=\"{}\"/>\n",
            format!("{:?}", action.action_type).to_lowercase(),
            xml_escape(&action.key),
            variable_value_str(&action.value),
        ));
    }
    xml.push_str("    </node>\n");
}

fn write_random_node(
    rid: &str,
    d: &crate::model::node::RandomData,
    node: &crate::model::node::Node,
    id_map: &std::collections::HashMap<uuid::Uuid, String>,
    conn_map: &std::collections::HashMap<uuid::Uuid, uuid::Uuid>,
    xml: &mut String,
) {
    xml.push_str(&format!(
        "    <node id=\"{rid}\" type=\"random\">\n"
    ));
    for (i, branch) in d.branches.iter().enumerate() {
        let next = node
            .outputs
            .get(i)
            .and_then(|port| conn_map.get(&port.id.0))
            .and_then(|target| id_map.get(target))
            .cloned();
        let next_attr = next.as_deref().unwrap_or("");
        xml.push_str(&format!(
            "      <branch weight=\"{}\" next=\"{next_attr}\"/>\n",
            branch.weight,
        ));
    }
    xml.push_str("    </node>\n");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::node::Node;

    #[test]
    fn export_simple_graph() {
        let mut graph = DialogueGraph::new();
        let start = Node::new_start([0.0, 0.0]);
        let end = Node::new_end([200.0, 0.0]);
        let (s_out, e_in) = (start.outputs[0].id, end.inputs[0].id);
        let (s_id, e_id) = (start.id, end.id);
        graph.add_node(start);
        graph.add_node(end);
        graph.add_connection(s_id, s_out, e_id, e_in);
        let xml = export_xml(&graph, "test").unwrap();
        assert!(xml.contains("<?xml version=") && xml.contains("type=\"start\""));
        assert!(xml.contains("type=\"end\"") && xml.contains("</dialogue>"));
    }

    #[test]
    fn export_escaping() {
        let mut graph = DialogueGraph::new();
        let mut dlg = Node::new_dialogue([0.0, 0.0]);
        if let NodeType::Dialogue(ref mut d) = dlg.node_type {
            d.text = "Hello <world> & \"friends\"".to_string();
            d.speaker_name = "O'Brien".to_string();
        }
        graph.add_node(dlg);
        let xml = export_xml(&graph, "esc&test").unwrap();
        assert!(xml.contains("esc&amp;test") && xml.contains("&lt;world&gt;"));
        assert!(xml.contains("&quot;friends&quot;") && xml.contains("O&apos;Brien"));
    }

    #[test]
    fn export_empty_graph() {
        let graph = DialogueGraph::new();
        let xml = export_xml(&graph, "empty").unwrap();
        assert!(xml.contains("<variables/>") && xml.contains("<nodes/>"));
    }

    #[test]
    fn export_no_positions() {
        let mut graph = DialogueGraph::new();
        graph.add_node(Node::new_start([123.0, 456.0]));
        let xml = export_xml(&graph, "test").unwrap();
        assert!(!xml.contains("position") && !xml.contains("123"));
    }
}
