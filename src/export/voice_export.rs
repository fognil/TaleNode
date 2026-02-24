use crate::export::json_export_helpers::build_id_map;
use crate::model::graph::DialogueGraph;
use crate::model::node::NodeType;

/// Escape a field for CSV: wrap in double quotes if it contains comma, quote, or newline.
/// Inner double quotes are escaped as "".
fn csv_escape(field: &str) -> String {
    if field.contains(',') || field.contains('"') || field.contains('\n') {
        let escaped = field.replace('"', "\"\"");
        format!("\"{escaped}\"")
    } else {
        field.to_string()
    }
}

/// Export a voice script CSV from a DialogueGraph.
///
/// Columns: LineID,Speaker,Text,Emotion,AudioFile,Duration,Notes
/// Only Dialogue nodes are included, sorted by readable ID.
pub fn export_voice_csv(graph: &DialogueGraph, name: &str) -> String {
    let id_map = build_id_map(graph);
    let _ = name; // name reserved for future header comment

    // Collect dialogue nodes with their readable IDs
    let mut rows: Vec<(String, &crate::model::node::Node)> = graph
        .nodes
        .values()
        .filter(|n| matches!(n.node_type, NodeType::Dialogue(_)))
        .filter_map(|n| {
            id_map.get(&n.id).map(|rid| (rid.clone(), n))
        })
        .collect();

    // Sort by readable ID (dlg_1, dlg_2, ...)
    rows.sort_by(|a, b| {
        let num_a = a.0.strip_prefix("dlg_").and_then(|s| s.parse::<usize>().ok()).unwrap_or(0);
        let num_b = b.0.strip_prefix("dlg_").and_then(|s| s.parse::<usize>().ok()).unwrap_or(0);
        num_a.cmp(&num_b)
    });

    let mut csv = String::from("LineID,Speaker,Text,Emotion,AudioFile,Duration,Notes\n");

    for (readable_id, node) in &rows {
        if let NodeType::Dialogue(data) = &node.node_type {
            // Resolve speaker: prefer character name from speaker_id
            let speaker = if let Some(sid) = data.speaker_id {
                graph
                    .characters
                    .iter()
                    .find(|c| c.id == sid)
                    .map(|c| c.name.as_str())
                    .unwrap_or(data.speaker_name.as_str())
            } else {
                data.speaker_name.as_str()
            };

            let audio = data.audio_clip.as_deref().unwrap_or("");
            let duration = data.metadata.get("duration").map(|s| s.as_str()).unwrap_or("");
            let notes = data.metadata.get("notes").map(|s| s.as_str()).unwrap_or("");

            csv.push_str(&csv_escape(readable_id));
            csv.push(',');
            csv.push_str(&csv_escape(speaker));
            csv.push(',');
            csv.push_str(&csv_escape(&data.text));
            csv.push(',');
            csv.push_str(&csv_escape(&data.emotion));
            csv.push(',');
            csv.push_str(&csv_escape(audio));
            csv.push(',');
            csv.push_str(&csv_escape(duration));
            csv.push(',');
            csv.push_str(&csv_escape(notes));
            csv.push('\n');
        }
    }

    csv
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::character::Character;
    use crate::model::node::{Node, NodeType};

    #[test]
    fn export_empty_graph_header_only() {
        let graph = DialogueGraph::new();
        let csv = export_voice_csv(&graph, "test");
        assert_eq!(csv, "LineID,Speaker,Text,Emotion,AudioFile,Duration,Notes\n");
    }

    #[test]
    fn export_dialogue_nodes_correct_rows() {
        let mut graph = DialogueGraph::new();

        let ch = Character::new("Elder");
        let ch_id = ch.id;
        graph.characters.push(ch);

        let mut dlg1 = Node::new_dialogue([0.0, 0.0]);
        if let NodeType::Dialogue(ref mut d) = dlg1.node_type {
            d.speaker_id = Some(ch_id);
            d.speaker_name = "Elder".to_string();
            d.text = "Hello, traveler!".to_string();
            d.emotion = "happy".to_string();
            d.audio_clip = Some("dlg_1.wav".to_string());
        }
        graph.add_node(dlg1);

        let mut dlg2 = Node::new_dialogue([0.0, 100.0]);
        if let NodeType::Dialogue(ref mut d) = dlg2.node_type {
            d.speaker_name = "Guard".to_string();
            d.text = "Halt!".to_string();
            d.emotion = "angry".to_string();
            d.metadata.insert("notes".to_string(), "needs alt take".to_string());
        }
        graph.add_node(dlg2);

        let csv = export_voice_csv(&graph, "test");
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(lines.len(), 3); // header + 2 rows
        assert!(lines[1].starts_with("dlg_1,Elder,"));
        assert!(lines[1].contains("happy"));
        assert!(lines[1].contains("dlg_1.wav"));
        assert!(lines[2].starts_with("dlg_2,Guard,Halt!,angry"));
        assert!(lines[2].contains("needs alt take"));
    }

    #[test]
    fn csv_escaping_commas_quotes_newlines() {
        let mut graph = DialogueGraph::new();
        let mut dlg = Node::new_dialogue([0.0, 0.0]);
        if let NodeType::Dialogue(ref mut d) = dlg.node_type {
            d.speaker_name = "NPC".to_string();
            d.text = "He said, \"hello\"".to_string();
            d.emotion = "neutral".to_string();
        }
        graph.add_node(dlg);

        let csv = export_voice_csv(&graph, "test");
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(lines.len(), 2);
        // Text with comma and quotes should be wrapped and escaped
        assert!(lines[1].contains("\"He said, \"\"hello\"\"\""));
    }
}
