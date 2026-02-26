use super::eval::{eval_expr, eval_to_bool, eval_to_string};
use super::expr::Expr;
use super::interpolate_parse::parse_text;
use super::ScriptContext;

/// A segment of interpolated text.
#[derive(Debug, Clone)]
pub(super) enum TextSegment {
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
    /// Sequence block `{~a|b|c}`: shows items in order, sticks on last.
    Sequence { items: Vec<Vec<TextSegment>> },
    /// Cycle block `{&a|b|c}`: cycles through items repeatedly.
    Cycle { items: Vec<Vec<TextSegment>> },
    /// Shuffle block `{!a|b|c}`: random pick each time.
    Shuffle { items: Vec<Vec<TextSegment>> },
    /// Inline set command: `<<set var = expr>>`.
    SetCommand { var_name: String, expr: Expr },
    /// Generic command marker: `<<name args>>` (produces no output in playtest).
    CommandMarker { _raw: String },
}

/// Evaluate interpolated text segments into a final string.
fn interpolate(segments: &[TextSegment], ctx: &mut ScriptContext, seq_idx: &mut usize) -> String {
    let mut result = String::new();
    for seg in segments {
        match seg {
            TextSegment::Literal(s) => result.push_str(s),
            TextSegment::Expression(expr) => match eval_expr(expr, ctx) {
                Ok(val) => result.push_str(&eval_to_string(&val)),
                Err(_) => result.push_str("???"),
            },
            TextSegment::Conditional {
                condition,
                then_text,
                else_text,
            } => {
                let cond_val = eval_expr(condition, ctx)
                    .map(|v| eval_to_bool(&v))
                    .unwrap_or(false);
                if cond_val {
                    result.push_str(&interpolate(then_text, ctx, seq_idx));
                } else {
                    result.push_str(&interpolate(else_text, ctx, seq_idx));
                }
            }
            TextSegment::Sequence { items } => {
                let count = ctx.next_seq_count(*seq_idx);
                *seq_idx += 1;
                let idx = count.min(items.len() - 1);
                result.push_str(&interpolate(&items[idx], ctx, seq_idx));
            }
            TextSegment::Cycle { items } => {
                let count = ctx.next_seq_count(*seq_idx);
                *seq_idx += 1;
                let idx = count % items.len();
                result.push_str(&interpolate(&items[idx], ctx, seq_idx));
            }
            TextSegment::Shuffle { items } => {
                *seq_idx += 1;
                let seed = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .subsec_nanos() as usize;
                let idx = seed % items.len();
                result.push_str(&interpolate(&items[idx], ctx, seq_idx));
            }
            TextSegment::SetCommand { var_name, expr } => {
                if let Ok(val) = eval_expr(expr, ctx) {
                    ctx.set(var_name, val);
                }
            }
            TextSegment::CommandMarker { _raw } => {
                // Generic commands produce no output in playtest
            }
        }
    }
    result
}

/// Parse and interpolate text in one call. Returns original text on parse error.
pub fn interpolate_text(text: &str, ctx: &mut ScriptContext) -> String {
    // Quick check: if no interpolation markers, skip parsing
    if !text.contains('{') && !text.contains("<<") {
        return text.to_string();
    }
    match parse_text(text) {
        Ok(segments) => {
            let mut seq_idx = 0;
            interpolate(&segments, ctx, &mut seq_idx)
        }
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
        let mut ctx = ScriptContext::default();
        assert_eq!(interpolate_text("Hello world", &mut ctx), "Hello world");
    }

    #[test]
    fn simple_variable_substitution() {
        let mut ctx = ctx_with(&[("player_name", VariableValue::Text("Hero".to_string()))]);
        assert_eq!(
            interpolate_text("Hello {player_name}!", &mut ctx),
            "Hello Hero!"
        );
    }

    #[test]
    fn math_expression() {
        let mut ctx = ctx_with(&[("gold", VariableValue::Int(50))]);
        assert_eq!(
            interpolate_text("You need {100 - gold} more gold.", &mut ctx),
            "You need 50 more gold."
        );
    }

    #[test]
    fn multiple_expressions() {
        let mut ctx = ctx_with(&[
            ("name", VariableValue::Text("Hero".to_string())),
            ("gold", VariableValue::Int(50)),
        ]);
        assert_eq!(
            interpolate_text("Hello {name}, you have {gold} gold.", &mut ctx),
            "Hello Hero, you have 50 gold."
        );
    }

    #[test]
    fn conditional_with_else() {
        let mut ctx = ctx_with(&[("has_key", VariableValue::Bool(true))]);
        let text = "{if has_key}You unlock the door.{else}The door is locked.{/if}";
        assert_eq!(interpolate_text(text, &mut ctx), "You unlock the door.");

        let mut ctx2 = ctx_with(&[("has_key", VariableValue::Bool(false))]);
        assert_eq!(interpolate_text(text, &mut ctx2), "The door is locked.");
    }

    #[test]
    fn conditional_without_else() {
        let mut ctx = ctx_with(&[("gold", VariableValue::Int(100))]);
        let text = "{if gold >= 50}You can afford it.{/if}";
        assert_eq!(interpolate_text(text, &mut ctx), "You can afford it.");

        let mut ctx2 = ctx_with(&[("gold", VariableValue::Int(10))]);
        assert_eq!(interpolate_text(text, &mut ctx2), "");
    }

    #[test]
    fn undefined_variable_shows_placeholder() {
        let mut ctx = ScriptContext::default();
        assert_eq!(interpolate_text("Value: {missing}", &mut ctx), "Value: ???");
    }

    #[test]
    fn invalid_expression_returns_original() {
        let mut ctx = ScriptContext::default();
        assert_eq!(interpolate_text("Hello {", &mut ctx), "Hello {");
    }

    #[test]
    fn no_braces_fast_path() {
        let mut ctx = ScriptContext::default();
        let text = "No interpolation here";
        assert_eq!(interpolate_text(text, &mut ctx), text);
    }

    #[test]
    fn expression_with_comparison() {
        let mut ctx = ctx_with(&[("gold", VariableValue::Int(200))]);
        assert_eq!(
            interpolate_text("Rich: {gold >= 100}", &mut ctx),
            "Rich: true"
        );
    }

    #[test]
    fn sequence_advances_and_sticks() {
        let mut ctx = ScriptContext::default();
        ctx.set_seq_scope("test_node");
        let text = "{~Hello|Welcome back|Old friend}";
        assert_eq!(interpolate_text(text, &mut ctx), "Hello");
        assert_eq!(interpolate_text(text, &mut ctx), "Welcome back");
        assert_eq!(interpolate_text(text, &mut ctx), "Old friend");
        // Sticks on last
        assert_eq!(interpolate_text(text, &mut ctx), "Old friend");
    }

    #[test]
    fn cycle_wraps_around() {
        let mut ctx = ScriptContext::default();
        ctx.set_seq_scope("test_node");
        let text = "{&morning|afternoon|evening}";
        assert_eq!(interpolate_text(text, &mut ctx), "morning");
        assert_eq!(interpolate_text(text, &mut ctx), "afternoon");
        assert_eq!(interpolate_text(text, &mut ctx), "evening");
        // Wraps around
        assert_eq!(interpolate_text(text, &mut ctx), "morning");
    }

    #[test]
    fn shuffle_picks_from_items() {
        let mut ctx = ScriptContext::default();
        let text = "{!red|green|blue}";
        let result = interpolate_text(text, &mut ctx);
        assert!(["red", "green", "blue"].contains(&result.as_str()));
    }

    #[test]
    fn sequence_with_expressions() {
        let mut ctx = ctx_with(&[("gold", VariableValue::Int(50))]);
        ctx.set_seq_scope("test_node");
        let text = "{~You have {gold} gold|You still have {gold} gold}";
        assert_eq!(interpolate_text(text, &mut ctx), "You have 50 gold");
        assert_eq!(interpolate_text(text, &mut ctx), "You still have 50 gold");
    }

    #[test]
    fn set_command_modifies_context() {
        let mut ctx = ctx_with(&[("gold", VariableValue::Int(50))]);
        let text = "You had {gold} gold.<<set gold = 100>> Now you have {gold}.";
        let result = interpolate_text(text, &mut ctx);
        assert_eq!(result, "You had 50 gold. Now you have 100.");
        assert_eq!(ctx.get("gold"), Some(&VariableValue::Int(100)));
    }

    #[test]
    fn set_command_with_expression() {
        let mut ctx = ctx_with(&[("hp", VariableValue::Int(80))]);
        let text = "<<set hp = hp + 20>>HP is now {hp}.";
        let result = interpolate_text(text, &mut ctx);
        assert_eq!(result, "HP is now 100.");
    }

    #[test]
    fn set_command_without_equals() {
        let mut ctx = ScriptContext::default();
        let text = "<<set flag true>>Flag: {flag}";
        let result = interpolate_text(text, &mut ctx);
        assert_eq!(result, "Flag: true");
    }

    #[test]
    fn generic_command_produces_no_output() {
        let mut ctx = ScriptContext::default();
        let text = "Hello<<wait 2>> world";
        assert_eq!(interpolate_text(text, &mut ctx), "Hello world");
    }

    #[test]
    fn different_scopes_independent() {
        let mut ctx = ScriptContext::default();
        let text = "{~first|second|third}";
        ctx.set_seq_scope("node_a");
        assert_eq!(interpolate_text(text, &mut ctx), "first");
        ctx.set_seq_scope("node_b");
        assert_eq!(interpolate_text(text, &mut ctx), "first");
        ctx.set_seq_scope("node_a");
        assert_eq!(interpolate_text(text, &mut ctx), "second");
    }
}
