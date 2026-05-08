//! A tiny expression-language evaluator.

use std::fmt;

/// A parsed expression node.
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
        op: UnaryOp,
        operand: Box<Expr>,
    },
}

/// Binary operators.
#[derive(Clone, Copy)]
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
#[derive(Clone, Copy)]
pub enum UnaryOp {
    Neg,
    Not,
}

/// A runtime value: either a 64-bit integer or a boolean.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Value {
    Int(i64),
    Bool(bool),
}

/// An error produced by [`parse`], carrying the byte offset at which parsing failed.
#[derive(Debug)]
pub struct ParseError {
    /// The byte offset within the input where parsing failed.
    pub offset: usize,
    /// A short human-readable message.
    pub message: String,
}

/// An error produced by [`eval`].
#[derive(Debug)]
pub enum EvalError {
    /// Operands had wrong types for the operation (e.g., `1 + true`).
    TypeError(String),
    /// Integer division (or modulo) by zero.
    DivisionByZero,
    /// Reference to an identifier that is not in scope.
    UnboundIdentifier(String),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{}", n),
            Value::Bool(b) => write!(f, "{}", b),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "parse error at offset {}: {}", self.offset, self.message)
    }
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvalError::TypeError(m) => write!(f, "type error: {}", m),
            EvalError::DivisionByZero => write!(f, "division by zero"),
            EvalError::UnboundIdentifier(name) => write!(f, "unbound identifier: {}", name),
        }
    }
}

// ---------- Lexer ----------

enum TokKind {
    Int(i64),
    Ident(String),
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
    LParen,
    RParen,
    Eq,
    Plus,
    Minus,
    Star,
    Slash,
    EqEq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

struct Token {
    kind: TokKind,
    offset: usize,
}

fn tokenize(input: &str) -> Result<Vec<Token>, ParseError> {
    let bytes = input.as_bytes();
    let mut i = 0;
    let mut tokens = Vec::new();
    while i < bytes.len() {
        let b = bytes[i];
        if b == b' ' || b == b'\t' || b == b'\n' || b == b'\r' {
            i += 1;
            continue;
        }
        let start = i;
        let kind = match b {
            b'(' => {
                i += 1;
                TokKind::LParen
            }
            b')' => {
                i += 1;
                TokKind::RParen
            }
            b'+' => {
                i += 1;
                TokKind::Plus
            }
            b'-' => {
                i += 1;
                TokKind::Minus
            }
            b'*' => {
                i += 1;
                TokKind::Star
            }
            b'/' => {
                i += 1;
                TokKind::Slash
            }
            b'=' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'=' {
                    i += 2;
                    TokKind::EqEq
                } else {
                    i += 1;
                    TokKind::Eq
                }
            }
            b'!' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'=' {
                    i += 2;
                    TokKind::Ne
                } else {
                    return Err(ParseError {
                        offset: i,
                        message: "expected '=' after '!'".to_string(),
                    });
                }
            }
            b'<' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'=' {
                    i += 2;
                    TokKind::Le
                } else {
                    i += 1;
                    TokKind::Lt
                }
            }
            b'>' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'=' {
                    i += 2;
                    TokKind::Ge
                } else {
                    i += 1;
                    TokKind::Gt
                }
            }
            b'0'..=b'9' => {
                let s = i;
                while i < bytes.len() && bytes[i].is_ascii_digit() {
                    i += 1;
                }
                let num_str = std::str::from_utf8(&bytes[s..i]).unwrap();
                let n: i64 = num_str.parse().map_err(|_| ParseError {
                    offset: s,
                    message: format!("invalid integer literal '{}'", num_str),
                })?;
                TokKind::Int(n)
            }
            b'a'..=b'z' | b'_' => {
                let s = i;
                while i < bytes.len() {
                    let c = bytes[i];
                    if c == b'_' || c.is_ascii_lowercase() || c.is_ascii_digit() {
                        i += 1;
                    } else {
                        break;
                    }
                }
                let id = std::str::from_utf8(&bytes[s..i]).unwrap();
                match id {
                    "let" => TokKind::Let,
                    "in" => TokKind::In,
                    "if" => TokKind::If,
                    "then" => TokKind::Then,
                    "else" => TokKind::Else,
                    "or" => TokKind::Or,
                    "and" => TokKind::And,
                    "not" => TokKind::Not,
                    "true" => TokKind::True,
                    "false" => TokKind::False,
                    _ => TokKind::Ident(id.to_string()),
                }
            }
            _ => {
                return Err(ParseError {
                    offset: i,
                    message: format!("unexpected character {:?}", b as char),
                });
            }
        };
        tokens.push(Token { kind, offset: start });
    }
    Ok(tokens)
}

// ---------- Parser ----------

struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
    end_offset: usize,
}

impl<'a> Parser<'a> {
    fn peek(&self) -> Option<&'a TokKind> {
        self.tokens.get(self.pos).map(|t| &t.kind)
    }

    fn current_offset(&self) -> usize {
        self.tokens
            .get(self.pos)
            .map(|t| t.offset)
            .unwrap_or(self.end_offset)
    }

    fn bump(&mut self) {
        self.pos += 1;
    }

    fn expect_eq(&mut self) -> Result<(), ParseError> {
        match self.peek() {
            Some(TokKind::Eq) => {
                self.bump();
                Ok(())
            }
            _ => Err(ParseError {
                offset: self.current_offset(),
                message: "expected '='".to_string(),
            }),
        }
    }

    fn expect_in(&mut self) -> Result<(), ParseError> {
        match self.peek() {
            Some(TokKind::In) => {
                self.bump();
                Ok(())
            }
            _ => Err(ParseError {
                offset: self.current_offset(),
                message: "expected 'in'".to_string(),
            }),
        }
    }

    fn expect_then(&mut self) -> Result<(), ParseError> {
        match self.peek() {
            Some(TokKind::Then) => {
                self.bump();
                Ok(())
            }
            _ => Err(ParseError {
                offset: self.current_offset(),
                message: "expected 'then'".to_string(),
            }),
        }
    }

    fn expect_else(&mut self) -> Result<(), ParseError> {
        match self.peek() {
            Some(TokKind::Else) => {
                self.bump();
                Ok(())
            }
            _ => Err(ParseError {
                offset: self.current_offset(),
                message: "expected 'else'".to_string(),
            }),
        }
    }

    fn expect_rparen(&mut self) -> Result<(), ParseError> {
        match self.peek() {
            Some(TokKind::RParen) => {
                self.bump();
                Ok(())
            }
            _ => Err(ParseError {
                offset: self.current_offset(),
                message: "expected ')'".to_string(),
            }),
        }
    }

    fn expect_ident(&mut self) -> Result<String, ParseError> {
        match self.peek() {
            Some(TokKind::Ident(s)) => {
                let s = s.clone();
                self.bump();
                Ok(s)
            }
            _ => Err(ParseError {
                offset: self.current_offset(),
                message: "expected identifier".to_string(),
            }),
        }
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        match self.peek() {
            Some(TokKind::Let) => self.parse_let(),
            Some(TokKind::If) => self.parse_if(),
            _ => self.parse_or(),
        }
    }

    fn parse_let(&mut self) -> Result<Expr, ParseError> {
        self.bump(); // 'let'
        let name = self.expect_ident()?;
        self.expect_eq()?;
        let value = self.parse_expr()?;
        self.expect_in()?;
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
        self.expect_then()?;
        let then_branch = self.parse_expr()?;
        self.expect_else()?;
        let else_branch = self.parse_expr()?;
        Ok(Expr::If {
            cond: Box::new(cond),
            then_branch: Box::new(then_branch),
            else_branch: Box::new(else_branch),
        })
    }

    fn parse_or(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_and()?;
        while matches!(self.peek(), Some(TokKind::Or)) {
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
        while matches!(self.peek(), Some(TokKind::And)) {
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
        if matches!(self.peek(), Some(TokKind::Not)) {
            self.bump();
            let operand = self.parse_not()?;
            Ok(Expr::Unary {
                op: UnaryOp::Not,
                operand: Box::new(operand),
            })
        } else {
            self.parse_cmp()
        }
    }

    fn parse_cmp(&mut self) -> Result<Expr, ParseError> {
        let lhs = self.parse_add()?;
        let op = match self.peek() {
            Some(TokKind::EqEq) => BinOp::Eq,
            Some(TokKind::Ne) => BinOp::Ne,
            Some(TokKind::Lt) => BinOp::Lt,
            Some(TokKind::Le) => BinOp::Le,
            Some(TokKind::Gt) => BinOp::Gt,
            Some(TokKind::Ge) => BinOp::Ge,
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
                Some(TokKind::Plus) => BinOp::Add,
                Some(TokKind::Minus) => BinOp::Sub,
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
                Some(TokKind::Star) => BinOp::Mul,
                Some(TokKind::Slash) => BinOp::Div,
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
        if matches!(self.peek(), Some(TokKind::Minus)) {
            self.bump();
            let operand = self.parse_unary()?;
            Ok(Expr::Unary {
                op: UnaryOp::Neg,
                operand: Box::new(operand),
            })
        } else {
            self.parse_atom()
        }
    }

    fn parse_atom(&mut self) -> Result<Expr, ParseError> {
        match self.peek() {
            Some(TokKind::Int(n)) => {
                let n = *n;
                self.bump();
                Ok(Expr::Int(n))
            }
            Some(TokKind::True) => {
                self.bump();
                Ok(Expr::Bool(true))
            }
            Some(TokKind::False) => {
                self.bump();
                Ok(Expr::Bool(false))
            }
            Some(TokKind::Ident(s)) => {
                let s = s.clone();
                self.bump();
                Ok(Expr::Ident(s))
            }
            Some(TokKind::LParen) => {
                self.bump();
                let e = self.parse_expr()?;
                self.expect_rparen()?;
                Ok(e)
            }
            _ => Err(ParseError {
                offset: self.current_offset(),
                message: "expected expression".to_string(),
            }),
        }
    }
}

/// Parse the input string into an [`Expr`].
///
/// Returns a [`ParseError`] (with the byte offset of the failure) if the input
/// does not match the grammar.
pub fn parse(input: &str) -> Result<Expr, ParseError> {
    let tokens = tokenize(input)?;
    let mut parser = Parser {
        tokens: &tokens,
        pos: 0,
        end_offset: input.len(),
    };
    let expr = parser.parse_expr()?;
    if parser.pos < parser.tokens.len() {
        return Err(ParseError {
            offset: parser.current_offset(),
            message: "unexpected trailing input".to_string(),
        });
    }
    Ok(expr)
}

// ---------- Evaluator ----------

struct Env<'a> {
    parent: Option<&'a Env<'a>>,
    name: &'a str,
    value: Value,
}

impl<'a> Env<'a> {
    fn lookup(&self, name: &str) -> Option<Value> {
        if self.name == name {
            Some(self.value)
        } else {
            self.parent.and_then(|p| p.lookup(name))
        }
    }
}

fn eval_in(expr: &Expr, env: Option<&Env>) -> Result<Value, EvalError> {
    match expr {
        Expr::Int(n) => Ok(Value::Int(*n)),
        Expr::Bool(b) => Ok(Value::Bool(*b)),
        Expr::Ident(name) => env
            .and_then(|e| e.lookup(name))
            .ok_or_else(|| EvalError::UnboundIdentifier(name.clone())),
        Expr::Let { name, value, body } => {
            let v = eval_in(value, env)?;
            let new_env = Env {
                parent: env,
                name,
                value: v,
            };
            eval_in(body, Some(&new_env))
        }
        Expr::If {
            cond,
            then_branch,
            else_branch,
        } => match eval_in(cond, env)? {
            Value::Bool(true) => eval_in(then_branch, env),
            Value::Bool(false) => eval_in(else_branch, env),
            Value::Int(_) => Err(EvalError::TypeError(
                "'if' condition must be a boolean".to_string(),
            )),
        },
        Expr::Unary { op, operand } => {
            let v = eval_in(operand, env)?;
            match (op, v) {
                (UnaryOp::Neg, Value::Int(n)) => Ok(Value::Int(n.wrapping_neg())),
                (UnaryOp::Neg, Value::Bool(_)) => Err(EvalError::TypeError(
                    "unary '-' expects an integer".to_string(),
                )),
                (UnaryOp::Not, Value::Bool(b)) => Ok(Value::Bool(!b)),
                (UnaryOp::Not, Value::Int(_)) => Err(EvalError::TypeError(
                    "'not' expects a boolean".to_string(),
                )),
            }
        }
        Expr::Binary { op, lhs, rhs } => match op {
            BinOp::And => {
                let l = eval_in(lhs, env)?;
                let lb = match l {
                    Value::Bool(b) => b,
                    _ => return Err(EvalError::TypeError("'and' expects booleans".to_string())),
                };
                if !lb {
                    return Ok(Value::Bool(false));
                }
                match eval_in(rhs, env)? {
                    Value::Bool(b) => Ok(Value::Bool(b)),
                    _ => Err(EvalError::TypeError("'and' expects booleans".to_string())),
                }
            }
            BinOp::Or => {
                let l = eval_in(lhs, env)?;
                let lb = match l {
                    Value::Bool(b) => b,
                    _ => return Err(EvalError::TypeError("'or' expects booleans".to_string())),
                };
                if lb {
                    return Ok(Value::Bool(true));
                }
                match eval_in(rhs, env)? {
                    Value::Bool(b) => Ok(Value::Bool(b)),
                    _ => Err(EvalError::TypeError("'or' expects booleans".to_string())),
                }
            }
            _ => {
                let l = eval_in(lhs, env)?;
                let r = eval_in(rhs, env)?;
                apply_binop(*op, l, r)
            }
        },
    }
}

fn apply_binop(op: BinOp, l: Value, r: Value) -> Result<Value, EvalError> {
    match op {
        BinOp::Add => match (l, r) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a.wrapping_add(b))),
            _ => Err(EvalError::TypeError("'+' expects integers".to_string())),
        },
        BinOp::Sub => match (l, r) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a.wrapping_sub(b))),
            _ => Err(EvalError::TypeError("'-' expects integers".to_string())),
        },
        BinOp::Mul => match (l, r) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a.wrapping_mul(b))),
            _ => Err(EvalError::TypeError("'*' expects integers".to_string())),
        },
        BinOp::Div => match (l, r) {
            (Value::Int(_), Value::Int(0)) => Err(EvalError::DivisionByZero),
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a.wrapping_div(b))),
            _ => Err(EvalError::TypeError("'/' expects integers".to_string())),
        },
        BinOp::Eq => match (l, r) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a == b)),
            (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a == b)),
            _ => Err(EvalError::TypeError(
                "'==' requires same-type operands".to_string(),
            )),
        },
        BinOp::Ne => match (l, r) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a != b)),
            (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a != b)),
            _ => Err(EvalError::TypeError(
                "'!=' requires same-type operands".to_string(),
            )),
        },
        BinOp::Lt => match (l, r) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a < b)),
            _ => Err(EvalError::TypeError("'<' expects integers".to_string())),
        },
        BinOp::Le => match (l, r) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a <= b)),
            _ => Err(EvalError::TypeError("'<=' expects integers".to_string())),
        },
        BinOp::Gt => match (l, r) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a > b)),
            _ => Err(EvalError::TypeError("'>' expects integers".to_string())),
        },
        BinOp::Ge => match (l, r) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a >= b)),
            _ => Err(EvalError::TypeError("'>=' expects integers".to_string())),
        },
        BinOp::And | BinOp::Or => unreachable!(),
    }
}

/// Evaluate an expression to a [`Value`].
///
/// Returns an [`EvalError`] for type mismatches, division by zero, or unbound
/// identifier references.
pub fn eval(expr: &Expr) -> Result<Value, EvalError> {
    eval_in(expr, None)
}
