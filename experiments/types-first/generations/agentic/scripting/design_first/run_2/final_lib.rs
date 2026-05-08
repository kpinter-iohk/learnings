use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Int(i64),
    Bool(bool),
    Var(String),
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
    Unary {
        op: UnaryOp,
        operand: Box<Expr>,
    },
    Binary {
        op: BinaryOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    And,
    Or,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub offset: usize,
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "parse error at byte {}: {}", self.offset, self.message)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvalError {
    TypeError(String),
    DivisionByZero,
    UnboundIdentifier(String),
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvalError::TypeError(msg) => write!(f, "type error: {}", msg),
            EvalError::DivisionByZero => write!(f, "division by zero"),
            EvalError::UnboundIdentifier(name) => write!(f, "unbound identifier: {}", name),
        }
    }
}

// ---------- Lexer ----------

#[derive(Debug, Clone, PartialEq, Eq)]
enum Tok {
    Int(i64),
    Ident(String),
    KwLet,
    KwIn,
    KwIf,
    KwThen,
    KwElse,
    KwOr,
    KwAnd,
    KwNot,
    KwTrue,
    KwFalse,
    Eq,    // =
    EqEq,  // ==
    NotEq, // !=
    Lt,
    Le,
    Gt,
    Ge,
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
    Eof,
}

#[derive(Debug, Clone)]
struct Token {
    kind: Tok,
    offset: usize,
}

fn lex(input: &str) -> Result<Vec<Token>, ParseError> {
    let bytes = input.as_bytes();
    let mut i = 0;
    let mut tokens = Vec::new();
    while i < bytes.len() {
        let c = bytes[i];
        if c.is_ascii_whitespace() {
            i += 1;
            continue;
        }
        let start = i;
        match c {
            b'(' => {
                tokens.push(Token { kind: Tok::LParen, offset: start });
                i += 1;
            }
            b')' => {
                tokens.push(Token { kind: Tok::RParen, offset: start });
                i += 1;
            }
            b'+' => {
                tokens.push(Token { kind: Tok::Plus, offset: start });
                i += 1;
            }
            b'-' => {
                tokens.push(Token { kind: Tok::Minus, offset: start });
                i += 1;
            }
            b'*' => {
                tokens.push(Token { kind: Tok::Star, offset: start });
                i += 1;
            }
            b'/' => {
                tokens.push(Token { kind: Tok::Slash, offset: start });
                i += 1;
            }
            b'=' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'=' {
                    tokens.push(Token { kind: Tok::EqEq, offset: start });
                    i += 2;
                } else {
                    tokens.push(Token { kind: Tok::Eq, offset: start });
                    i += 1;
                }
            }
            b'!' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'=' {
                    tokens.push(Token { kind: Tok::NotEq, offset: start });
                    i += 2;
                } else {
                    return Err(ParseError {
                        offset: start,
                        message: "unexpected character '!'".to_string(),
                    });
                }
            }
            b'<' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'=' {
                    tokens.push(Token { kind: Tok::Le, offset: start });
                    i += 2;
                } else {
                    tokens.push(Token { kind: Tok::Lt, offset: start });
                    i += 1;
                }
            }
            b'>' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'=' {
                    tokens.push(Token { kind: Tok::Ge, offset: start });
                    i += 2;
                } else {
                    tokens.push(Token { kind: Tok::Gt, offset: start });
                    i += 1;
                }
            }
            b'0'..=b'9' => {
                let mut j = i;
                while j < bytes.len() && bytes[j].is_ascii_digit() {
                    j += 1;
                }
                let s = &input[i..j];
                let n: i64 = s.parse().map_err(|_| ParseError {
                    offset: start,
                    message: format!("invalid integer literal '{}'", s),
                })?;
                tokens.push(Token { kind: Tok::Int(n), offset: start });
                i = j;
            }
            b'a'..=b'z' | b'_' => {
                let mut j = i;
                while j < bytes.len()
                    && (bytes[j].is_ascii_lowercase()
                        || bytes[j].is_ascii_digit()
                        || bytes[j] == b'_')
                {
                    j += 1;
                }
                let s = &input[i..j];
                let kind = match s {
                    "let" => Tok::KwLet,
                    "in" => Tok::KwIn,
                    "if" => Tok::KwIf,
                    "then" => Tok::KwThen,
                    "else" => Tok::KwElse,
                    "or" => Tok::KwOr,
                    "and" => Tok::KwAnd,
                    "not" => Tok::KwNot,
                    "true" => Tok::KwTrue,
                    "false" => Tok::KwFalse,
                    _ => Tok::Ident(s.to_string()),
                };
                tokens.push(Token { kind, offset: start });
                i = j;
            }
            _ => {
                return Err(ParseError {
                    offset: start,
                    message: format!("unexpected character '{}'", c as char),
                });
            }
        }
    }
    tokens.push(Token { kind: Tok::Eof, offset: bytes.len() });
    Ok(tokens)
}

// ---------- Parser ----------

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn advance(&mut self) -> Token {
        let t = self.tokens[self.pos].clone();
        if self.pos + 1 < self.tokens.len() {
            self.pos += 1;
        }
        t
    }

    fn expect(&mut self, kind: &Tok, what: &str) -> Result<Token, ParseError> {
        if std::mem::discriminant(&self.peek().kind) == std::mem::discriminant(kind) {
            Ok(self.advance())
        } else {
            Err(ParseError {
                offset: self.peek().offset,
                message: format!("expected {}", what),
            })
        }
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        match &self.peek().kind {
            Tok::KwLet => self.parse_let(),
            Tok::KwIf => self.parse_if(),
            _ => self.parse_or(),
        }
    }

    fn parse_let(&mut self) -> Result<Expr, ParseError> {
        self.advance(); // let
        let name_tok = self.advance();
        let name = match name_tok.kind {
            Tok::Ident(s) => s,
            _ => {
                return Err(ParseError {
                    offset: name_tok.offset,
                    message: "expected identifier after 'let'".to_string(),
                });
            }
        };
        self.expect(&Tok::Eq, "'=' after identifier in let")?;
        let value = self.parse_expr()?;
        self.expect(&Tok::KwIn, "'in' in let expression")?;
        let body = self.parse_expr()?;
        Ok(Expr::Let {
            name,
            value: Box::new(value),
            body: Box::new(body),
        })
    }

    fn parse_if(&mut self) -> Result<Expr, ParseError> {
        self.advance(); // if
        let cond = self.parse_expr()?;
        self.expect(&Tok::KwThen, "'then' in if expression")?;
        let then_branch = self.parse_expr()?;
        self.expect(&Tok::KwElse, "'else' in if expression")?;
        let else_branch = self.parse_expr()?;
        Ok(Expr::If {
            cond: Box::new(cond),
            then_branch: Box::new(then_branch),
            else_branch: Box::new(else_branch),
        })
    }

    fn parse_or(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_and()?;
        while matches!(self.peek().kind, Tok::KwOr) {
            self.advance();
            let rhs = self.parse_and()?;
            lhs = Expr::Binary {
                op: BinaryOp::Or,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            };
        }
        Ok(lhs)
    }

    fn parse_and(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_not()?;
        while matches!(self.peek().kind, Tok::KwAnd) {
            self.advance();
            let rhs = self.parse_not()?;
            lhs = Expr::Binary {
                op: BinaryOp::And,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            };
        }
        Ok(lhs)
    }

    fn parse_not(&mut self) -> Result<Expr, ParseError> {
        if matches!(self.peek().kind, Tok::KwNot) {
            self.advance();
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
        let op = match self.peek().kind {
            Tok::EqEq => Some(BinaryOp::Eq),
            Tok::NotEq => Some(BinaryOp::Ne),
            Tok::Lt => Some(BinaryOp::Lt),
            Tok::Le => Some(BinaryOp::Le),
            Tok::Gt => Some(BinaryOp::Gt),
            Tok::Ge => Some(BinaryOp::Ge),
            _ => None,
        };
        if let Some(op) = op {
            self.advance();
            let rhs = self.parse_add()?;
            Ok(Expr::Binary {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            })
        } else {
            Ok(lhs)
        }
    }

    fn parse_add(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_mul()?;
        loop {
            let op = match self.peek().kind {
                Tok::Plus => BinaryOp::Add,
                Tok::Minus => BinaryOp::Sub,
                _ => break,
            };
            self.advance();
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
            let op = match self.peek().kind {
                Tok::Star => BinaryOp::Mul,
                Tok::Slash => BinaryOp::Div,
                _ => break,
            };
            self.advance();
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
        if matches!(self.peek().kind, Tok::Minus) {
            self.advance();
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
        let tok = self.advance();
        match tok.kind {
            Tok::Int(n) => Ok(Expr::Int(n)),
            Tok::KwTrue => Ok(Expr::Bool(true)),
            Tok::KwFalse => Ok(Expr::Bool(false)),
            Tok::Ident(s) => Ok(Expr::Var(s)),
            Tok::LParen => {
                let e = self.parse_expr()?;
                self.expect(&Tok::RParen, "')'")?;
                Ok(e)
            }
            _ => Err(ParseError {
                offset: tok.offset,
                message: "expected expression".to_string(),
            }),
        }
    }
}

pub fn parse(input: &str) -> Result<Expr, ParseError> {
    let tokens = lex(input)?;
    let mut p = Parser { tokens, pos: 0 };
    let e = p.parse_expr()?;
    if !matches!(p.peek().kind, Tok::Eof) {
        return Err(ParseError {
            offset: p.peek().offset,
            message: "trailing input after expression".to_string(),
        });
    }
    Ok(e)
}

// ---------- Evaluator ----------

struct Env<'a> {
    name: &'a str,
    value: Value,
    parent: Option<&'a Env<'a>>,
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

fn eval_with(expr: &Expr, env: Option<&Env<'_>>) -> Result<Value, EvalError> {
    match expr {
        Expr::Int(n) => Ok(Value::Int(*n)),
        Expr::Bool(b) => Ok(Value::Bool(*b)),
        Expr::Var(name) => env
            .and_then(|e| e.lookup(name))
            .ok_or_else(|| EvalError::UnboundIdentifier(name.clone())),
        Expr::Let { name, value, body } => {
            let v = eval_with(value, env)?;
            let new_env = Env {
                name,
                value: v,
                parent: env,
            };
            eval_with(body, Some(&new_env))
        }
        Expr::If {
            cond,
            then_branch,
            else_branch,
        } => {
            let c = eval_with(cond, env)?;
            match c {
                Value::Bool(true) => eval_with(then_branch, env),
                Value::Bool(false) => eval_with(else_branch, env),
                Value::Int(_) => Err(EvalError::TypeError(
                    "if condition must be a boolean".to_string(),
                )),
            }
        }
        Expr::Unary { op, operand } => {
            let v = eval_with(operand, env)?;
            match (op, v) {
                (UnaryOp::Neg, Value::Int(n)) => Ok(Value::Int(-n)),
                (UnaryOp::Neg, Value::Bool(_)) => Err(EvalError::TypeError(
                    "unary '-' requires an integer".to_string(),
                )),
                (UnaryOp::Not, Value::Bool(b)) => Ok(Value::Bool(!b)),
                (UnaryOp::Not, Value::Int(_)) => Err(EvalError::TypeError(
                    "'not' requires a boolean".to_string(),
                )),
            }
        }
        Expr::Binary { op, lhs, rhs } => eval_binary(*op, lhs, rhs, env),
    }
}

fn eval_binary(
    op: BinaryOp,
    lhs: &Expr,
    rhs: &Expr,
    env: Option<&Env<'_>>,
) -> Result<Value, EvalError> {
    // Short-circuiting boolean operators.
    if matches!(op, BinaryOp::And | BinaryOp::Or) {
        let l = eval_with(lhs, env)?;
        let lb = match l {
            Value::Bool(b) => b,
            Value::Int(_) => {
                return Err(EvalError::TypeError(format!(
                    "'{}' requires booleans",
                    if op == BinaryOp::And { "and" } else { "or" }
                )));
            }
        };
        match op {
            BinaryOp::And => {
                if !lb {
                    return Ok(Value::Bool(false));
                }
            }
            BinaryOp::Or => {
                if lb {
                    return Ok(Value::Bool(true));
                }
            }
            _ => unreachable!(),
        }
        let r = eval_with(rhs, env)?;
        match r {
            Value::Bool(b) => Ok(Value::Bool(b)),
            Value::Int(_) => Err(EvalError::TypeError(format!(
                "'{}' requires booleans",
                if op == BinaryOp::And { "and" } else { "or" }
            ))),
        }
    } else {
        let l = eval_with(lhs, env)?;
        let r = eval_with(rhs, env)?;
        match op {
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div => match (l, r) {
                (Value::Int(a), Value::Int(b)) => match op {
                    BinaryOp::Add => Ok(Value::Int(a.wrapping_add(b))),
                    BinaryOp::Sub => Ok(Value::Int(a.wrapping_sub(b))),
                    BinaryOp::Mul => Ok(Value::Int(a.wrapping_mul(b))),
                    BinaryOp::Div => {
                        if b == 0 {
                            Err(EvalError::DivisionByZero)
                        } else {
                            Ok(Value::Int(a.wrapping_div(b)))
                        }
                    }
                    _ => unreachable!(),
                },
                _ => Err(EvalError::TypeError(
                    "arithmetic operators require integers".to_string(),
                )),
            },
            BinaryOp::Eq | BinaryOp::Ne => match (l, r) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(if op == BinaryOp::Eq {
                    a == b
                } else {
                    a != b
                })),
                (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(if op == BinaryOp::Eq {
                    a == b
                } else {
                    a != b
                })),
                _ => Err(EvalError::TypeError(
                    "equality operands must have the same type".to_string(),
                )),
            },
            BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => match (l, r) {
                (Value::Int(a), Value::Int(b)) => {
                    let result = match op {
                        BinaryOp::Lt => a < b,
                        BinaryOp::Le => a <= b,
                        BinaryOp::Gt => a > b,
                        BinaryOp::Ge => a >= b,
                        _ => unreachable!(),
                    };
                    Ok(Value::Bool(result))
                }
                _ => Err(EvalError::TypeError(
                    "ordering comparisons require integers".to_string(),
                )),
            },
            BinaryOp::And | BinaryOp::Or => unreachable!(),
        }
    }
}

pub fn eval(expr: &Expr) -> Result<Value, EvalError> {
    eval_with(expr, None)
}
