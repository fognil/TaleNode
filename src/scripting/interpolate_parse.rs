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

fn check_stopped_at_else(chars: &[char], pos: usize) -> bool {
    if pos < 5 {
        return false;
    }
    let window: String = chars[pos.saturating_sub(6)..pos].iter().collect();
    window.contains("else}")
}
