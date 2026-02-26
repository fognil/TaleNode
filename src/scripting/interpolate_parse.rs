use super::expr::parse_expr;
use super::interpolate::TextSegment;

/// Parse text with `{...}` interpolation and `{if ...}...{else}...{/if}` blocks.
pub(super) fn parse_text(input: &str) -> Result<Vec<TextSegment>, String> {
    let mut segments = Vec::new();
    parse_text_inner(input, &mut 0, &mut segments, false)?;
    Ok(segments)
}

fn parse_text_inner(
    input: &str,
    pos: &mut usize,
    segments: &mut Vec<TextSegment>,
    in_conditional: bool,
) -> Result<(), String> {
    let chars: Vec<char> = input.chars().collect();
    let mut literal_start = *pos;

    while *pos < chars.len() {
        if chars[*pos] == '{' {
            // Flush any accumulated literal
            if *pos > literal_start {
                let lit: String = chars[literal_start..*pos].iter().collect();
                segments.push(TextSegment::Literal(lit));
            }

            *pos += 1; // skip '{'
            skip_whitespace(&chars, pos);

            // Check for special blocks
            let remaining: String = chars[*pos..].iter().collect();

            if remaining.starts_with("/if") {
                *pos += 3;
                skip_whitespace(&chars, pos);
                if *pos < chars.len() && chars[*pos] == '}' {
                    *pos += 1;
                }
                return if in_conditional {
                    Ok(())
                } else {
                    Err("Unexpected {/if} outside conditional block".to_string())
                };
            }

            if remaining.starts_with("else") && peek_after_word(&chars, *pos + 4) == Some('}') {
                *pos += 4;
                skip_whitespace(&chars, pos);
                if *pos < chars.len() && chars[*pos] == '}' {
                    *pos += 1;
                }
                return if in_conditional {
                    Ok(())
                } else {
                    Err("Unexpected {else} outside conditional block".to_string())
                };
            }

            if remaining.starts_with("if ") || remaining.starts_with("if\t") {
                *pos += 2;
                skip_whitespace(&chars, pos);

                let cond_str = read_until_close_brace(&chars, pos)?;
                let condition = parse_expr(&cond_str)?;

                let mut then_text = Vec::new();
                parse_text_inner(input, pos, &mut then_text, true)?;

                let mut else_text = Vec::new();
                let before: String = chars[..(*pos).min(chars.len())].iter().collect();
                if before.ends_with("}") {
                    let check_else = check_stopped_at_else(&chars, *pos);
                    if check_else {
                        parse_text_inner(input, pos, &mut else_text, true)?;
                    }
                }

                segments.push(TextSegment::Conditional {
                    condition,
                    then_text,
                    else_text,
                });
                literal_start = *pos;
                continue;
            }

            // Sequence/cycle/shuffle: {~a|b|c}, {&a|b|c}, {!a|b|c}
            if matches!(remaining.chars().next(), Some('~' | '&' | '!')) {
                let prefix = chars[*pos];
                *pos += 1;
                let content = read_until_close_brace(&chars, pos)?;
                let items = parse_pipe_items(&content)?;
                if items.is_empty() {
                    return Err(format!("Empty {prefix}...}} block"));
                }
                let seg = match prefix {
                    '~' => TextSegment::Sequence { items },
                    '&' => TextSegment::Cycle { items },
                    '!' => TextSegment::Shuffle { items },
                    _ => unreachable!(),
                };
                segments.push(seg);
                literal_start = *pos;
                continue;
            }

            // Regular expression: {expr}
            let expr_str = read_until_close_brace(&chars, pos)?;
            let expr = parse_expr(&expr_str)?;
            segments.push(TextSegment::Expression(expr));
            literal_start = *pos;
        } else if chars[*pos] == '<' && *pos + 1 < chars.len() && chars[*pos + 1] == '<' {
            // Flush literal
            if *pos > literal_start {
                let lit: String = chars[literal_start..*pos].iter().collect();
                segments.push(TextSegment::Literal(lit));
            }
            *pos += 2; // skip "<<"
            let content = read_until_double_close(&chars, pos)?;
            segments.push(parse_command(&content)?);
            literal_start = *pos;
        } else {
            *pos += 1;
        }
    }

    // Flush remaining literal
    if *pos > literal_start {
        let lit: String = chars[literal_start..*pos].iter().collect();
        if !lit.is_empty() {
            segments.push(TextSegment::Literal(lit));
        }
    }

    Ok(())
}

fn skip_whitespace(chars: &[char], pos: &mut usize) {
    while *pos < chars.len() && (chars[*pos] == ' ' || chars[*pos] == '\t') {
        *pos += 1;
    }
}

fn peek_after_word(chars: &[char], pos: usize) -> Option<char> {
    let mut p = pos;
    while p < chars.len() && (chars[p] == ' ' || chars[p] == '\t') {
        p += 1;
    }
    chars.get(p).copied()
}

fn read_until_close_brace(chars: &[char], pos: &mut usize) -> Result<String, String> {
    let start = *pos;
    let mut depth = 1;
    while *pos < chars.len() {
        match chars[*pos] {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    let content: String = chars[start..*pos].iter().collect();
                    *pos += 1;
                    return Ok(content.trim().to_string());
                }
            }
            _ => {}
        }
        *pos += 1;
    }
    Err("Unterminated '{'".to_string())
}

/// Split a string by top-level `|` (respecting brace nesting), then parse each part.
fn parse_pipe_items(content: &str) -> Result<Vec<Vec<TextSegment>>, String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut depth = 0;
    for ch in content.chars() {
        match ch {
            '{' => {
                depth += 1;
                current.push(ch);
            }
            '}' => {
                depth -= 1;
                current.push(ch);
            }
            '|' if depth == 0 => {
                parts.push(std::mem::take(&mut current));
            }
            _ => current.push(ch),
        }
    }
    parts.push(current);

    let mut items = Vec::new();
    for part in &parts {
        items.push(parse_text(part)?);
    }
    Ok(items)
}

/// Read content between `<<` and `>>`.
fn read_until_double_close(chars: &[char], pos: &mut usize) -> Result<String, String> {
    let start = *pos;
    while *pos + 1 < chars.len() {
        if chars[*pos] == '>' && chars[*pos + 1] == '>' {
            let content: String = chars[start..*pos].iter().collect();
            *pos += 2; // skip ">>"
            return Ok(content.trim().to_string());
        }
        *pos += 1;
    }
    Err("Unterminated '<<'".to_string())
}

/// Parse a `<<command args>>` into a TextSegment.
fn parse_command(content: &str) -> Result<TextSegment, String> {
    let content = content.trim();
    if content.is_empty() {
        return Err("Empty command <<>>".to_string());
    }

    // Check for "set var op= expr" or "set var = expr" or "set var expr"
    if let Some(rest) = content.strip_prefix("set ") {
        let rest = rest.trim();
        // Find variable name (first identifier)
        let var_end = rest
            .find(|c: char| !c.is_ascii_alphanumeric() && c != '_')
            .unwrap_or(rest.len());
        if var_end == 0 {
            return Err("<<set>> missing variable name".to_string());
        }
        let var_name = rest[..var_end].to_string();
        let after = rest[var_end..].trim();
        // Check for compound assignment: +=, -=, *=, /=, %=
        let (compound_op, expr_after) = match after.strip_prefix("+=") {
            Some(s) => (Some("+"), s),
            None => match after.strip_prefix("-=") {
                Some(s) => (Some("-"), s),
                None => match after.strip_prefix("*=") {
                    Some(s) => (Some("*"), s),
                    None => match after.strip_prefix("/=") {
                        Some(s) => (Some("/"), s),
                        None => match after.strip_prefix("%=") {
                            Some(s) => (Some("%"), s),
                            None => (None, after),
                        },
                    },
                },
            },
        };
        if let Some(op) = compound_op {
            // Desugar: var op= expr → var = var op expr
            let rhs = expr_after.trim();
            if rhs.is_empty() {
                return Err(format!("<<set {var_name} {op}=>> missing value"));
            }
            let full_expr = format!("{var_name} {op} ({rhs})");
            let expr = parse_expr(&full_expr)?;
            return Ok(TextSegment::SetCommand { var_name, expr });
        }
        // Strip optional '='
        let expr_str = expr_after.strip_prefix('=').unwrap_or(expr_after).trim();
        if expr_str.is_empty() {
            return Err(format!("<<set {var_name}>> missing value expression"));
        }
        let expr = parse_expr(expr_str)?;
        return Ok(TextSegment::SetCommand { var_name, expr });
    }

    // Generic command marker
    Ok(TextSegment::CommandMarker {
        _raw: content.to_string(),
    })
}

fn check_stopped_at_else(chars: &[char], pos: usize) -> bool {
    if pos < 5 {
        return false;
    }
    let window: String = chars[pos.saturating_sub(6)..pos].iter().collect();
    window.contains("else}")
}
