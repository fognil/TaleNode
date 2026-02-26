use crate::model::node::VariableValue;

use super::expr::{BinOp, Expr};
use super::expr_tokenizer::Token;

// --- Recursive descent parser ---

// Precedence levels (low to high):
// 1. || (or)
// 2. && (and)
// 3. == != (equality)
// 4. > < >= <= (comparison)
// 5. + - (additive)
// 6. * / % (multiplicative)
// 7. unary ! -

pub(super) struct Parser {
    tokens: Vec<Token>,
    pub(super) pos: usize,
}

impl Parser {
    pub(super) fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    pub(super) fn tokens_len(&self) -> usize {
        self.tokens.len()
    }

    pub(super) fn token_at(&self, idx: usize) -> Option<&Token> {
        self.tokens.get(idx)
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

    pub(super) fn parse_expr(&mut self) -> Result<Expr, String> {
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
            Some(Token::Ident(name)) => {
                // Check if followed by '(' → function call
                if matches!(self.peek(), Some(Token::LParen)) {
                    self.next(); // consume '('
                    let args = self.parse_args()?;
                    self.expect_rparen()?;
                    Ok(Expr::FunctionCall { name, args })
                } else {
                    Ok(Expr::Variable(name))
                }
            }
            Some(Token::LParen) => {
                let expr = self.parse_expr()?;
                self.expect_rparen()?;
                Ok(expr)
            }
            Some(tok) => Err(format!("Unexpected token: {tok:?}")),
            None => Err("Unexpected end of expression".to_string()),
        }
    }

    fn parse_args(&mut self) -> Result<Vec<Expr>, String> {
        let mut args = Vec::new();
        if matches!(self.peek(), Some(Token::RParen)) {
            return Ok(args);
        }
        args.push(self.parse_expr()?);
        while matches!(self.peek(), Some(Token::Comma)) {
            self.next(); // consume ','
            args.push(self.parse_expr()?);
        }
        Ok(args)
    }
}
