use super::expr::{parse_expr, Expr};
use super::interpolate::TextSegment;

/// Why `parse_text_inner` stopped scanning.
enum StopReason {
    EndOfInput,
    EndIf,
    Else,
    ElseIf(Expr),
}

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
) -> Result<StopReason, String> {
    let chars: Vec<char> = input.chars().collect();
    let mut literal_start = *pos;

    while *pos < chars.len() {
        if chars[*pos] == '{' {
            flush_literal(&chars, literal_start, *pos, segments);
            *pos += 1; // skip '{'
            skip_whitespace(&chars, pos);
            let remaining: String = chars[*pos..].iter().collect();

            if remaining.starts_with("/if") {
                *pos += 3;
                skip_to_close_brace(&chars, pos);
                return if in_conditional {
                    Ok(StopReason::EndIf)
                } else {
                    Err("Unexpected {/if} outside conditional block".to_string())
                };
            }

            if remaining.starts_with("elseif ") || remaining.starts_with("elseif\t") {
                *pos += 6; // skip "elseif"
                skip_whitespace(&chars, pos);
                let cond_str = read_until_close_brace(&chars, pos)?;
                let cond = parse_expr(&cond_str)?;
                return if in_conditional {
                    Ok(StopReason::ElseIf(cond))
                } else {
                    Err("Unexpected {elseif} outside conditional block".to_string())
                };
            }

            if remaining.starts_with("else")
                && peek_after_word(&chars, *pos + 4) == Some('}')
            {
                *pos += 4;
                skip_to_close_brace(&chars, pos);
                return if in_conditional {
                    Ok(StopReason::Else)
                } else {
                    Err("Unexpected {else} outside conditional block".to_string())
                };
            }

            if remaining.starts_with("if ") || remaining.starts_with("if\t") {
                *pos += 2;
                skip_whitespace(&chars, pos);
                let cond_str = read_until_close_brace(&chars, pos)?;
                let condition = parse_expr(&cond_str)?;
                let seg = parse_conditional_chain(input, pos, condition)?;
                segments.push(seg);
                literal_start = *pos;
                continue;
            }

            // Sequence/cycle/shuffle: {~a|b|c}, {&a|b|c}, {!a|b|c}
            if matches!(remaining.chars().next(), Some('~' | '&' | '!' | '?')) {
                let prefix = chars[*pos];
                *pos += 1;
                let content = read_until_close_brace(&chars, pos)?;
                let items = parse_pipe_items(&content)?;
                if items.is_empty() {
                    return Err(format!("Empty {{{prefix}...}} block"));
                }
                let seg = match prefix {
                    '~' => TextSegment::Sequence { items },
                    '&' => TextSegment::Cycle { items },
                    '!' => TextSegment::Shuffle { items },
                    '?' => TextSegment::OnceOnly { items },
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
            flush_literal(&chars, literal_start, *pos, segments);
            *pos += 2; // skip "<<"
            let content = read_until_double_close(&chars, pos)?;
            segments.push(parse_command(&content)?);
            literal_start = *pos;
        } else {
            *pos += 1;
        }
    }

    flush_literal(&chars, literal_start, *pos, segments);
    Ok(StopReason::EndOfInput)
}

/// Parse the full `{if}...{elseif}...{else}...{/if}` chain after the first condition.
fn parse_conditional_chain(
    input: &str,
    pos: &mut usize,
    first_cond: Expr,
) -> Result<TextSegment, String> {
    let mut branches = Vec::new();

    // Parse first branch body
    let mut body = Vec::new();
    let mut reason = parse_text_inner(input, pos, &mut body, true)?;
    branches.push((first_cond, body));

    // Handle elseif chain
    while let StopReason::ElseIf(cond) = reason {
        let mut elseif_body = Vec::new();
        reason = parse_text_inner(input, pos, &mut elseif_body, true)?;
        branches.push((cond, elseif_body));
    }

    // Handle else
    let else_text = match reason {
        StopReason::Else => {
            let mut else_body = Vec::new();
            parse_text_inner(input, pos, &mut else_body, true)?;
            else_body
        }
        _ => Vec::new(),
    };

    Ok(TextSegment::Conditional {
        branches,
        else_text,
    })
}

fn flush_literal(chars: &[char], start: usize, end: usize, segments: &mut Vec<TextSegment>) {
    if end > start {
        let lit: String = chars[start..end].iter().collect();
        if !lit.is_empty() {
            segments.push(TextSegment::Literal(lit));
        }
    }
}

fn skip_whitespace(chars: &[char], pos: &mut usize) {
    while *pos < chars.len() && (chars[*pos] == ' ' || chars[*pos] == '\t') {
        *pos += 1;
    }
}

fn skip_to_close_brace(chars: &[char], pos: &mut usize) {
    skip_whitespace(chars, pos);
    if *pos < chars.len() && chars[*pos] == '}' {
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
            *pos += 2;
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

    if let Some(rest) = content.strip_prefix("set ") {
        return parse_set_command(rest.trim());
    }

    Ok(TextSegment::CommandMarker {
        _raw: content.to_string(),
    })
}

fn parse_set_command(rest: &str) -> Result<TextSegment, String> {
    let var_end = rest
        .find(|c: char| !c.is_ascii_alphanumeric() && c != '_')
        .unwrap_or(rest.len());
    if var_end == 0 {
        return Err("<<set>> missing variable name".to_string());
    }
    let var_name = rest[..var_end].to_string();
    let after = rest[var_end..].trim();

    // Check for compound assignment: +=, -=, *=, /=, %=
    for (prefix, op) in [("+=", "+"), ("-=", "-"), ("*=", "*"), ("/=", "/"), ("%=", "%")] {
        if let Some(rhs) = after.strip_prefix(prefix) {
            let rhs = rhs.trim();
            if rhs.is_empty() {
                return Err(format!("<<set {var_name} {op}=>> missing value"));
            }
            let full_expr = format!("{var_name} {op} ({rhs})");
            let expr = parse_expr(&full_expr)?;
            return Ok(TextSegment::SetCommand { var_name, expr });
        }
    }

    // Simple assignment: strip optional '='
    let expr_str = after.strip_prefix('=').unwrap_or(after).trim();
    if expr_str.is_empty() {
        return Err(format!("<<set {var_name}>> missing value expression"));
    }
    let expr = parse_expr(expr_str)?;
    Ok(TextSegment::SetCommand { var_name, expr })
}
