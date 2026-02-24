use super::eval::{eval_expr, eval_to_bool, eval_to_string};
use super::expr::{parse_expr, Expr};
use super::ScriptContext;

/// A segment of interpolated text.
#[derive(Debug, Clone)]
enum TextSegment {
    /// Literal text (no interpolation).
    Literal(String),
    /// An expression to evaluate and insert: `{expr}`.
    Expression(Expr),
    /// Conditional block: `{if cond}...{else}...{/if}`.
    Conditional {
        condition: Expr,
        then_text: Vec<TextSegment>,
        else_text: Vec<TextSegment>,
    },
}

/// Parse text with `{...}` interpolation and `{if ...}...{else}...{/if}` blocks.
fn parse_text(input: &str) -> Result<Vec<TextSegment>, String> {
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
                // End of conditional block
                *pos += 3; // skip "/if"
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
                // {else} marker — return to let caller handle
                *pos += 4; // skip "else"
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
                // Conditional block: {if condition}
                *pos += 2; // skip "if"
                skip_whitespace(&chars, pos);

                let cond_str = read_until_close_brace(&chars, pos)?;
                let condition = parse_expr(&cond_str)?;

                // Parse "then" text
                let mut then_text = Vec::new();
                parse_text_inner(input, pos, &mut then_text, true)?;

                // Check if we stopped at {else} or {/if}
                let mut else_text = Vec::new();
                // Look back to see if previous stop was {else}
                let before: String = chars[..(*pos).min(chars.len())].iter().collect();
                if before.ends_with("}") {
                    // Check if it was {else} by looking at what preceded the }
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
                    *pos += 1; // skip '}'
                    return Ok(content.trim().to_string());
                }
            }
            _ => {}
        }
        *pos += 1;
    }
    Err("Unterminated '{'".to_string())
}

fn check_stopped_at_else(chars: &[char], pos: usize) -> bool {
    // Check if the text just before pos ends with "else}"
    if pos < 5 {
        return false;
    }
    let window: String = chars[pos.saturating_sub(6)..pos].iter().collect();
    window.contains("else}")
}

/// Evaluate interpolated text segments into a final string.
fn interpolate(segments: &[TextSegment], ctx: &ScriptContext) -> String {
    let mut result = String::new();
    for seg in segments {
        match seg {
            TextSegment::Literal(s) => result.push_str(s),
            TextSegment::Expression(expr) => {
                match eval_expr(expr, ctx) {
                    Ok(val) => result.push_str(&eval_to_string(&val)),
                    Err(_) => result.push_str("???"),
                }
            }
            TextSegment::Conditional {
                condition,
                then_text,
                else_text,
            } => {
                let cond_val = eval_expr(condition, ctx)
                    .map(|v| eval_to_bool(&v))
                    .unwrap_or(false);
                if cond_val {
                    result.push_str(&interpolate(then_text, ctx));
                } else {
                    result.push_str(&interpolate(else_text, ctx));
                }
            }
        }
    }
    result
}

/// Parse and interpolate text in one call. Returns original text on parse error.
pub fn interpolate_text(text: &str, ctx: &ScriptContext) -> String {
    // Quick check: if no braces, skip parsing
    if !text.contains('{') {
        return text.to_string();
    }
    match parse_text(text) {
        Ok(segments) => interpolate(&segments, ctx),
        Err(_) => text.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::node::VariableValue;

    fn ctx_with(vars: &[(&str, VariableValue)]) -> ScriptContext {
        let mut ctx = ScriptContext::default();
        for (name, val) in vars {
            ctx.set(name, val.clone());
        }
        ctx
    }

    #[test]
    fn plain_text_unchanged() {
        let ctx = ScriptContext::default();
        assert_eq!(interpolate_text("Hello world", &ctx), "Hello world");
    }

    #[test]
    fn simple_variable_substitution() {
        let ctx = ctx_with(&[("player_name", VariableValue::Text("Hero".to_string()))]);
        assert_eq!(
            interpolate_text("Hello {player_name}!", &ctx),
            "Hello Hero!"
        );
    }

    #[test]
    fn math_expression() {
        let ctx = ctx_with(&[("gold", VariableValue::Int(50))]);
        assert_eq!(
            interpolate_text("You need {100 - gold} more gold.", &ctx),
            "You need 50 more gold."
        );
    }

    #[test]
    fn multiple_expressions() {
        let ctx = ctx_with(&[
            ("name", VariableValue::Text("Hero".to_string())),
            ("gold", VariableValue::Int(50)),
        ]);
        assert_eq!(
            interpolate_text("Hello {name}, you have {gold} gold.", &ctx),
            "Hello Hero, you have 50 gold."
        );
    }

    #[test]
    fn conditional_with_else() {
        let ctx = ctx_with(&[("has_key", VariableValue::Bool(true))]);
        let text = "{if has_key}You unlock the door.{else}The door is locked.{/if}";
        assert_eq!(interpolate_text(text, &ctx), "You unlock the door.");

        let ctx2 = ctx_with(&[("has_key", VariableValue::Bool(false))]);
        assert_eq!(interpolate_text(text, &ctx2), "The door is locked.");
    }

    #[test]
    fn conditional_without_else() {
        let ctx = ctx_with(&[("gold", VariableValue::Int(100))]);
        let text = "{if gold >= 50}You can afford it.{/if}";
        assert_eq!(interpolate_text(text, &ctx), "You can afford it.");

        let ctx2 = ctx_with(&[("gold", VariableValue::Int(10))]);
        assert_eq!(interpolate_text(text, &ctx2), "");
    }

    #[test]
    fn undefined_variable_shows_placeholder() {
        let ctx = ScriptContext::default();
        assert_eq!(interpolate_text("Value: {missing}", &ctx), "Value: ???");
    }

    #[test]
    fn invalid_expression_returns_original() {
        let ctx = ScriptContext::default();
        // Unclosed brace returns original
        assert_eq!(interpolate_text("Hello {", &ctx), "Hello {");
    }

    #[test]
    fn no_braces_fast_path() {
        let ctx = ScriptContext::default();
        let text = "No interpolation here";
        assert_eq!(interpolate_text(text, &ctx), text);
    }

    #[test]
    fn expression_with_comparison() {
        let ctx = ctx_with(&[("gold", VariableValue::Int(200))]);
        assert_eq!(
            interpolate_text("Rich: {gold >= 100}", &ctx),
            "Rich: true"
        );
    }
}
