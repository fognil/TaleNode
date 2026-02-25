use super::ink_parse::InkLine;

pub(super) fn try_parse_condition(
    trimmed: &str,
    _lines: &[&str],
    i: &mut usize,
) -> Option<InkLine> {
    // Simple inline conditional: {condition: true_text | false_text}
    if !trimmed.starts_with('{') || !trimmed.ends_with('}') {
        return None;
    }
    let inner = &trimmed[1..trimmed.len() - 1];
    if let Some(colon_pos) = inner.find(':') {
        let expr = inner[..colon_pos].trim().to_string();
        let rest = inner[colon_pos + 1..].trim();
        let (true_text, false_text) = if let Some(pipe_pos) = rest.find('|') {
            (
                rest[..pipe_pos].trim().to_string(),
                rest[pipe_pos + 1..].trim().to_string(),
            )
        } else {
            (rest.to_string(), String::new())
        };
        let true_body = if true_text.is_empty() {
            Vec::new()
        } else {
            vec![InkLine::Dialogue {
                text: true_text,
                tags: Vec::new(),
            }]
        };
        let false_body = if false_text.is_empty() {
            Vec::new()
        } else {
            vec![InkLine::Dialogue {
                text: false_text,
                tags: Vec::new(),
            }]
        };
        *i += 1;
        return Some(InkLine::Condition {
            expression: expr,
            true_body,
            false_body,
        });
    }
    None
}

pub(super) fn extract_knot_name(line: &str) -> String {
    let s = line.trim_start_matches('=').trim_end_matches('=');
    let s = if let Some(tag_pos) = s.find('#') {
        &s[..tag_pos]
    } else {
        s
    };
    s.trim().to_string()
}

pub(super) fn extract_tags(s: &str) -> Vec<String> {
    s.split('#')
        .filter(|t| !t.trim().is_empty())
        .map(|t| t.trim().to_string())
        .collect()
}

pub(super) fn split_tags(s: &str) -> (String, Vec<String>) {
    if let Some(tag_pos) = s.find('#') {
        let text = s[..tag_pos].trim().to_string();
        let tags = extract_tags(&s[tag_pos..]);
        (text, tags)
    } else {
        (s.to_string(), Vec::new())
    }
}

pub(super) fn clean_choice_text(text: &str) -> String {
    let mut result = String::new();
    let mut in_bracket = false;
    for ch in text.chars() {
        match ch {
            '[' => in_bracket = true,
            ']' => in_bracket = false,
            _ if !in_bracket => result.push(ch),
            _ => {}
        }
    }
    result.trim().to_string()
}

pub(super) fn parse_var_assignment(s: &str) -> Option<(String, String)> {
    let eq_pos = s.find('=')?;
    let name = s[..eq_pos].trim().to_string();
    let val = s[eq_pos + 1..].trim().to_string();
    if name.is_empty() {
        return None;
    }
    Some((name, val))
}

pub(super) fn is_unsupported(line: &str) -> bool {
    line.starts_with("EXTERNAL ")
        || line.starts_with("LIST ")
        || line.starts_with("=== function")
        || line.starts_with("== function")
        || line.starts_with("<>")
}
