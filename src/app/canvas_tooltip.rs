use crate::model::node::{Node, NodeType};

/// Build tooltip text for a node (shows key content).
pub(super) fn node_tooltip_text(node: &Node) -> String {
    match &node.node_type {
        NodeType::Dialogue(data) => {
            let speaker = if data.speaker_name.is_empty() {
                String::new()
            } else {
                format!("[{}] ", data.speaker_name)
            };
            format!("{}{}", speaker, data.text)
        }
        NodeType::Choice(data) => {
            let mut s = data.prompt.clone();
            for (i, c) in data.choices.iter().enumerate() {
                s.push_str(&format!("\n  {}. {}", i + 1, c.text));
            }
            s
        }
        NodeType::Condition(data) => {
            format!("{} {:?} {:?}", data.variable_name, data.operator, data.value)
        }
        NodeType::Event(data) => {
            data.actions
                .iter()
                .map(|a| format!("{} = {:?}", a.key, a.value))
                .collect::<Vec<_>>()
                .join("\n")
        }
        NodeType::End(data) => {
            if data.tag.is_empty() {
                "End".to_string()
            } else {
                format!("End: {}", data.tag)
            }
        }
        NodeType::SubGraph(data) => format!("Sub-graph: {}", data.name),
        _ => String::new(),
    }
}
