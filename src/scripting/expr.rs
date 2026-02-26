use crate::model::node::VariableValue;

use super::expr_tokenizer::tokenize;

/// Expression AST node.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(VariableValue),
    Variable(String),
    BinaryOp {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
    },
    Not(Box<Expr>),
    Neg(Box<Expr>),
    FunctionCall {
        name: String,
        args: Vec<Expr>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Neq,
    Gt,
    Lt,
    Gte,
    Lte,
    And,
    Or,
}

/// Parse an expression string into an AST.
pub fn parse_expr(input: &str) -> Result<Expr, String> {
    let tokens = tokenize(input)?;
    if tokens.is_empty() {
        return Err("Empty expression".to_string());
    }
    let mut parser = super::expr_parser::Parser::new(tokens);
    let expr = parser.parse_expr()?;
    if parser.pos < parser.tokens_len() {
        return Err(format!(
            "Unexpected token after expression: {:?}",
            parser.token_at(parser.pos).unwrap()
        ));
    }
    Ok(expr)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_integer() {
        let expr = parse_expr("42").unwrap();
        assert_eq!(expr, Expr::Literal(VariableValue::Int(42)));
    }

    #[test]
    fn parse_float() {
        let expr = parse_expr("3.14").unwrap();
        assert_eq!(expr, Expr::Literal(VariableValue::Float(3.14)));
    }

    #[test]
    fn parse_string() {
        let expr = parse_expr("\"hello\"").unwrap();
        assert_eq!(
            expr,
            Expr::Literal(VariableValue::Text("hello".to_string()))
        );
    }

    #[test]
    fn parse_bool() {
        assert_eq!(
            parse_expr("true").unwrap(),
            Expr::Literal(VariableValue::Bool(true))
        );
        assert_eq!(
            parse_expr("false").unwrap(),
            Expr::Literal(VariableValue::Bool(false))
        );
    }

    #[test]
    fn parse_variable() {
        assert_eq!(
            parse_expr("gold").unwrap(),
            Expr::Variable("gold".to_string())
        );
    }

    #[test]
    fn parse_binary_add() {
        let expr = parse_expr("gold + 10").unwrap();
        assert_eq!(
            expr,
            Expr::BinaryOp {
                left: Box::new(Expr::Variable("gold".to_string())),
                op: BinOp::Add,
                right: Box::new(Expr::Literal(VariableValue::Int(10))),
            }
        );
    }

    #[test]
    fn parse_comparison() {
        let expr = parse_expr("gold >= 100").unwrap();
        assert_eq!(
            expr,
            Expr::BinaryOp {
                left: Box::new(Expr::Variable("gold".to_string())),
                op: BinOp::Gte,
                right: Box::new(Expr::Literal(VariableValue::Int(100))),
            }
        );
    }

    #[test]
    fn parse_precedence() {
        let expr = parse_expr("2 + 3 * 4").unwrap();
        assert_eq!(
            expr,
            Expr::BinaryOp {
                left: Box::new(Expr::Literal(VariableValue::Int(2))),
                op: BinOp::Add,
                right: Box::new(Expr::BinaryOp {
                    left: Box::new(Expr::Literal(VariableValue::Int(3))),
                    op: BinOp::Mul,
                    right: Box::new(Expr::Literal(VariableValue::Int(4))),
                }),
            }
        );
    }

    #[test]
    fn parse_boolean_operators() {
        let expr = parse_expr("a && b || c").unwrap();
        assert!(matches!(expr, Expr::BinaryOp { op: BinOp::Or, .. }));
    }

    #[test]
    fn parse_not() {
        let expr = parse_expr("!has_key").unwrap();
        assert_eq!(
            expr,
            Expr::Not(Box::new(Expr::Variable("has_key".to_string())))
        );
    }

    #[test]
    fn parse_negation() {
        let expr = parse_expr("-5").unwrap();
        assert_eq!(
            expr,
            Expr::Neg(Box::new(Expr::Literal(VariableValue::Int(5))))
        );
    }

    #[test]
    fn parse_parens() {
        let expr = parse_expr("(2 + 3) * 4").unwrap();
        assert!(matches!(expr, Expr::BinaryOp { op: BinOp::Mul, .. }));
    }

    #[test]
    fn parse_complex_boolean() {
        let expr = parse_expr("has_key && level > 5").unwrap();
        assert!(matches!(expr, Expr::BinaryOp { op: BinOp::And, .. }));
    }

    #[test]
    fn parse_empty_errors() {
        assert!(parse_expr("").is_err());
    }

    #[test]
    fn parse_unterminated_string() {
        assert!(parse_expr("\"hello").is_err());
    }

    #[test]
    fn parse_function_no_args() {
        let expr = parse_expr("random()").unwrap();
        assert_eq!(
            expr,
            Expr::FunctionCall {
                name: "random".to_string(),
                args: vec![],
            }
        );
    }

    #[test]
    fn parse_function_one_arg() {
        let expr = parse_expr("abs(-5)").unwrap();
        assert!(matches!(expr, Expr::FunctionCall { name, args } if name == "abs" && args.len() == 1));
    }

    #[test]
    fn parse_function_multiple_args() {
        let expr = parse_expr("clamp(hp, 0, 100)").unwrap();
        assert!(matches!(expr, Expr::FunctionCall { name, args } if name == "clamp" && args.len() == 3));
    }

    #[test]
    fn parse_function_nested() {
        let expr = parse_expr("max(abs(-5), len(\"hi\"))").unwrap();
        assert!(matches!(expr, Expr::FunctionCall { name, args } if name == "max" && args.len() == 2));
    }

    #[test]
    fn parse_function_in_expression() {
        let expr = parse_expr("len(name) + 1").unwrap();
        assert!(matches!(expr, Expr::BinaryOp { op: BinOp::Add, .. }));
    }
}
