//! A tiny expression-language evaluator.
//!
//! Public API:
//! - `parse(input: &str) -> Result<Expr, ParseError>`
//! - `eval(expr: &Expr) -> Result<Value, EvalError>`

use std::fmt;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// A runtime value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Int(i64),
    Bool(bool),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{}", n),
            Value::Bool(b) => write!(f, "{}", b),
        }
    }
}

/// Binary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Or,
    And,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Add,
    Sub,
    Mul,
    Div,
}

/// Unary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    Neg,
    Not,
}

/// AST node.
#[derive(Debug, Clone)]
pub enum Expr {
    Int(i64),
    Bool(bool),
    Ident(String),
    Let {
        name: String,
        value: Box<Expr>,
        body: Box<Expr>,
    },
    If {
        cond: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Box<Expr>,
    },
    Binary {
        op: BinOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Unary {
        op: UnOp,
        expr: Box<Expr>,
    },
}

/// A parse error. Carries the byte offset within the input where parsing failed.
#[derive(Clone)]
pub struct ParseError {
    pub offset: usize,
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "parse error at offset {}: {}", self.offset, self.message)
    }
}

impl fmt::Debug for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ParseError")
            .field("offset", &self.offset)
            .field("message", &self.message)
            .finish()
    }
}

/// An evaluation error.
#[derive(Clone)]
pub enum EvalError {
    /// Operator/operand type mismatch (e.g. `1 + true`, `if 1 then a else b`).
    TypeError(String),
    /// Integer division by zero.
    DivisionByZero,
    /// Reference to an identifier that is not bound.
    UnboundIdentifier(String),
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvalError::TypeError(s) => write!(f, "type error: {}", s),
            EvalError::DivisionByZero => write!(f, "division by zero"),
            EvalError::UnboundIdentifier(name) => write!(f, "unbound identifier: {}", name),
        }
    }
}

impl fmt::Debug for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvalError::TypeError(s) => f.debug_tuple("TypeError").field(s).finish(),
            EvalError::DivisionByZero => f.write_str("DivisionByZero"),
            EvalError::UnboundIdentifier(name) => {
                f.debug_tuple("UnboundIdentifier").field(name).finish()
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Lexer
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Int(i64),
    Ident(String),
    // Keywords
    Let,
    In,
    If,
    Then,
    Else,
    Or,
    And,
    Not,
    True,
    False,
    // Symbols
    Plus,
    Minus,
    Star,
    Slash,
    Eq,
    EqEq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    LParen,
    RParen,
    Eof,
}

struct Lexer<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Self {
        Lexer {
            bytes: input.as_bytes(),
            pos: 0,
        }
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
    }

    fn next_token(&mut self) -> Result<(Token, usize), ParseError> {
        self.skip_whitespace();
        let start = self.pos;
        if self.pos >= self.bytes.len() {
            return Ok((Token::Eof, start));
        }

        let c = self.bytes[self.pos];
        match c {
            b'0'..=b'9' => {
                let mut n: i64 = 0;
                while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_digit() {
                    let d = (self.bytes[self.pos] - b'0') as i64;
                    n = n
                        .checked_mul(10)
                        .and_then(|x| x.checked_add(d))
                        .ok_or_else(|| ParseError {
                            offset: start,
                            message: "integer literal out of range for i64".to_string(),
                        })?;
                    self.pos += 1;
                }
                // A digit run immediately followed by an identifier-start
                // character (e.g. `1abc`) is rejected outright for clarity.
                if self.pos < self.bytes.len() {
                    let nc = self.bytes[self.pos];
                    if nc == b'_' || nc.is_ascii_lowercase() || nc.is_ascii_uppercase() {
                        return Err(ParseError {
                            offset: self.pos,
                            message: "unexpected character after integer literal".to_string(),
                        });
                    }
                }
                Ok((Token::Int(n), start))
            }
            b'a'..=b'z' | b'_' => {
                let begin = self.pos;
                while self.pos < self.bytes.len() {
                    let nc = self.bytes[self.pos];
                    if nc == b'_' || nc.is_ascii_lowercase() || nc.is_ascii_digit() {
                        self.pos += 1;
                    } else {
                        break;
                    }
                }
                // Safe: the slice is pure ASCII by construction.
                let s = std::str::from_utf8(&self.bytes[begin..self.pos]).unwrap();
                let tok = match s {
                    "let" => Token::Let,
                    "in" => Token::In,
                    "if" => Token::If,
                    "then" => Token::Then,
                    "else" => Token::Else,
                    "or" => Token::Or,
                    "and" => Token::And,
                    "not" => Token::Not,
                    "true" => Token::True,
                    "false" => Token::False,
                    _ => Token::Ident(s.to_string()),
                };
                Ok((tok, start))
            }
            b'+' => {
                self.pos += 1;
                Ok((Token::Plus, start))
            }
            b'-' => {
                self.pos += 1;
                Ok((Token::Minus, start))
            }
            b'*' => {
                self.pos += 1;
                Ok((Token::Star, start))
            }
            b'/' => {
                self.pos += 1;
                Ok((Token::Slash, start))
            }
            b'(' => {
                self.pos += 1;
                Ok((Token::LParen, start))
            }
            b')' => {
                self.pos += 1;
                Ok((Token::RParen, start))
            }
            b'=' => {
                self.pos += 1;
                if self.pos < self.bytes.len() && self.bytes[self.pos] == b'=' {
                    self.pos += 1;
                    Ok((Token::EqEq, start))
                } else {
                    Ok((Token::Eq, start))
                }
            }
            b'!' => {
                self.pos += 1;
                if self.pos < self.bytes.len() && self.bytes[self.pos] == b'=' {
                    self.pos += 1;
                    Ok((Token::Ne, start))
                } else {
                    Err(ParseError {
                        offset: start,
                        message: "expected '!='".to_string(),
                    })
                }
            }
            b'<' => {
                self.pos += 1;
                if self.pos < self.bytes.len() && self.bytes[self.pos] == b'=' {
                    self.pos += 1;
                    Ok((Token::Le, start))
                } else {
                    Ok((Token::Lt, start))
                }
            }
            b'>' => {
                self.pos += 1;
                if self.pos < self.bytes.len() && self.bytes[self.pos] == b'=' {
                    self.pos += 1;
                    Ok((Token::Ge, start))
                } else {
                    Ok((Token::Gt, start))
                }
            }
            _ => Err(ParseError {
                offset: start,
                message: if c < 0x80 {
                    format!("unexpected character {:?}", c as char)
                } else {
                    format!("unexpected byte 0x{:02x}", c)
                },
            }),
        }
    }
}

// ---------------------------------------------------------------------------
// Parser
// ---------------------------------------------------------------------------

struct Parser {
    tokens: Vec<(Token, usize)>,
    pos: usize,
}

impl Parser {
    fn new(input: &str) -> Result<Self, ParseError> {
        let mut lexer = Lexer::new(input);
        let mut tokens = Vec::new();
        loop {
            let (tok, off) = lexer.next_token()?;
            let is_eof = matches!(tok, Token::Eof);
            tokens.push((tok, off));
            if is_eof {
                break;
            }
        }
        Ok(Parser { tokens, pos: 0 })
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos].0
    }

    fn peek_offset(&self) -> usize {
        self.tokens[self.pos].1
    }

    fn bump(&mut self) {
        if !matches!(self.tokens[self.pos].0, Token::Eof) {
            self.pos += 1;
        }
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        match self.peek() {
            Token::Let => self.parse_let(),
            Token::If => self.parse_if(),
            _ => self.parse_or(),
        }
    }

    fn parse_let(&mut self) -> Result<Expr, ParseError> {
        self.bump(); // 'let'
        let name = match self.peek().clone() {
            Token::Ident(s) => {
                self.bump();
                s
            }
            _ => {
                return Err(ParseError {
                    offset: self.peek_offset(),
                    message: "expected identifier after 'let'".to_string(),
                });
            }
        };
        if !matches!(self.peek(), Token::Eq) {
            return Err(ParseError {
                offset: self.peek_offset(),
                message: "expected '=' after identifier in 'let'".to_string(),
            });
        }
        self.bump(); // '='
        let value = self.parse_expr()?;
        if !matches!(self.peek(), Token::In) {
            return Err(ParseError {
                offset: self.peek_offset(),
                message: "expected 'in' in 'let'".to_string(),
            });
        }
        self.bump(); // 'in'
        let body = self.parse_expr()?;
        Ok(Expr::Let {
            name,
            value: Box::new(value),
            body: Box::new(body),
        })
    }

    fn parse_if(&mut self) -> Result<Expr, ParseError> {
        self.bump(); // 'if'
        let cond = self.parse_expr()?;
        if !matches!(self.peek(), Token::Then) {
            return Err(ParseError {
                offset: self.peek_offset(),
                message: "expected 'then'".to_string(),
            });
        }
        self.bump(); // 'then'
        let then_branch = self.parse_expr()?;
        if !matches!(self.peek(), Token::Else) {
            return Err(ParseError {
                offset: self.peek_offset(),
                message: "expected 'else'".to_string(),
            });
        }
        self.bump(); // 'else'
        let else_branch = self.parse_expr()?;
        Ok(Expr::If {
            cond: Box::new(cond),
            then_branch: Box::new(then_branch),
            else_branch: Box::new(else_branch),
        })
    }

    fn parse_or(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_and()?;
        while matches!(self.peek(), Token::Or) {
            self.bump();
            let rhs = self.parse_and()?;
            lhs = Expr::Binary {
                op: BinOp::Or,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            };
        }
        Ok(lhs)
    }

    fn parse_and(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_not()?;
        while matches!(self.peek(), Token::And) {
            self.bump();
            let rhs = self.parse_not()?;
            lhs = Expr::Binary {
                op: BinOp::And,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            };
        }
        Ok(lhs)
    }

    fn parse_not(&mut self) -> Result<Expr, ParseError> {
        if matches!(self.peek(), Token::Not) {
            self.bump();
            let inner = self.parse_not()?;
            Ok(Expr::Unary {
                op: UnOp::Not,
                expr: Box::new(inner),
            })
        } else {
            self.parse_cmp()
        }
    }

    fn parse_cmp(&mut self) -> Result<Expr, ParseError> {
        let lhs = self.parse_add()?;
        let op = match self.peek() {
            Token::EqEq => BinOp::Eq,
            Token::Ne => BinOp::Ne,
            Token::Lt => BinOp::Lt,
            Token::Le => BinOp::Le,
            Token::Gt => BinOp::Gt,
            Token::Ge => BinOp::Ge,
            _ => return Ok(lhs),
        };
        self.bump();
        let rhs = self.parse_add()?;
        Ok(Expr::Binary {
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        })
    }

    fn parse_add(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_mul()?;
        loop {
            let op = match self.peek() {
                Token::Plus => BinOp::Add,
                Token::Minus => BinOp::Sub,
                _ => break,
            };
            self.bump();
            let rhs = self.parse_mul()?;
            lhs = Expr::Binary {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            };
        }
        Ok(lhs)
    }

    fn parse_mul(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_unary()?;
        loop {
            let op = match self.peek() {
                Token::Star => BinOp::Mul,
                Token::Slash => BinOp::Div,
                _ => break,
            };
            self.bump();
            let rhs = self.parse_unary()?;
            lhs = Expr::Binary {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            };
        }
        Ok(lhs)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        if matches!(self.peek(), Token::Minus) {
            self.bump();
            let inner = self.parse_unary()?;
            Ok(Expr::Unary {
                op: UnOp::Neg,
                expr: Box::new(inner),
            })
        } else {
            self.parse_atom()
        }
    }

    fn parse_atom(&mut self) -> Result<Expr, ParseError> {
        let offset = self.peek_offset();
        match self.peek().clone() {
            Token::Int(n) => {
                self.bump();
                Ok(Expr::Int(n))
            }
            Token::True => {
                self.bump();
                Ok(Expr::Bool(true))
            }
            Token::False => {
                self.bump();
                Ok(Expr::Bool(false))
            }
            Token::Ident(s) => {
                self.bump();
                Ok(Expr::Ident(s))
            }
            Token::LParen => {
                self.bump();
                let e = self.parse_expr()?;
                if !matches!(self.peek(), Token::RParen) {
                    return Err(ParseError {
                        offset: self.peek_offset(),
                        message: "expected ')'".to_string(),
                    });
                }
                self.bump();
                Ok(e)
            }
            Token::Eof => Err(ParseError {
                offset,
                message: "unexpected end of input".to_string(),
            }),
            _ => Err(ParseError {
                offset,
                message: "expected an expression".to_string(),
            }),
        }
    }
}

/// Parse the input into an `Expr` tree. The whole input must be consumed.
pub fn parse(input: &str) -> Result<Expr, ParseError> {
    let mut parser = Parser::new(input)?;
    let expr = parser.parse_expr()?;
    if !matches!(parser.peek(), Token::Eof) {
        return Err(ParseError {
            offset: parser.peek_offset(),
            message: "unexpected trailing input".to_string(),
        });
    }
    Ok(expr)
}

// ---------------------------------------------------------------------------
// Evaluator
// ---------------------------------------------------------------------------

/// Evaluate an expression with an empty initial environment.
pub fn eval(expr: &Expr) -> Result<Value, EvalError> {
    let mut env: Vec<(String, Value)> = Vec::new();
    eval_in(expr, &mut env)
}

fn lookup(env: &[(String, Value)], name: &str) -> Option<Value> {
    for (n, v) in env.iter().rev() {
        if n == name {
            return Some(v.clone());
        }
    }
    None
}

fn eval_in(expr: &Expr, env: &mut Vec<(String, Value)>) -> Result<Value, EvalError> {
    match expr {
        Expr::Int(n) => Ok(Value::Int(*n)),
        Expr::Bool(b) => Ok(Value::Bool(*b)),
        Expr::Ident(name) => {
            lookup(env, name).ok_or_else(|| EvalError::UnboundIdentifier(name.clone()))
        }
        Expr::Let { name, value, body } => {
            let v = eval_in(value, env)?;
            env.push((name.clone(), v));
            let result = eval_in(body, env);
            env.pop();
            result
        }
        Expr::If {
            cond,
            then_branch,
            else_branch,
        } => {
            let c = eval_in(cond, env)?;
            match c {
                Value::Bool(true) => eval_in(then_branch, env),
                Value::Bool(false) => eval_in(else_branch, env),
                Value::Int(_) => Err(EvalError::TypeError(
                    "condition of 'if' must be bool, got int".to_string(),
                )),
            }
        }
        Expr::Unary { op, expr } => {
            let v = eval_in(expr, env)?;
            match (op, v) {
                (UnOp::Neg, Value::Int(n)) => Ok(Value::Int(n.wrapping_neg())),
                (UnOp::Neg, Value::Bool(_)) => Err(EvalError::TypeError(
                    "unary '-' requires int, got bool".to_string(),
                )),
                (UnOp::Not, Value::Bool(b)) => Ok(Value::Bool(!b)),
                (UnOp::Not, Value::Int(_)) => Err(EvalError::TypeError(
                    "'not' requires bool, got int".to_string(),
                )),
            }
        }
        Expr::Binary { op, lhs, rhs } => match op {
            // Short-circuiting boolean operators.
            BinOp::And => {
                let l = eval_in(lhs, env)?;
                let lb = match l {
                    Value::Bool(b) => b,
                    Value::Int(_) => {
                        return Err(EvalError::TypeError(
                            "left operand of 'and' must be bool, got int".to_string(),
                        ));
                    }
                };
                if !lb {
                    return Ok(Value::Bool(false));
                }
                let r = eval_in(rhs, env)?;
                match r {
                    Value::Bool(b) => Ok(Value::Bool(b)),
                    Value::Int(_) => Err(EvalError::TypeError(
                        "right operand of 'and' must be bool, got int".to_string(),
                    )),
                }
            }
            BinOp::Or => {
                let l = eval_in(lhs, env)?;
                let lb = match l {
                    Value::Bool(b) => b,
                    Value::Int(_) => {
                        return Err(EvalError::TypeError(
                            "left operand of 'or' must be bool, got int".to_string(),
                        ));
                    }
                };
                if lb {
                    return Ok(Value::Bool(true));
                }
                let r = eval_in(rhs, env)?;
                match r {
                    Value::Bool(b) => Ok(Value::Bool(b)),
                    Value::Int(_) => Err(EvalError::TypeError(
                        "right operand of 'or' must be bool, got int".to_string(),
                    )),
                }
            }
            _ => {
                let l = eval_in(lhs, env)?;
                let r = eval_in(rhs, env)?;
                eval_strict_binop(*op, l, r)
            }
        },
    }
}

fn eval_strict_binop(op: BinOp, l: Value, r: Value) -> Result<Value, EvalError> {
    fn type_err(op_name: &str) -> EvalError {
        EvalError::TypeError(format!("operator '{}' has incompatible operand types", op_name))
    }

    match op {
        BinOp::Add => match (l, r) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a.wrapping_add(b))),
            _ => Err(type_err("+")),
        },
        BinOp::Sub => match (l, r) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a.wrapping_sub(b))),
            _ => Err(type_err("-")),
        },
        BinOp::Mul => match (l, r) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a.wrapping_mul(b))),
            _ => Err(type_err("*")),
        },
        BinOp::Div => match (l, r) {
            (Value::Int(_), Value::Int(0)) => Err(EvalError::DivisionByZero),
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a.wrapping_div(b))),
            _ => Err(type_err("/")),
        },
        BinOp::Eq => match (l, r) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a == b)),
            (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a == b)),
            _ => Err(type_err("==")),
        },
        BinOp::Ne => match (l, r) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a != b)),
            (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a != b)),
            _ => Err(type_err("!=")),
        },
        BinOp::Lt => match (l, r) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a < b)),
            _ => Err(type_err("<")),
        },
        BinOp::Le => match (l, r) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a <= b)),
            _ => Err(type_err("<=")),
        },
        BinOp::Gt => match (l, r) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a > b)),
            _ => Err(type_err(">")),
        },
        BinOp::Ge => match (l, r) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a >= b)),
            _ => Err(type_err(">=")),
        },
        BinOp::And | BinOp::Or => unreachable!("handled in eval_in"),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn run(s: &str) -> Result<Value, String> {
        let e = parse(s).map_err(|e| e.to_string())?;
        eval(&e).map_err(|e| e.to_string())
    }

    #[test]
    fn arithmetic() {
        assert_eq!(run("1 + 2 * 3").unwrap(), Value::Int(7));
        assert_eq!(run("(1 + 2) * 3").unwrap(), Value::Int(9));
        assert_eq!(run("10 - 3 - 2").unwrap(), Value::Int(5));
        assert_eq!(run("-3 + 5").unwrap(), Value::Int(2));
        assert_eq!(run("- - 4").unwrap(), Value::Int(4));
    }

    #[test]
    fn booleans_and_cmp() {
        assert_eq!(run("true and false").unwrap(), Value::Bool(false));
        assert_eq!(run("true or false").unwrap(), Value::Bool(true));
        assert_eq!(run("not true").unwrap(), Value::Bool(false));
        assert_eq!(run("1 < 2 and 3 >= 3").unwrap(), Value::Bool(true));
        assert_eq!(run("1 == 1").unwrap(), Value::Bool(true));
        assert_eq!(run("true != false").unwrap(), Value::Bool(true));
    }

    #[test]
    fn let_and_if() {
        assert_eq!(
            run("let x = 10 in let y = x + 5 in if y > 12 then y else 0").unwrap(),
            Value::Int(15)
        );
        assert_eq!(
            run("let x = true in if x then 1 else 2").unwrap(),
            Value::Int(1)
        );
    }

    #[test]
    fn type_errors() {
        assert!(matches!(
            eval(&parse("1 + true").unwrap()),
            Err(EvalError::TypeError(_))
        ));
        assert!(matches!(
            eval(&parse("if 1 then 2 else 3").unwrap()),
            Err(EvalError::TypeError(_))
        ));
        assert!(matches!(
            eval(&parse("not 1").unwrap()),
            Err(EvalError::TypeError(_))
        ));
        assert!(matches!(
            eval(&parse("- true").unwrap()),
            Err(EvalError::TypeError(_))
        ));
    }

    #[test]
    fn div_zero_and_unbound() {
        assert!(matches!(
            eval(&parse("10 / 0").unwrap()),
            Err(EvalError::DivisionByZero)
        ));
        assert!(matches!(
            eval(&parse("foo + 1").unwrap()),
            Err(EvalError::UnboundIdentifier(_))
        ));
    }

    #[test]
    fn parse_errors_report_offset() {
        let err = parse("1 + ").unwrap_err();
        assert!(err.offset >= 4);
        let err = parse("1 + @").unwrap_err();
        assert_eq!(err.offset, 4);
        let err = parse("let x 1 in x").unwrap_err();
        assert_eq!(err.offset, 6);
    }

    #[test]
    fn display_value() {
        assert_eq!(format!("{}", Value::Int(42)), "42");
        assert_eq!(format!("{}", Value::Int(-3)), "-3");
        assert_eq!(format!("{}", Value::Bool(true)), "true");
        assert_eq!(format!("{}", Value::Bool(false)), "false");
    }

    #[test]
    fn short_circuit() {
        // Right side would otherwise be a type error / unbound.
        assert_eq!(run("false and bogus").unwrap(), Value::Bool(false));
        assert_eq!(run("true or bogus").unwrap(), Value::Bool(true));
    }

    #[test]
    fn shadowing() {
        assert_eq!(
            run("let x = 1 in let x = 2 in x").unwrap(),
            Value::Int(2)
        );
        assert_eq!(
            run("let x = 1 in (let x = 2 in x) + x").unwrap(),
            Value::Int(3)
        );
    }
}
