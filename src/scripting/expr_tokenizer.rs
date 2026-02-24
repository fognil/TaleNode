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
