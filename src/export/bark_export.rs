use crate::model::graph::DialogueGraph;

/// Export bark/ambient dialogue lines as CSV.
/// Format: CharacterName,Text,Weight,Condition
pub fn export_bark_csv(graph: &DialogueGraph) -> String {
    let mut out = String::from("CharacterName,Text,Weight,Condition\n");

    for ch in &graph.characters {
        let barks = match graph.barks.get(&ch.id) {
            Some(b) => b,
            None => continue,
        };
        for bark in barks {
            let cond = bark.condition_variable.as_deref().unwrap_or("");
            out.push_str(&format!(
                "{},{},{:.2},{}\n",
                csv_escape(&ch.name),
                csv_escape(&bark.text),
                bark.weight,
                csv_escape(cond),
            ));
        }
    }

    out
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::bark::BarkLine;
    use crate::model::character::Character;

    #[test]
    fn export_empty_graph() {
        let graph = DialogueGraph::new();
        let csv = export_bark_csv(&graph);
        assert_eq!(csv, "CharacterName,Text,Weight,Condition\n");
    }

    #[test]
    fn export_bark_lines() {
        let mut graph = DialogueGraph::new();
        let ch = Character::new("Guard");
        let char_id = ch.id;
        graph.characters.push(ch);
        graph.barks.insert(
            char_id,
            vec![
                BarkLine::new("Halt!"),
                BarkLine::new("Move along."),
            ],
        );
        let csv = export_bark_csv(&graph);
        assert!(csv.contains("Guard,Halt!,1.00,"));
        assert!(csv.contains("Guard,Move along.,1.00,"));
    }

    #[test]
    fn export_bark_with_condition() {
        let mut graph = DialogueGraph::new();
        let ch = Character::new("NPC");
        let char_id = ch.id;
        graph.characters.push(ch);
        let mut bark = BarkLine::new("Danger!");
        bark.condition_variable = Some("alert".to_string());
        graph.barks.insert(char_id, vec![bark]);
        let csv = export_bark_csv(&graph);
        assert!(csv.contains("alert"));
    }
}
