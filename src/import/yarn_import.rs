use crate::model::graph::DialogueGraph;

use super::yarn_build::YarnNode;

// --- Intermediate parse types ---

pub(super) enum YarnLine {
    Dialogue { speaker: Option<String>, text: String },
    ShortcutOption { text: String, condition: Option<String>, body: Vec<YarnLine> },
    SetCommand { variable: String, value: String },
    JumpCommand { target: String },
    Link { target: String },
}

// --- Public entry ---

/// Import a Yarn Spinner (.yarn) file into a DialogueGraph.
pub fn import_yarn(content: &str) -> Result<DialogueGraph, String> {
    let raw_nodes = split_yarn_nodes(content)?;
    if raw_nodes.is_empty() {
        return Err("No yarn nodes found in file".to_string());
    }
    let yarn_nodes: Vec<YarnNode> = raw_nodes
        .into_iter()
        .map(parse_yarn_node)
        .collect::<Result<Vec<_>, _>>()?;
    super::yarn_build::build_graph(yarn_nodes)
}

// --- Parsing helpers ---

fn split_yarn_nodes(content: &str) -> Result<Vec<&str>, String> {
    let mut nodes = Vec::new();
    for block in content.split("===") {
        let trimmed = block.trim();
        if trimmed.is_empty() {
            continue;
        }
        if !trimmed.contains("---") {
            continue;
        }
        nodes.push(trimmed);
    }
    Ok(nodes)
}

fn parse_yarn_node(raw: &str) -> Result<YarnNode, String> {
    let sep_idx = raw.find("---").ok_or("Missing --- separator")?;
    let header = &raw[..sep_idx];
    let body = raw[sep_idx + 3..].trim();

    let mut title = String::new();

    for line in header.lines() {
        let line = line.trim();
        if let Some(val) = line.strip_prefix("title:") {
            title = val.trim().to_string();
        }
    }
    if title.is_empty() {
        return Err("Yarn node missing title".to_string());
    }

    let lines = parse_body_lines(body);
    Ok(YarnNode { title, lines })
}

fn parse_body_lines(body: &str) -> Vec<YarnLine> {
    let mut result = Vec::new();
    let lines: Vec<&str> = body.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();
        if trimmed.is_empty() {
            i += 1;
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("->") {
            let (text, condition) = parse_shortcut_option(rest.trim());
            let indent = line.len() - line.trim_start().len();
            let mut body_lines = Vec::new();
            let mut j = i + 1;
            while j < lines.len() {
                let next = lines[j];
                let next_trimmed = next.trim();
                if next_trimmed.is_empty() {
                    j += 1;
                    continue;
                }
                let next_indent = next.len() - next.trim_start().len();
                if next_indent > indent {
                    body_lines.push(next_trimmed);
                    j += 1;
                } else {
                    break;
                }
            }
            let body = parse_flat_lines(&body_lines);
            result.push(YarnLine::ShortcutOption { text, condition, body });
            i = j;
        } else if trimmed.starts_with("<<set ") {
            if let Some(cmd) = parse_set_command(trimmed) {
                result.push(cmd);
            }
            i += 1;
        } else if trimmed.starts_with("<<jump ") {
            if let Some(target) = trimmed
                .strip_prefix("<<jump ")
                .and_then(|s| s.strip_suffix(">>"))
            {
                result.push(YarnLine::JumpCommand {
                    target: target.trim().to_string(),
                });
            }
            i += 1;
        } else if trimmed.starts_with("[[") && trimmed.ends_with("]]") {
            let inner = &trimmed[2..trimmed.len() - 2];
            let target = if let Some(idx) = inner.find('|') {
                inner[idx + 1..].trim().to_string()
            } else {
                inner.trim().to_string()
            };
            result.push(YarnLine::Link { target });
            i += 1;
        } else if trimmed.starts_with("<<") {
            // Skip unknown commands
            i += 1;
        } else {
            let (speaker, text) = parse_dialogue_line(trimmed);
            result.push(YarnLine::Dialogue { speaker, text });
            i += 1;
        }
    }
    result
}

fn parse_flat_lines(lines: &[&str]) -> Vec<YarnLine> {
    let mut result = Vec::new();
    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with("<<set ") {
            if let Some(cmd) = parse_set_command(trimmed) {
                result.push(cmd);
            }
        } else if trimmed.starts_with("<<jump ") {
            if let Some(target) = trimmed
                .strip_prefix("<<jump ")
                .and_then(|s| s.strip_suffix(">>"))
            {
                result.push(YarnLine::JumpCommand {
                    target: target.trim().to_string(),
                });
            }
        } else if trimmed.starts_with("[[") && trimmed.ends_with("]]") {
            let inner = &trimmed[2..trimmed.len() - 2];
            let target = if let Some(idx) = inner.find('|') {
                inner[idx + 1..].trim().to_string()
            } else {
                inner.trim().to_string()
            };
            result.push(YarnLine::Link { target });
        } else {
            let (speaker, text) = parse_dialogue_line(trimmed);
            result.push(YarnLine::Dialogue { speaker, text });
        }
    }
    result
}

fn parse_dialogue_line(line: &str) -> (Option<String>, String) {
    if let Some(idx) = line.find(':') {
        let candidate = line[..idx].trim();
        if !candidate.is_empty()
            && !candidate.starts_with('<')
            && !candidate.starts_with('[')
            && candidate.chars().all(|c| c.is_alphanumeric() || c == '_' || c == ' ')
        {
            let text = line[idx + 1..].trim().to_string();
            return (Some(candidate.to_string()), text);
        }
    }
    (None, line.to_string())
}

fn parse_shortcut_option(text: &str) -> (String, Option<String>) {
    if let Some(cond_start) = text.find("<<if ") {
        let option_text = text[..cond_start].trim().to_string();
        let rest = &text[cond_start + 5..];
        let condition = rest.strip_suffix(">>").map(|s| s.trim().to_string());
        (option_text, condition)
    } else {
        (text.to_string(), None)
    }
}

fn parse_set_command(line: &str) -> Option<YarnLine> {
    let inner = line.strip_prefix("<<set ")?.strip_suffix(">>")?;
    let inner = inner.trim();
    let (var_name, value) = if let Some(idx) = inner.find(" to ") {
        (
            inner[..idx].trim().trim_start_matches('$').to_string(),
            inner[idx + 4..].trim().to_string(),
        )
    } else if let Some(idx) = inner.find('=') {
        (
            inner[..idx].trim().trim_start_matches('$').to_string(),
            inner[idx + 1..].trim().to_string(),
        )
    } else {
        return None;
    };
    Some(YarnLine::SetCommand { variable: var_name, value })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::node::NodeType;

    #[test]
    fn import_simple_dialogue() {
        let yarn = "\
title: Start
---
Guard: Hello there!
Guard: Welcome to the village.
===";
        let graph = import_yarn(yarn).expect("should parse");
        assert!(graph.nodes.len() >= 3);
        assert_eq!(graph.characters.len(), 1);
        assert_eq!(graph.characters[0].name, "Guard");
    }

    #[test]
    fn import_shortcut_options() {
        let yarn = "\
title: Start
---
Guard: What do you want?
-> Buy something
    Guard: Here are my wares.
-> Leave
    Guard: Goodbye.
===";
        let graph = import_yarn(yarn).expect("should parse");
        let choice_count = graph.nodes.values().filter(|n| {
            matches!(n.node_type, NodeType::Choice(_))
        }).count();
        assert!(choice_count >= 1);
    }

    #[test]
    fn import_set_variable() {
        let yarn = "\
title: Start
---
<<set $gold to 100>>
===";
        let graph = import_yarn(yarn).expect("should parse");
        assert_eq!(graph.variables.len(), 1);
        assert_eq!(graph.variables[0].name, "gold");
        let event_count = graph.nodes.values().filter(|n| {
            matches!(n.node_type, NodeType::Event(_))
        }).count();
        assert_eq!(event_count, 1);
    }

    #[test]
    fn import_jump_between_nodes() {
        let yarn = "\
title: Start
---
Guard: Hello!
<<jump Shop>>
===
title: Shop
---
Merchant: Welcome to my shop!
===";
        let graph = import_yarn(yarn).expect("should parse");
        assert!(graph.connections.len() >= 2);
    }

    #[test]
    fn import_empty_returns_error() {
        assert!(import_yarn("").is_err());
    }

    #[test]
    fn import_malformed_no_title() {
        let yarn = "\
---
Guard: Hello!
===";
        assert!(import_yarn(yarn).is_err());
    }
}
