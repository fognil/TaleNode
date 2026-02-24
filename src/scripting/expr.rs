use crate::model::node::VariableValue;

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

// --- Tokenizer ---

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Ident(String),
    Op(BinOp),
    Not,
    Minus,
    LParen,
    RParen,
}

fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        match chars[i] {
            ' ' | '\t' | '\r' | '\n' => i += 1,
            '(' => {
                tokens.push(Token::LParen);
                i += 1;
            }
            ')' => {
                tokens.push(Token::RParen);
                i += 1;
            }
            '+' => {
                tokens.push(Token::Op(BinOp::Add));
                i += 1;
            }
            '*' => {
                tokens.push(Token::Op(BinOp::Mul));
                i += 1;
            }
            '/' => {
                tokens.push(Token::Op(BinOp::Div));
                i += 1;
            }
            '%' => {
                tokens.push(Token::Op(BinOp::Mod));
                i += 1;
            }
            '-' => {
                // Decide if this is unary minus or subtraction
                let is_unary = tokens.is_empty()
                    || matches!(
                        tokens.last(),
                        Some(Token::Op(_) | Token::Not | Token::Minus | Token::LParen)
                    );
                if is_unary {
                    tokens.push(Token::Minus);
                } else {
                    tokens.push(Token::Op(BinOp::Sub));
                }
                i += 1;
            }
            '!' if i + 1 < chars.len() && chars[i + 1] == '=' => {
                tokens.push(Token::Op(BinOp::Neq));
                i += 2;
            }
            '!' => {
                tokens.push(Token::Not);
                i += 1;
            }
            '=' if i + 1 < chars.len() && chars[i + 1] == '=' => {
                tokens.push(Token::Op(BinOp::Eq));
                i += 2;
            }
            '>' if i + 1 < chars.len() && chars[i + 1] == '=' => {
                tokens.push(Token::Op(BinOp::Gte));
                i += 2;
            }
            '>' => {
                tokens.push(Token::Op(BinOp::Gt));
                i += 1;
            }
            '<' if i + 1 < chars.len() && chars[i + 1] == '=' => {
                tokens.push(Token::Op(BinOp::Lte));
                i += 2;
            }
            '<' => {
                tokens.push(Token::Op(BinOp::Lt));
                i += 1;
            }
            '&' if i + 1 < chars.len() && chars[i + 1] == '&' => {
                tokens.push(Token::Op(BinOp::And));
                i += 2;
            }
            '|' if i + 1 < chars.len() && chars[i + 1] == '|' => {
                tokens.push(Token::Op(BinOp::Or));
                i += 2;
            }
            '"' => {
                i += 1;
                let start = i;
                while i < chars.len() && chars[i] != '"' {
                    i += 1;
                }
                if i >= chars.len() {
                    return Err("Unterminated string literal".to_string());
                }
                let s: String = chars[start..i].iter().collect();
                tokens.push(Token::Str(s));
                i += 1; // skip closing "
            }
            c if c.is_ascii_digit() => {
                let start = i;
                while i < chars.len() && chars[i].is_ascii_digit() {
                    i += 1;
                }
                if i < chars.len() && chars[i] == '.' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit() {
                    i += 1; // skip dot
                    while i < chars.len() && chars[i].is_ascii_digit() {
                        i += 1;
                    }
                    let s: String = chars[start..i].iter().collect();
                    let f: f64 = s.parse().map_err(|_| format!("Invalid float: {s}"))?;
                    tokens.push(Token::Float(f));
                } else {
                    let s: String = chars[start..i].iter().collect();
                    let n: i64 = s.parse().map_err(|_| format!("Invalid int: {s}"))?;
                    tokens.push(Token::Int(n));
                }
            }
            c if c.is_ascii_alphabetic() || c == '_' => {
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                let word: String = chars[start..i].iter().collect();
                match word.as_str() {
                    "true" => tokens.push(Token::Bool(true)),
                    "false" => tokens.push(Token::Bool(false)),
                    _ => tokens.push(Token::Ident(word)),
                }
            }
            c => return Err(format!("Unexpected character: '{c}'")),
        }
    }
    Ok(tokens)
}

// --- Recursive descent parser ---

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<Token> {
        let tok = self.tokens.get(self.pos)?.clone();
        self.pos += 1;
        Some(tok)
    }

    fn expect_rparen(&mut self) -> Result<(), String> {
        if self.next() == Some(Token::RParen) {
            Ok(())
        } else {
            Err("Expected ')'".to_string())
        }
    }

    // Precedence levels (low to high):
    // 1. || (or)
    // 2. && (and)
    // 3. == != (equality)
    // 4. > < >= <= (comparison)
    // 5. + - (additive)
    // 6. * / % (multiplicative)
    // 7. unary ! -

    fn parse_expr(&mut self) -> Result<Expr, String> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_and()?;
        while matches!(self.peek(), Some(Token::Op(BinOp::Or))) {
            self.next();
            let right = self.parse_and()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op: BinOp::Or,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_equality()?;
        while matches!(self.peek(), Some(Token::Op(BinOp::And))) {
            self.next();
            let right = self.parse_equality()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op: BinOp::And,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_comparison()?;
        while matches!(self.peek(), Some(Token::Op(BinOp::Eq | BinOp::Neq))) {
            let op = match self.next() {
                Some(Token::Op(op)) => op,
                _ => unreachable!(),
            };
            let right = self.parse_comparison()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_additive()?;
        while matches!(
            self.peek(),
            Some(Token::Op(BinOp::Gt | BinOp::Lt | BinOp::Gte | BinOp::Lte))
        ) {
            let op = match self.next() {
                Some(Token::Op(op)) => op,
                _ => unreachable!(),
            };
            let right = self.parse_additive()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_multiplicative()?;
        while matches!(self.peek(), Some(Token::Op(BinOp::Add | BinOp::Sub))) {
            let op = match self.next() {
                Some(Token::Op(op)) => op,
                _ => unreachable!(),
            };
            let right = self.parse_multiplicative()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_unary()?;
        while matches!(
            self.peek(),
            Some(Token::Op(BinOp::Mul | BinOp::Div | BinOp::Mod))
        ) {
            let op = match self.next() {
                Some(Token::Op(op)) => op,
                _ => unreachable!(),
            };
            let right = self.parse_unary()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        if matches!(self.peek(), Some(Token::Not)) {
            self.next();
            let expr = self.parse_unary()?;
            return Ok(Expr::Not(Box::new(expr)));
        }
        if matches!(self.peek(), Some(Token::Minus)) {
            self.next();
            let expr = self.parse_unary()?;
            return Ok(Expr::Neg(Box::new(expr)));
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        match self.next() {
            Some(Token::Int(n)) => Ok(Expr::Literal(VariableValue::Int(n))),
            Some(Token::Float(f)) => Ok(Expr::Literal(VariableValue::Float(f))),
            Some(Token::Str(s)) => Ok(Expr::Literal(VariableValue::Text(s))),
            Some(Token::Bool(b)) => Ok(Expr::Literal(VariableValue::Bool(b))),
            Some(Token::Ident(name)) => Ok(Expr::Variable(name)),
            Some(Token::LParen) => {
                let expr = self.parse_expr()?;
                self.expect_rparen()?;
                Ok(expr)
            }
            Some(tok) => Err(format!("Unexpected token: {tok:?}")),
            None => Err("Unexpected end of expression".to_string()),
        }
    }
}

/// Parse an expression string into an AST.
pub fn parse_expr(input: &str) -> Result<Expr, String> {
    let tokens = tokenize(input)?;
    if tokens.is_empty() {
        return Err("Empty expression".to_string());
    }
    let mut parser = Parser::new(tokens);
    let expr = parser.parse_expr()?;
    if parser.pos < parser.tokens.len() {
        return Err(format!(
            "Unexpected token after expression: {:?}",
            parser.tokens[parser.pos]
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
        assert_eq!(expr, Expr::Literal(VariableValue::Text("hello".to_string())));
    }

    #[test]
    fn parse_bool() {
        assert_eq!(parse_expr("true").unwrap(), Expr::Literal(VariableValue::Bool(true)));
        assert_eq!(parse_expr("false").unwrap(), Expr::Literal(VariableValue::Bool(false)));
    }

    #[test]
    fn parse_variable() {
        assert_eq!(parse_expr("gold").unwrap(), Expr::Variable("gold".to_string()));
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
        // 2 + 3 * 4 should parse as 2 + (3 * 4)
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
        // || is lower precedence: (a && b) || c
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
        // && binds tighter than nothing else here, > binds tighter than &&
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
}
