use crate::model::node::VariableValue;

use super::expr::{BinOp, Expr};
use super::ScriptContext;

/// Evaluate an expression against a script context.
pub fn eval_expr(expr: &Expr, ctx: &ScriptContext) -> Result<VariableValue, String> {
    match expr {
        Expr::Literal(v) => Ok(v.clone()),
        Expr::Variable(name) => ctx
            .get(name)
            .cloned()
            .ok_or_else(|| format!("Undefined variable: {name}")),
        Expr::Not(inner) => {
            let val = eval_expr(inner, ctx)?;
            Ok(VariableValue::Bool(!eval_to_bool(&val)))
        }
        Expr::Neg(inner) => {
            let val = eval_expr(inner, ctx)?;
            match val {
                VariableValue::Int(n) => Ok(VariableValue::Int(-n)),
                VariableValue::Float(f) => Ok(VariableValue::Float(-f)),
                VariableValue::Bool(b) => Ok(VariableValue::Int(if b { -1 } else { 0 })),
                VariableValue::Text(_) => Err("Cannot negate text".to_string()),
            }
        }
        Expr::FunctionCall { name, args } => eval_function(name, args, ctx),
        Expr::Ternary {
            condition,
            then_expr,
            else_expr,
        } => {
            let cond = eval_expr(condition, ctx)?;
            if eval_to_bool(&cond) {
                eval_expr(then_expr, ctx)
            } else {
                eval_expr(else_expr, ctx)
            }
        }
        Expr::BinaryOp { left, op, right } => {
            let lv = eval_expr(left, ctx)?;
            // Short-circuit for && and ||
            match op {
                BinOp::And => {
                    if !eval_to_bool(&lv) {
                        return Ok(VariableValue::Bool(false));
                    }
                    let rv = eval_expr(right, ctx)?;
                    return Ok(VariableValue::Bool(eval_to_bool(&rv)));
                }
                BinOp::Or => {
                    if eval_to_bool(&lv) {
                        return Ok(VariableValue::Bool(true));
                    }
                    let rv = eval_expr(right, ctx)?;
                    return Ok(VariableValue::Bool(eval_to_bool(&rv)));
                }
                _ => {}
            }
            let rv = eval_expr(right, ctx)?;
            eval_binary(*op, &lv, &rv)
        }
    }
}

fn eval_function(
    name: &str,
    args: &[Expr],
    ctx: &ScriptContext,
) -> Result<VariableValue, String> {
    // Context-aware functions
    match name {
        "visits" => return eval_visits(args, ctx),
        "visited" => return eval_visited(args, ctx),
        _ => {}
    }
    let vals: Result<Vec<_>, _> = args.iter().map(|a| eval_expr(a, ctx)).collect();
    let vals = vals?;
    super::builtins::eval_builtin(name, &vals)
}

fn eval_visit_count(args: &[Expr], ctx: &ScriptContext) -> Result<usize, String> {
    match args.len() {
        0 => Ok(ctx.current_visits()),
        1 => {
            let val = eval_expr(&args[0], ctx)?;
            Ok(ctx.get_visits(&eval_to_string(&val)))
        }
        n => Err(format!("visits()/visited() expects 0 or 1 arg(s), got {n}")),
    }
}

fn eval_visits(args: &[Expr], ctx: &ScriptContext) -> Result<VariableValue, String> {
    eval_visit_count(args, ctx).map(|c| VariableValue::Int(c as i64))
}

fn eval_visited(args: &[Expr], ctx: &ScriptContext) -> Result<VariableValue, String> {
    eval_visit_count(args, ctx).map(|c| VariableValue::Bool(c > 0))
}

fn eval_binary(op: BinOp, lv: &VariableValue, rv: &VariableValue) -> Result<VariableValue, String> {
    // String concatenation: any + Text or Text + any
    if op == BinOp::Add
        && (matches!(lv, VariableValue::Text(_)) || matches!(rv, VariableValue::Text(_)))
    {
        return Ok(VariableValue::Text(format!(
            "{}{}",
            eval_to_string(lv),
            eval_to_string(rv)
        )));
    }

    // Numeric operations
    match op {
        BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => {
            let (l, r) = coerce_numeric(lv, rv)?;
            match (l, r) {
                (NumVal::Int(a), NumVal::Int(b)) => {
                    let result = match op {
                        BinOp::Add => a.checked_add(b).ok_or("Integer overflow")?,
                        BinOp::Sub => a.checked_sub(b).ok_or("Integer overflow")?,
                        BinOp::Mul => a.checked_mul(b).ok_or("Integer overflow")?,
                        BinOp::Div => {
                            if b == 0 {
                                return Err("Division by zero".to_string());
                            }
                            a / b
                        }
                        BinOp::Mod => {
                            if b == 0 {
                                return Err("Division by zero".to_string());
                            }
                            a % b
                        }
                        _ => unreachable!(),
                    };
                    Ok(VariableValue::Int(result))
                }
                (NumVal::Float(a), NumVal::Float(b)) => {
                    let result = match op {
                        BinOp::Add => a + b,
                        BinOp::Sub => a - b,
                        BinOp::Mul => a * b,
                        BinOp::Div => a / b,
                        BinOp::Mod => a % b,
                        _ => unreachable!(),
                    };
                    Ok(VariableValue::Float(result))
                }
                _ => unreachable!(),
            }
        }
        BinOp::Eq => Ok(VariableValue::Bool(values_equal(lv, rv))),
        BinOp::Neq => Ok(VariableValue::Bool(!values_equal(lv, rv))),
        BinOp::Gt | BinOp::Lt | BinOp::Gte | BinOp::Lte => {
            let ord = compare_values(lv, rv)?;
            let result = match op {
                BinOp::Gt => ord == std::cmp::Ordering::Greater,
                BinOp::Lt => ord == std::cmp::Ordering::Less,
                BinOp::Gte => ord != std::cmp::Ordering::Less,
                BinOp::Lte => ord != std::cmp::Ordering::Greater,
                _ => unreachable!(),
            };
            Ok(VariableValue::Bool(result))
        }
        BinOp::And | BinOp::Or => unreachable!("handled above"),
    }
}

enum NumVal {
    Int(i64),
    Float(f64),
}

fn coerce_numeric(lv: &VariableValue, rv: &VariableValue) -> Result<(NumVal, NumVal), String> {
    let l = to_num(lv)?;
    let r = to_num(rv)?;
    // If either is float, promote both
    match (&l, &r) {
        (NumVal::Float(_), _) | (_, NumVal::Float(_)) => {
            let lf = match l {
                NumVal::Int(i) => i as f64,
                NumVal::Float(f) => f,
            };
            let rf = match r {
                NumVal::Int(i) => i as f64,
                NumVal::Float(f) => f,
            };
            Ok((NumVal::Float(lf), NumVal::Float(rf)))
        }
        _ => Ok((l, r)),
    }
}

fn to_num(v: &VariableValue) -> Result<NumVal, String> {
    match v {
        VariableValue::Int(n) => Ok(NumVal::Int(*n)),
        VariableValue::Float(f) => Ok(NumVal::Float(*f)),
        VariableValue::Bool(b) => Ok(NumVal::Int(if *b { 1 } else { 0 })),
        VariableValue::Text(_) => Err("Cannot use text in arithmetic".to_string()),
    }
}

fn values_equal(lv: &VariableValue, rv: &VariableValue) -> bool {
    match (lv, rv) {
        (VariableValue::Bool(a), VariableValue::Bool(b)) => a == b,
        (VariableValue::Text(a), VariableValue::Text(b)) => a == b,
        _ => {
            // Try numeric comparison
            if let (Ok(a), Ok(b)) = (to_f64(lv), to_f64(rv)) {
                (a - b).abs() < f64::EPSILON
            } else {
                eval_to_string(lv) == eval_to_string(rv)
            }
        }
    }
}

fn compare_values(
    lv: &VariableValue,
    rv: &VariableValue,
) -> Result<std::cmp::Ordering, String> {
    // Numeric comparison
    if let (Ok(a), Ok(b)) = (to_f64(lv), to_f64(rv)) {
        return Ok(a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal));
    }
    // String comparison
    Ok(eval_to_string(lv).cmp(&eval_to_string(rv)))
}

fn to_f64(v: &VariableValue) -> Result<f64, ()> {
    match v {
        VariableValue::Int(n) => Ok(*n as f64),
        VariableValue::Float(f) => Ok(*f),
        VariableValue::Bool(b) => Ok(if *b { 1.0 } else { 0.0 }),
        VariableValue::Text(_) => Err(()),
    }
}

/// Convert a VariableValue to a boolean (for conditions).
pub fn eval_to_bool(value: &VariableValue) -> bool {
    match value {
        VariableValue::Bool(b) => *b,
        VariableValue::Int(n) => *n != 0,
        VariableValue::Float(f) => *f != 0.0,
        VariableValue::Text(s) => !s.is_empty() && s != "false",
    }
}

/// Convert a VariableValue to a display string.
pub fn eval_to_string(value: &VariableValue) -> String {
    match value {
        VariableValue::Bool(b) => b.to_string(),
        VariableValue::Int(n) => n.to_string(),
        VariableValue::Float(f) => format!("{f}"),
        VariableValue::Text(s) => s.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scripting::expr::parse_expr;

    fn ctx_with(vars: &[(&str, VariableValue)]) -> ScriptContext {
        let mut ctx = ScriptContext::default();
        for (name, val) in vars {
            ctx.set(name, val.clone());
        }
        ctx
    }

    fn eval(input: &str, ctx: &ScriptContext) -> VariableValue {
        let expr = parse_expr(input).unwrap();
        eval_expr(&expr, ctx).unwrap()
    }

    #[test]
    fn eval_literal() {
        let ctx = ScriptContext::default();
        assert_eq!(eval("42", &ctx), VariableValue::Int(42));
        assert_eq!(eval("3.14", &ctx), VariableValue::Float(3.14));
        assert_eq!(eval("true", &ctx), VariableValue::Bool(true));
    }

    #[test]
    fn eval_variable_and_arithmetic() {
        let ctx = ctx_with(&[("gold", VariableValue::Int(50))]);
        assert_eq!(eval("gold", &ctx), VariableValue::Int(50));
        assert_eq!(eval("gold + 10", &ctx), VariableValue::Int(60));
        assert_eq!(eval("100 - gold", &ctx), VariableValue::Int(50));
        assert_eq!(eval("gold * 2", &ctx), VariableValue::Int(100));
        assert_eq!(eval("gold / 5", &ctx), VariableValue::Int(10));
        assert_eq!(eval("gold % 3", &ctx), VariableValue::Int(2));
    }

    #[test]
    fn eval_comparison_and_boolean() {
        let ctx = ctx_with(&[("gold", VariableValue::Int(50))]);
        assert_eq!(eval("gold >= 100", &ctx), VariableValue::Bool(false));
        assert_eq!(eval("gold >= 50", &ctx), VariableValue::Bool(true));
        assert_eq!(eval("gold < 100", &ctx), VariableValue::Bool(true));
        assert_eq!(eval("gold == 50", &ctx), VariableValue::Bool(true));
        assert_eq!(eval("gold != 50", &ctx), VariableValue::Bool(false));
        let ctx2 = ctx_with(&[("has_key", VariableValue::Bool(true)), ("level", VariableValue::Int(10))]);
        assert_eq!(eval("has_key && level > 5", &ctx2), VariableValue::Bool(true));
        assert_eq!(eval("has_key && level > 20", &ctx2), VariableValue::Bool(false));
        assert_eq!(eval("false || has_key", &ctx2), VariableValue::Bool(true));
    }

    #[test]
    fn eval_unary_and_misc() {
        let ctx = ctx_with(&[("flag", VariableValue::Bool(true))]);
        assert_eq!(eval("!flag", &ctx), VariableValue::Bool(false));
        assert_eq!(eval("!false", &ctx), VariableValue::Bool(true));
        let ctx2 = ScriptContext::default();
        assert_eq!(eval("-5", &ctx2), VariableValue::Int(-5));
        assert_eq!(eval("-3.14", &ctx2), VariableValue::Float(-3.14));
        assert_eq!(eval("5 + 1.5", &ctx2), VariableValue::Float(6.5));
        let ctx3 = ctx_with(&[("name", VariableValue::Text("Hero".to_string()))]);
        assert_eq!(eval("name + \" the Brave\"", &ctx3), VariableValue::Text("Hero the Brave".to_string()));
    }

    #[test]
    fn eval_error_cases() {
        let ctx = ScriptContext::default();
        assert!(eval_expr(&parse_expr("10 / 0").unwrap(), &ctx).is_err());
        assert!(eval_expr(&parse_expr("nonexistent").unwrap(), &ctx).is_err());
    }

    #[test]
    fn eval_to_bool_and_string() {
        assert!(eval_to_bool(&VariableValue::Bool(true)));
        assert!(!eval_to_bool(&VariableValue::Bool(false)));
        assert!(eval_to_bool(&VariableValue::Int(1)));
        assert!(!eval_to_bool(&VariableValue::Int(0)));
        assert!(eval_to_bool(&VariableValue::Text("yes".to_string())));
        assert!(!eval_to_bool(&VariableValue::Text("".to_string())));
        assert!(!eval_to_bool(&VariableValue::Text("false".to_string())));
        assert_eq!(eval_to_string(&VariableValue::Bool(true)), "true");
        assert_eq!(eval_to_string(&VariableValue::Int(42)), "42");
        assert_eq!(eval_to_string(&VariableValue::Float(3.14)), "3.14");
        assert_eq!(eval_to_string(&VariableValue::Text("hi".to_string())), "hi");
    }

    #[test]
    fn eval_ternary() {
        let ctx = ctx_with(&[("gold", VariableValue::Int(200))]);
        assert_eq!(eval("gold > 100 ? \"rich\" : \"poor\"", &ctx), VariableValue::Text("rich".to_string()));
        let ctx2 = ctx_with(&[("gold", VariableValue::Int(10))]);
        assert_eq!(eval("gold > 100 ? \"rich\" : \"poor\"", &ctx2), VariableValue::Text("poor".to_string()));
        let ctx3 = ctx_with(&[("x", VariableValue::Int(-5))]);
        assert_eq!(eval("x < 0 ? -x : x", &ctx3), VariableValue::Int(5));
    }

    #[test]
    fn eval_visits_and_visited() {
        let mut ctx = ScriptContext::default();
        ctx.set_seq_scope("node_abc");
        assert_eq!(eval("visits()", &ctx), VariableValue::Int(0));
        assert_eq!(eval("visited()", &ctx), VariableValue::Bool(false));
        ctx.record_visit("node_abc");
        assert_eq!(eval("visits()", &ctx), VariableValue::Int(1));
        assert_eq!(eval("visited()", &ctx), VariableValue::Bool(true));
        ctx.record_visit("node_abc");
        assert_eq!(eval("visits()", &ctx), VariableValue::Int(2));
        // Specific node lookup
        ctx.record_visit("market");
        ctx.record_visit("market");
        assert_eq!(eval("visits(\"market\")", &ctx), VariableValue::Int(2));
        assert_eq!(eval("visits(\"tavern\")", &ctx), VariableValue::Int(0));
        assert_eq!(eval("visited(\"market\")", &ctx), VariableValue::Bool(true));
    }

    #[test]
    fn eval_keyword_aliases() {
        let ctx = ctx_with(&[
            ("a", VariableValue::Bool(true)),
            ("b", VariableValue::Bool(false)),
        ]);
        assert_eq!(eval("a and not b", &ctx), VariableValue::Bool(true));
        assert_eq!(eval("a or b", &ctx), VariableValue::Bool(true));
        assert_eq!(eval("not a", &ctx), VariableValue::Bool(false));
    }
}
