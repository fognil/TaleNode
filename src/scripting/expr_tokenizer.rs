use super::expr::BinOp;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
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
    Comma,
    Question,
    Colon,
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, String> {
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
            ',' => {
                tokens.push(Token::Comma);
                i += 1;
            }
            '?' => {
                tokens.push(Token::Question);
                i += 1;
            }
            ':' => {
                tokens.push(Token::Colon);
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
                let is_unary = tokens.is_empty()
                    || matches!(
                        tokens.last(),
                        Some(
                            Token::Op(_)
                                | Token::Not
                                | Token::Minus
                                | Token::LParen
                                | Token::Question
                                | Token::Colon
                                | Token::Comma
                        )
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
                if i < chars.len()
                    && chars[i] == '.'
                    && i + 1 < chars.len()
                    && chars[i + 1].is_ascii_digit()
                {
                    i += 1; // skip dot
                    while i < chars.len() && chars[i].is_ascii_digit() {
                        i += 1;
                    }
                    let s: String = chars[start..i].iter().collect();
                    let f: f64 =
                        s.parse().map_err(|_| format!("Invalid float: {s}"))?;
                    tokens.push(Token::Float(f));
                } else {
                    let s: String = chars[start..i].iter().collect();
                    let n: i64 =
                        s.parse().map_err(|_| format!("Invalid int: {s}"))?;
                    tokens.push(Token::Int(n));
                }
            }
            c if c.is_ascii_alphabetic() || c == '_' => {
                let start = i;
                while i < chars.len()
                    && (chars[i].is_ascii_alphanumeric() || chars[i] == '_')
                {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_integer() {
        let tokens = tokenize("42").unwrap();
        assert_eq!(tokens, vec![Token::Int(42)]);
    }

    #[test]
    fn tokenize_float() {
        let tokens = tokenize("3.14").unwrap();
        assert_eq!(tokens, vec![Token::Float(3.14)]);
    }

    #[test]
    fn tokenize_string_literal() {
        let tokens = tokenize("\"hello world\"").unwrap();
        assert_eq!(tokens, vec![Token::Str("hello world".to_string())]);
    }

    #[test]
    fn tokenize_unterminated_string() {
        assert!(tokenize("\"oops").is_err());
    }

    #[test]
    fn tokenize_booleans() {
        let tokens = tokenize("true false").unwrap();
        assert_eq!(tokens, vec![Token::Bool(true), Token::Bool(false)]);
    }

    #[test]
    fn tokenize_identifier() {
        let tokens = tokenize("player_name").unwrap();
        assert_eq!(tokens, vec![Token::Ident("player_name".to_string())]);
    }

    #[test]
    fn tokenize_comparison_operators() {
        let tokens = tokenize("== != > < >= <=").unwrap();
        assert_eq!(tokens, vec![
            Token::Op(BinOp::Eq), Token::Op(BinOp::Neq),
            Token::Op(BinOp::Gt), Token::Op(BinOp::Lt),
            Token::Op(BinOp::Gte), Token::Op(BinOp::Lte),
        ]);
    }

    #[test]
    fn tokenize_arithmetic() {
        let tokens = tokenize("1 + 2 * 3 / 4 % 5").unwrap();
        assert_eq!(tokens, vec![
            Token::Int(1), Token::Op(BinOp::Add), Token::Int(2),
            Token::Op(BinOp::Mul), Token::Int(3), Token::Op(BinOp::Div),
            Token::Int(4), Token::Op(BinOp::Mod), Token::Int(5),
        ]);
    }

    #[test]
    fn tokenize_logical_operators() {
        let tokens = tokenize("true && false || !true").unwrap();
        assert_eq!(tokens, vec![
            Token::Bool(true), Token::Op(BinOp::And), Token::Bool(false),
            Token::Op(BinOp::Or), Token::Not, Token::Bool(true),
        ]);
    }

    #[test]
    fn tokenize_parentheses() {
        let tokens = tokenize("(1 + 2)").unwrap();
        assert_eq!(tokens, vec![
            Token::LParen, Token::Int(1), Token::Op(BinOp::Add),
            Token::Int(2), Token::RParen,
        ]);
    }

    #[test]
    fn tokenize_unary_minus() {
        let tokens = tokenize("-5").unwrap();
        assert_eq!(tokens, vec![Token::Minus, Token::Int(5)]);
        // Minus after operator is unary
        let tokens = tokenize("3 + -2").unwrap();
        assert_eq!(tokens, vec![
            Token::Int(3), Token::Op(BinOp::Add), Token::Minus, Token::Int(2),
        ]);
    }

    #[test]
    fn tokenize_binary_minus() {
        let tokens = tokenize("5 - 3").unwrap();
        assert_eq!(tokens, vec![
            Token::Int(5), Token::Op(BinOp::Sub), Token::Int(3),
        ]);
    }

    #[test]
    fn tokenize_unexpected_char() {
        assert!(tokenize("@").is_err());
    }

    #[test]
    fn tokenize_complex_expression() {
        let tokens = tokenize("gold >= 100 && has_key == true").unwrap();
        assert_eq!(tokens, vec![
            Token::Ident("gold".to_string()), Token::Op(BinOp::Gte), Token::Int(100),
            Token::Op(BinOp::And),
            Token::Ident("has_key".to_string()), Token::Op(BinOp::Eq), Token::Bool(true),
        ]);
    }

    #[test]
    fn tokenize_empty_input() {
        let tokens = tokenize("").unwrap();
        assert!(tokens.is_empty());
    }

    #[test]
    fn tokenize_whitespace_only() {
        let tokens = tokenize("   \t\n  ").unwrap();
        assert!(tokens.is_empty());
    }

    #[test]
    fn tokenize_comma() {
        let tokens = tokenize("clamp(x, 0, 100)").unwrap();
        assert_eq!(tokens, vec![
            Token::Ident("clamp".to_string()), Token::LParen,
            Token::Ident("x".to_string()), Token::Comma,
            Token::Int(0), Token::Comma,
            Token::Int(100), Token::RParen,
        ]);
    }
}
