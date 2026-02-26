use crate::model::node::VariableValue;

use super::eval::eval_to_string;

pub fn eval_builtin(name: &str, args: &[VariableValue]) -> Result<VariableValue, String> {
    match name {
        "abs" => builtin_abs(args),
        "round" => builtin_round(args),
        "floor" => builtin_floor(args),
        "ceil" => builtin_ceil(args),
        "min" => builtin_min_max(args, true),
        "max" => builtin_min_max(args, false),
        "clamp" => builtin_clamp(args),
        "random" => builtin_random(args),
        "len" => builtin_len(args),
        "upper" => builtin_upper(args),
        "lower" => builtin_lower(args),
        "trim" => builtin_trim(args),
        "contains" => builtin_contains(args),
        "starts_with" => builtin_starts_with(args),
        "ends_with" => builtin_ends_with(args),
        "replace" => builtin_replace(args),
        "substr" => builtin_substr(args),
        "str" => builtin_str(args),
        "int" => builtin_int(args),
        "float" => builtin_float(args),
        _ => Err(format!("Unknown function: {name}")),
    }
}

fn expect_args(name: &str, args: &[VariableValue], expected: usize) -> Result<(), String> {
    if args.len() != expected {
        Err(format!(
            "{name}() expects {expected} arg(s), got {}",
            args.len()
        ))
    } else {
        Ok(())
    }
}

fn to_f64_val(v: &VariableValue) -> Result<f64, String> {
    match v {
        VariableValue::Int(n) => Ok(*n as f64),
        VariableValue::Float(f) => Ok(*f),
        VariableValue::Bool(b) => Ok(if *b { 1.0 } else { 0.0 }),
        VariableValue::Text(_) => Err("Expected number, got text".to_string()),
    }
}

fn builtin_abs(args: &[VariableValue]) -> Result<VariableValue, String> {
    expect_args("abs", args, 1)?;
    match &args[0] {
        VariableValue::Int(n) => Ok(VariableValue::Int(n.abs())),
        VariableValue::Float(f) => Ok(VariableValue::Float(f.abs())),
        _ => Err("abs() requires a number".to_string()),
    }
}

fn builtin_round(args: &[VariableValue]) -> Result<VariableValue, String> {
    expect_args("round", args, 1)?;
    let f = to_f64_val(&args[0])?;
    Ok(VariableValue::Int(f.round() as i64))
}

fn builtin_floor(args: &[VariableValue]) -> Result<VariableValue, String> {
    expect_args("floor", args, 1)?;
    let f = to_f64_val(&args[0])?;
    Ok(VariableValue::Int(f.floor() as i64))
}

fn builtin_ceil(args: &[VariableValue]) -> Result<VariableValue, String> {
    expect_args("ceil", args, 1)?;
    let f = to_f64_val(&args[0])?;
    Ok(VariableValue::Int(f.ceil() as i64))
}

fn builtin_min_max(args: &[VariableValue], is_min: bool) -> Result<VariableValue, String> {
    let name = if is_min { "min" } else { "max" };
    expect_args(name, args, 2)?;
    let a = to_f64_val(&args[0])?;
    let b = to_f64_val(&args[1])?;
    let result = if is_min { a.min(b) } else { a.max(b) };
    if matches!(
        (&args[0], &args[1]),
        (VariableValue::Int(_), VariableValue::Int(_))
    ) {
        Ok(VariableValue::Int(result as i64))
    } else {
        Ok(VariableValue::Float(result))
    }
}

fn builtin_clamp(args: &[VariableValue]) -> Result<VariableValue, String> {
    expect_args("clamp", args, 3)?;
    let val = to_f64_val(&args[0])?;
    let lo = to_f64_val(&args[1])?;
    let hi = to_f64_val(&args[2])?;
    let result = val.clamp(lo, hi);
    if matches!(
        (&args[0], &args[1], &args[2]),
        (
            VariableValue::Int(_),
            VariableValue::Int(_),
            VariableValue::Int(_)
        )
    ) {
        Ok(VariableValue::Int(result as i64))
    } else {
        Ok(VariableValue::Float(result))
    }
}

fn builtin_random(args: &[VariableValue]) -> Result<VariableValue, String> {
    expect_args("random", args, 2)?;
    let lo = to_f64_val(&args[0])? as i64;
    let hi = to_f64_val(&args[1])? as i64;
    if lo > hi {
        return Err("random() min > max".to_string());
    }
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos() as i64;
    let range = hi - lo + 1;
    Ok(VariableValue::Int(lo + (seed.unsigned_abs() as i64 % range)))
}

fn builtin_len(args: &[VariableValue]) -> Result<VariableValue, String> {
    expect_args("len", args, 1)?;
    match &args[0] {
        VariableValue::Text(s) => Ok(VariableValue::Int(s.len() as i64)),
        other => Ok(VariableValue::Int(eval_to_string(other).len() as i64)),
    }
}

fn builtin_upper(args: &[VariableValue]) -> Result<VariableValue, String> {
    expect_args("upper", args, 1)?;
    Ok(VariableValue::Text(eval_to_string(&args[0]).to_uppercase()))
}

fn builtin_lower(args: &[VariableValue]) -> Result<VariableValue, String> {
    expect_args("lower", args, 1)?;
    Ok(VariableValue::Text(eval_to_string(&args[0]).to_lowercase()))
}

fn builtin_trim(args: &[VariableValue]) -> Result<VariableValue, String> {
    expect_args("trim", args, 1)?;
    Ok(VariableValue::Text(
        eval_to_string(&args[0]).trim().to_string(),
    ))
}

fn builtin_contains(args: &[VariableValue]) -> Result<VariableValue, String> {
    expect_args("contains", args, 2)?;
    let haystack = eval_to_string(&args[0]);
    let needle = eval_to_string(&args[1]);
    Ok(VariableValue::Bool(haystack.contains(&needle)))
}

fn builtin_starts_with(args: &[VariableValue]) -> Result<VariableValue, String> {
    expect_args("starts_with", args, 2)?;
    let s = eval_to_string(&args[0]);
    let prefix = eval_to_string(&args[1]);
    Ok(VariableValue::Bool(s.starts_with(&prefix)))
}

fn builtin_ends_with(args: &[VariableValue]) -> Result<VariableValue, String> {
    expect_args("ends_with", args, 2)?;
    let s = eval_to_string(&args[0]);
    let suffix = eval_to_string(&args[1]);
    Ok(VariableValue::Bool(s.ends_with(&suffix)))
}

fn builtin_replace(args: &[VariableValue]) -> Result<VariableValue, String> {
    expect_args("replace", args, 3)?;
    let s = eval_to_string(&args[0]);
    let from = eval_to_string(&args[1]);
    let to = eval_to_string(&args[2]);
    Ok(VariableValue::Text(s.replace(&from, &to)))
}

fn builtin_substr(args: &[VariableValue]) -> Result<VariableValue, String> {
    expect_args("substr", args, 3)?;
    let s = eval_to_string(&args[0]);
    let start = to_f64_val(&args[1])? as usize;
    let length = to_f64_val(&args[2])? as usize;
    let result: String = s.chars().skip(start).take(length).collect();
    Ok(VariableValue::Text(result))
}

fn builtin_str(args: &[VariableValue]) -> Result<VariableValue, String> {
    expect_args("str", args, 1)?;
    Ok(VariableValue::Text(eval_to_string(&args[0])))
}

fn builtin_int(args: &[VariableValue]) -> Result<VariableValue, String> {
    expect_args("int", args, 1)?;
    match &args[0] {
        VariableValue::Int(n) => Ok(VariableValue::Int(*n)),
        VariableValue::Float(f) => Ok(VariableValue::Int(*f as i64)),
        VariableValue::Bool(b) => Ok(VariableValue::Int(if *b { 1 } else { 0 })),
        VariableValue::Text(s) => s
            .parse::<i64>()
            .map(VariableValue::Int)
            .map_err(|_| format!("Cannot convert \"{s}\" to int")),
    }
}

fn builtin_float(args: &[VariableValue]) -> Result<VariableValue, String> {
    expect_args("float", args, 1)?;
    let f = to_f64_val(&args[0])?;
    Ok(VariableValue::Float(f))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scripting::expr::parse_expr;
    use crate::scripting::eval::eval_expr;
    use crate::scripting::ScriptContext;

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
    fn abs_int_and_float() {
        let ctx = ScriptContext::default();
        assert_eq!(eval("abs(-5)", &ctx), VariableValue::Int(5));
        assert_eq!(eval("abs(-3.14)", &ctx), VariableValue::Float(3.14));
    }

    #[test]
    fn round_floor_ceil() {
        let ctx = ScriptContext::default();
        assert_eq!(eval("round(3.7)", &ctx), VariableValue::Int(4));
        assert_eq!(eval("round(3.2)", &ctx), VariableValue::Int(3));
        assert_eq!(eval("floor(3.9)", &ctx), VariableValue::Int(3));
        assert_eq!(eval("ceil(3.1)", &ctx), VariableValue::Int(4));
    }

    #[test]
    fn min_max_preserves_int() {
        let ctx = ScriptContext::default();
        assert_eq!(eval("min(5, 3)", &ctx), VariableValue::Int(3));
        assert_eq!(eval("max(5, 3)", &ctx), VariableValue::Int(5));
        assert_eq!(eval("min(5, 3.0)", &ctx), VariableValue::Float(3.0));
    }

    #[test]
    fn clamp_int_and_float() {
        let ctx = ctx_with(&[("hp", VariableValue::Int(150))]);
        assert_eq!(eval("clamp(hp, 0, 100)", &ctx), VariableValue::Int(100));
        assert_eq!(eval("clamp(-5, 0, 100)", &ctx), VariableValue::Int(0));
        assert_eq!(eval("clamp(50, 0, 100)", &ctx), VariableValue::Int(50));
    }

    #[test]
    fn random_in_range() {
        let ctx = ScriptContext::default();
        let val = eval("random(1, 10)", &ctx);
        if let VariableValue::Int(n) = val {
            assert!((1..=10).contains(&n));
        } else {
            panic!("Expected Int");
        }
    }

    #[test]
    fn len_of_string() {
        let ctx = ctx_with(&[("name", VariableValue::Text("Hello".to_string()))]);
        assert_eq!(eval("len(name)", &ctx), VariableValue::Int(5));
        assert_eq!(eval("len(\"\")", &ctx), VariableValue::Int(0));
    }

    #[test]
    fn upper_lower_trim() {
        let ctx = ctx_with(&[("s", VariableValue::Text(" Hello ".to_string()))]);
        assert_eq!(
            eval("upper(\"hello\")", &ctx),
            VariableValue::Text("HELLO".to_string())
        );
        assert_eq!(
            eval("lower(\"HELLO\")", &ctx),
            VariableValue::Text("hello".to_string())
        );
        assert_eq!(
            eval("trim(s)", &ctx),
            VariableValue::Text("Hello".to_string())
        );
    }

    #[test]
    fn contains_starts_ends() {
        let ctx = ScriptContext::default();
        assert_eq!(
            eval("contains(\"hello world\", \"world\")", &ctx),
            VariableValue::Bool(true)
        );
        assert_eq!(
            eval("starts_with(\"hello\", \"he\")", &ctx),
            VariableValue::Bool(true)
        );
        assert_eq!(
            eval("ends_with(\"hello\", \"lo\")", &ctx),
            VariableValue::Bool(true)
        );
        assert_eq!(
            eval("ends_with(\"hello\", \"he\")", &ctx),
            VariableValue::Bool(false)
        );
    }

    #[test]
    fn replace_and_substr() {
        let ctx = ScriptContext::default();
        assert_eq!(
            eval("replace(\"hello world\", \"world\", \"there\")", &ctx),
            VariableValue::Text("hello there".to_string())
        );
        assert_eq!(
            eval("substr(\"hello\", 1, 3)", &ctx),
            VariableValue::Text("ell".to_string())
        );
    }

    #[test]
    fn type_conversion() {
        let ctx = ScriptContext::default();
        assert_eq!(
            eval("str(42)", &ctx),
            VariableValue::Text("42".to_string())
        );
        assert_eq!(eval("int(3.9)", &ctx), VariableValue::Int(3));
        assert_eq!(eval("int(\"42\")", &ctx), VariableValue::Int(42));
        assert_eq!(eval("float(5)", &ctx), VariableValue::Float(5.0));
    }

    #[test]
    fn unknown_function_error() {
        let ctx = ScriptContext::default();
        let expr = parse_expr("foobar(1)").unwrap();
        assert!(eval_expr(&expr, &ctx).is_err());
    }

    #[test]
    fn wrong_arg_count_error() {
        let ctx = ScriptContext::default();
        let expr = parse_expr("abs(1, 2)").unwrap();
        assert!(eval_expr(&expr, &ctx).is_err());
    }

    #[test]
    fn nested_function_calls() {
        let ctx = ScriptContext::default();
        assert_eq!(eval("abs(min(-5, 3))", &ctx), VariableValue::Int(5));
        assert_eq!(eval("len(upper(\"hi\"))", &ctx), VariableValue::Int(2));
    }

    #[test]
    fn function_in_arithmetic() {
        let ctx = ctx_with(&[("name", VariableValue::Text("Hero".to_string()))]);
        assert_eq!(eval("len(name) + 1", &ctx), VariableValue::Int(5));
    }
}
