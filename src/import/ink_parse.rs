//! Intermediate structures from parsing .ink files.
//! Supports: knots, stitches, dialogue, choices (sticky + non-sticky),
//! diverts, variable declarations, simple conditionals, tags, gathers.

use super::ink_parse_helpers::*;

/// A top-level knot (=== name ===).
pub(super) struct InkKnot {
    pub name: String,
    pub lines: Vec<InkLine>,
    pub tags: Vec<String>,
}

/// A single parsed line/construct in Ink.
pub(super) enum InkLine {
    Dialogue {
        text: String,
        tags: Vec<String>,
    },
    Choice {
        text: String,
        _sticky: bool,
        body: Vec<InkLine>,
        condition: Option<String>,
    },
    Divert {
        target: String,
    },
    VarDecl {
        name: String,
        value: String,
    },
    VarSet {
        name: String,
        value: String,
    },
    Condition {
        expression: String,
        true_body: Vec<InkLine>,
        false_body: Vec<InkLine>,
    },
    Gather {
        text: String,
        tags: Vec<String>,
    },
}

/// Parse an entire .ink file into a list of global variables and knots.
pub(super) fn parse_ink(content: &str) -> (Vec<(String, String)>, Vec<InkKnot>) {
    let mut global_vars: Vec<(String, String)> = Vec::new();
    let mut knots: Vec<InkKnot> = Vec::new();
    let mut preamble_lines: Vec<InkLine> = Vec::new();

    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with("//") {
            i += 1;
            continue;
        }

        if is_unsupported(trimmed) {
            i += 1;
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("VAR ") {
            if let Some((name, val)) = parse_var_assignment(rest) {
                global_vars.push((name, val));
            }
            i += 1;
            continue;
        }

        if trimmed.starts_with("===") || (trimmed.starts_with("== ") && trimmed.ends_with(" =="))
        {
            if !preamble_lines.is_empty() && knots.is_empty() {
                knots.push(InkKnot {
                    name: "_start".to_string(),
                    lines: std::mem::take(&mut preamble_lines),
                    tags: Vec::new(),
                });
            }

            let name = extract_knot_name(trimmed);
            let mut knot_tags = Vec::new();
            if let Some(tag_pos) = trimmed.find('#') {
                let tag_part = &trimmed[tag_pos..];
                knot_tags = extract_tags(tag_part);
            }
            i += 1;

            let mut knot_lines = Vec::new();
            while i < lines.len() {
                let next = lines[i].trim();
                if next.starts_with("===")
                    || (next.starts_with("== ") && next.ends_with(" =="))
                {
                    break;
                }
                parse_line_into(next, &lines, &mut i, &mut knot_lines, 0);
            }
            knots.push(InkKnot {
                name,
                lines: knot_lines,
                tags: knot_tags,
            });
        } else {
            parse_line_into(trimmed, &lines, &mut i, &mut preamble_lines, 0);
        }
    }

    if knots.is_empty() && !preamble_lines.is_empty() {
        knots.push(InkKnot {
            name: "_start".to_string(),
            lines: preamble_lines,
            tags: Vec::new(),
        });
    }

    (global_vars, knots)
}

fn parse_line_into(
    trimmed: &str,
    lines: &[&str],
    i: &mut usize,
    out: &mut Vec<InkLine>,
    depth: usize,
) {
    if trimmed.is_empty() || trimmed.starts_with("//") {
        *i += 1;
        return;
    }

    if is_unsupported(trimmed) {
        *i += 1;
        return;
    }

    if let Some(rest) = trimmed.strip_prefix("~ ") {
        if let Some((name, val)) = parse_var_assignment(rest.trim()) {
            out.push(InkLine::VarSet { name, value: val });
        }
        *i += 1;
        return;
    }

    if let Some(rest) = trimmed.strip_prefix("VAR ") {
        if let Some((name, val)) = parse_var_assignment(rest) {
            out.push(InkLine::VarDecl { name, value: val });
        }
        *i += 1;
        return;
    }

    if trimmed.starts_with('{') && trimmed.contains('{') {
        if let Some(cond) = try_parse_condition(trimmed, lines, i) {
            out.push(cond);
            return;
        }
    }

    if trimmed.starts_with("- ") && !trimmed.starts_with("- >") {
        let gather_text = trimmed.strip_prefix("- ").unwrap_or("").trim();
        let (text, tags) = split_tags(gather_text);
        out.push(InkLine::Gather {
            text: text.to_string(),
            tags,
        });
        *i += 1;
        return;
    }

    if trimmed.starts_with("* ") || trimmed.starts_with("+ ") {
        parse_choice(trimmed, lines, i, out, depth);
        return;
    }

    if let Some(rest) = trimmed.strip_prefix("-> ") {
        let target = rest.trim().to_string();
        if target != "DONE" && target != "END" {
            out.push(InkLine::Divert { target });
        }
        *i += 1;
        return;
    }

    // Plain dialogue text
    let (text, tags) = split_tags(trimmed);
    if let Some(divert_pos) = text.find(" -> ") {
        let dialogue_part = text[..divert_pos].trim();
        let target = text[divert_pos + 4..].trim();
        if !dialogue_part.is_empty() {
            out.push(InkLine::Dialogue {
                text: dialogue_part.to_string(),
                tags: tags.clone(),
            });
        }
        if !target.is_empty() && target != "DONE" && target != "END" {
            out.push(InkLine::Divert {
                target: target.to_string(),
            });
        }
    } else if !text.is_empty() {
        out.push(InkLine::Dialogue {
            text: text.to_string(),
            tags,
        });
    }
    *i += 1;
}

fn parse_choice(
    trimmed: &str,
    lines: &[&str],
    i: &mut usize,
    out: &mut Vec<InkLine>,
    depth: usize,
) {
    let _sticky = trimmed.starts_with("+");
    let rest = &trimmed[2..];

    let (condition, choice_text) = if rest.starts_with('{') {
        if let Some(end) = rest.find('}') {
            let cond = rest[1..end].trim().to_string();
            let text = rest[end + 1..].trim();
            (Some(cond), text.to_string())
        } else {
            (None, rest.trim().to_string())
        }
    } else {
        (None, rest.trim().to_string())
    };

    let display_text = clean_choice_text(&choice_text);

    *i += 1;
    let mut body = Vec::new();
    while *i < lines.len() {
        let next = lines[*i].trim();
        if next.is_empty() {
            *i += 1;
            continue;
        }
        if next.starts_with("* ")
            || next.starts_with("+ ")
            || next.starts_with("- ")
            || next.starts_with("===")
            || (next.starts_with("== ") && next.ends_with(" =="))
        {
            break;
        }
        let indent = lines[*i].len() - lines[*i].trim_start().len();
        if indent == 0 && !next.starts_with("->") {
            break;
        }
        parse_line_into(next, lines, i, &mut body, depth + 1);
    }

    out.push(InkLine::Choice {
        text: display_text,
        _sticky,
        body,
        condition,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_knot() {
        let ink = "=== intro ===\nHello world!\n-> END\n";
        let (vars, knots) = parse_ink(ink);
        assert!(vars.is_empty());
        assert_eq!(knots.len(), 1);
        assert_eq!(knots[0].name, "intro");
        assert_eq!(knots[0].lines.len(), 1);
    }

    #[test]
    fn parse_global_vars() {
        let ink = "VAR gold = 100\nVAR has_key = true\n=== start ===\nHi\n";
        let (vars, knots) = parse_ink(ink);
        assert_eq!(vars.len(), 2);
        assert_eq!(vars[0].0, "gold");
        assert_eq!(vars[0].1, "100");
        assert_eq!(knots.len(), 1);
    }

    #[test]
    fn parse_choices() {
        let ink = "=== tavern ===\nWhat'll it be?\n* Ale\n  Good choice.\n* Wine\n  Fancy!\n";
        let (_, knots) = parse_ink(ink);
        assert_eq!(knots.len(), 1);
        let choices: Vec<_> = knots[0]
            .lines
            .iter()
            .filter(|l| matches!(l, InkLine::Choice { .. }))
            .collect();
        assert_eq!(choices.len(), 2);
    }

    #[test]
    fn parse_divert() {
        let ink = "=== start ===\nHello\n-> tavern\n=== tavern ===\nWelcome!\n";
        let (_, knots) = parse_ink(ink);
        assert_eq!(knots.len(), 2);
        let has_divert = knots[0].lines.iter().any(|l| {
            matches!(l, InkLine::Divert { target } if target == "tavern")
        });
        assert!(has_divert);
    }

    #[test]
    fn parse_tags() {
        let ink = "=== intro ===\nHello! #greeting #friendly\n";
        let (_, knots) = parse_ink(ink);
        if let InkLine::Dialogue { tags, .. } = &knots[0].lines[0] {
            assert_eq!(tags.len(), 2);
            assert_eq!(tags[0], "greeting");
        } else {
            panic!("Expected dialogue line");
        }
    }

    #[test]
    fn parse_preamble_without_knots() {
        let ink = "Hello world!\nThis is a simple story.\n";
        let (_, knots) = parse_ink(ink);
        assert_eq!(knots.len(), 1);
        assert_eq!(knots[0].name, "_start");
    }

    #[test]
    fn clean_choice_text_removes_brackets() {
        assert_eq!(clean_choice_text("Ask about [the] weather"), "Ask about  weather");
        assert_eq!(clean_choice_text("Open the door"), "Open the door");
    }

    #[test]
    fn parse_var_set() {
        let ink = "=== start ===\n~ gold = 50\nYou got gold!\n";
        let (_, knots) = parse_ink(ink);
        let has_set = knots[0]
            .lines
            .iter()
            .any(|l| matches!(l, InkLine::VarSet { name, .. } if name == "gold"));
        assert!(has_set);
    }
}
