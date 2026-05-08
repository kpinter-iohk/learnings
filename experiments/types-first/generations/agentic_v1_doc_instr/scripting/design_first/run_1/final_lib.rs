//! Tiny expression-language evaluator.
//!
//! Public surface: [`parse`], [`eval`], [`Expr`], [`Value`], [`ParseError`],
//! [`EvalError`].

use std::fmt;

/// A parsed expression tree.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Integer literal.
    Int(i64),
    /// Boolean literal.
    Bool(bool),
    /// Identifier (variable reference).
    Ident(String),
    /// `let NAME = VALUE in BODY`.
    Let {
        name: String,
        value: Box<Expr>,
        body: Box<Expr>,
    },
    /// `if COND then THEN else ELSE`.
    If {
        cond: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Box<Expr>,
    },
    /// Binary operation.
    Binary {
        op: BinOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    /// Unary operation.
    Unary { op: UnaryOp, expr: Box<Expr> },
}

/// Binary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Or,
    And,
    Eq,
    Neq,
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
pub enum UnaryOp {
    Neg,
    Not,
}

/// A runtime value.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Bool(bool),
}

/// Parse error. Carries the byte offset at which parsing failed.
#[derive(Clone, PartialEq)]
pub struct ParseError {
    pub offset: usize,
    pub message: String,
}

/// Evaluation error.
#[derive(Clone, PartialEq)]
pub enum EvalError {
    /// Operand types incompatible with the operator.
    TypeError(String),
    /// Integer division by zero.
    DivisionByZero,
    /// Reference to an identifier that has not been bound by an enclosing `let`.
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

impl fmt::Debug for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ParseError")
            .field("offset", &self.offset)
            .field("message", &self.message)
            .finish()
    }
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

impl fmt::Debug for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvalError::TypeError(msg) => f.debug_tuple("TypeError").field(msg).finish(),
            EvalError::DivisionByZero => f.write_str("DivisionByZero"),
            EvalError::UnboundIdentifier(name) => {
                f.debug_tuple("UnboundIdentifier").field(name).finish()
            }
        }
    }
}

// ----------------------------------------------------------------------------
// Tokenizer
// ----------------------------------------------------------------------------

#[derive(Clone)]
struct Token {
    kind: TokKind,
    offset: usize,
}

#[derive(Clone)]
enum TokKind {
    Int(i64),
    Ident(String),
    Op(&'static str),
}

fn tokenize(input: &str) -> Result<Vec<Token>, ParseError> {
    let bytes = input.as_bytes();
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if matches!(b, b' ' | b'\t' | b'\n' | b'\r') {
            i += 1;
            continue;
        }

        // Two-character operators.
        if i + 1 < bytes.len() {
            let two = &bytes[i..i + 2];
            let op2: Option<&'static str> = match two {
                b"==" => Some("=="),
                b"!=" => Some("!="),
                b"<=" => Some("<="),
                b">=" => Some(">="),
                _ => None,
            };
            if let Some(op) = op2 {
                tokens.push(Token { kind: TokKind::Op(op), offset: i });
                i += 2;
                continue;
            }
        }

        // Single-character operators.
        let op1: Option<&'static str> = match b {
            b'+' => Some("+"),
            b'-' => Some("-"),
            b'*' => Some("*"),
            b'/' => Some("/"),
            b'<' => Some("<"),
            b'>' => Some(">"),
            b'=' => Some("="),
            b'(' => Some("("),
            b')' => Some(")"),
            _ => None,
        };
        if let Some(op) = op1 {
            tokens.push(Token { kind: TokKind::Op(op), offset: i });
            i += 1;
            continue;
        }

        // Integer literal.
        if b.is_ascii_digit() {
            let start = i;
            let mut j = i;
            while j < bytes.len() && bytes[j].is_ascii_digit() {
                j += 1;
            }
            if j < bytes.len()
                && (bytes[j].is_ascii_alphabetic() || bytes[j] == b'_')
            {
                return Err(ParseError {
                    offset: j,
                    message: "unexpected character after integer literal".to_string(),
                });
            }
            let s = std::str::from_utf8(&bytes[start..j]).unwrap();
            let n: i64 = s.parse().map_err(|_| ParseError {
                offset: start,
                message: format!("invalid integer literal: {}", s),
            })?;
            tokens.push(Token { kind: TokKind::Int(n), offset: start });
            i = j;
            continue;
        }

        // Identifier (lowercase letters / underscore).
        if b == b'_' || b.is_ascii_lowercase() {
            let start = i;
            let mut j = i;
            while j < bytes.len()
                && (bytes[j] == b'_'
                    || bytes[j].is_ascii_lowercase()
                    || bytes[j].is_ascii_digit())
            {
                j += 1;
            }
            let s = std::str::from_utf8(&bytes[start..j]).unwrap().to_string();
            tokens.push(Token { kind: TokKind::Ident(s), offset: start });
            i = j;
            continue;
        }

        return Err(ParseError {
            offset: i,
            message: format!("unexpected character {:?}", b as char),
        });
    }
    Ok(tokens)
}

fn is_keyword(s: &str) -> bool {
    matches!(
        s,
        "let" | "in" | "if" | "then" | "else" | "or" | "and" | "not" | "true" | "false"
    )
}

// ----------------------------------------------------------------------------
// Parser
// ----------------------------------------------------------------------------

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    end_offset: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn current_offset(&self) -> usize {
        self.peek().map(|t| t.offset).unwrap_or(self.end_offset)
    }

    fn peek_is_op(&self, op: &str) -> bool {
        match self.peek() {
            Some(Token { kind: TokKind::Op(o), .. }) => *o == op,
            _ => false,
        }
    }

    fn peek_is_keyword(&self, kw: &str) -> bool {
        match self.peek() {
            Some(Token { kind: TokKind::Ident(s), .. }) => s == kw,
            _ => false,
        }
    }

    fn eat_op(&mut self, op: &str) -> bool {
        if self.peek_is_op(op) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn eat_keyword(&mut self, kw: &str) -> bool {
        if self.peek_is_keyword(kw) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn expect_op(&mut self, op: &str) -> Result<(), ParseError> {
        if self.eat_op(op) {
            Ok(())
        } else {
            Err(ParseError {
                offset: self.current_offset(),
                message: format!("expected '{}'", op),
            })
        }
    }

    fn expect_keyword(&mut self, kw: &str) -> Result<(), ParseError> {
        if self.eat_keyword(kw) {
            Ok(())
        } else {
            Err(ParseError {
                offset: self.current_offset(),
                message: format!("expected '{}'", kw),
            })
        }
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        if self.peek_is_keyword("let") {
            self.parse_let()
        } else if self.peek_is_keyword("if") {
            self.parse_if()
        } else {
            self.parse_or()
        }
    }

    fn parse_let(&mut self) -> Result<Expr, ParseError> {
        self.expect_keyword("let")?;
        let name_offset = self.current_offset();
        let name = match self.peek() {
            Some(Token { kind: TokKind::Ident(s), .. }) => {
                if is_keyword(s) {
                    return Err(ParseError {
                        offset: name_offset,
                        message: format!("expected identifier, found keyword '{}'", s),
                    });
                }
                s.clone()
            }
            _ => {
                return Err(ParseError {
                    offset: name_offset,
                    message: "expected identifier".to_string(),
                });
            }
        };
        self.pos += 1;
        self.expect_op("=")?;
        let value = self.parse_expr()?;
        self.expect_keyword("in")?;
        let body = self.parse_expr()?;
        Ok(Expr::Let {
            name,
            value: Box::new(value),
            body: Box::new(body),
        })
    }

    fn parse_if(&mut self) -> Result<Expr, ParseError> {
        self.expect_keyword("if")?;
        let cond = self.parse_expr()?;
        self.expect_keyword("then")?;
        let then_branch = self.parse_expr()?;
        self.expect_keyword("else")?;
        let else_branch = self.parse_expr()?;
        Ok(Expr::If {
            cond: Box::new(cond),
            then_branch: Box::new(then_branch),
            else_branch: Box::new(else_branch),
        })
    }

    fn parse_or(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_and()?;
        while self.eat_keyword("or") {
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
        while self.eat_keyword("and") {
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
        if self.eat_keyword("not") {
            let inner = self.parse_not()?;
            Ok(Expr::Unary {
                op: UnaryOp::Not,
                expr: Box::new(inner),
            })
        } else {
            self.parse_cmp()
        }
    }

    fn parse_cmp(&mut self) -> Result<Expr, ParseError> {
        let lhs = self.parse_add()?;
        let op = if self.peek_is_op("==") {
            Some(BinOp::Eq)
        } else if self.peek_is_op("!=") {
            Some(BinOp::Neq)
        } else if self.peek_is_op("<=") {
            Some(BinOp::Le)
        } else if self.peek_is_op(">=") {
            Some(BinOp::Ge)
        } else if self.peek_is_op("<") {
            Some(BinOp::Lt)
        } else if self.peek_is_op(">") {
            Some(BinOp::Gt)
        } else {
            None
        };
        if let Some(op) = op {
            self.pos += 1;
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
            let op = if self.peek_is_op("+") {
                Some(BinOp::Add)
            } else if self.peek_is_op("-") {
                Some(BinOp::Sub)
            } else {
                None
            };
            match op {
                Some(op) => {
                    self.pos += 1;
                    let rhs = self.parse_mul()?;
                    lhs = Expr::Binary {
                        op,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    };
                }
                None => break,
            }
        }
        Ok(lhs)
    }

    fn parse_mul(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_unary()?;
        loop {
            let op = if self.peek_is_op("*") {
                Some(BinOp::Mul)
            } else if self.peek_is_op("/") {
                Some(BinOp::Div)
            } else {
                None
            };
            match op {
                Some(op) => {
                    self.pos += 1;
                    let rhs = self.parse_unary()?;
                    lhs = Expr::Binary {
                        op,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    };
                }
                None => break,
            }
        }
        Ok(lhs)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        if self.eat_op("-") {
            let inner = self.parse_unary()?;
            Ok(Expr::Unary {
                op: UnaryOp::Neg,
                expr: Box::new(inner),
            })
        } else {
            self.parse_atom()
        }
    }

    fn parse_atom(&mut self) -> Result<Expr, ParseError> {
        if self.eat_op("(") {
            let e = self.parse_expr()?;
            self.expect_op(")")?;
            return Ok(e);
        }
        let tok = match self.peek() {
            Some(t) => t.clone(),
            None => {
                return Err(ParseError {
                    offset: self.end_offset,
                    message: "unexpected end of input".to_string(),
                });
            }
        };
        match tok.kind {
            TokKind::Int(n) => {
                self.pos += 1;
                Ok(Expr::Int(n))
            }
            TokKind::Ident(s) => {
                if s == "true" {
                    self.pos += 1;
                    Ok(Expr::Bool(true))
                } else if s == "false" {
                    self.pos += 1;
                    Ok(Expr::Bool(false))
                } else if is_keyword(&s) {
                    Err(ParseError {
                        offset: tok.offset,
                        message: format!("unexpected keyword '{}'", s),
                    })
                } else {
                    self.pos += 1;
                    Ok(Expr::Ident(s))
                }
            }
            TokKind::Op(_) => Err(ParseError {
                offset: tok.offset,
                message: "expected expression".to_string(),
            }),
        }
    }
}

/// Parse `input` into an [`Expr`].
pub fn parse(input: &str) -> Result<Expr, ParseError> {
    let tokens = tokenize(input)?;
    let mut p = Parser {
        tokens,
        pos: 0,
        end_offset: input.len(),
    };
    let e = p.parse_expr()?;
    if p.pos < p.tokens.len() {
        return Err(ParseError {
            offset: p.tokens[p.pos].offset,
            message: "unexpected trailing input".to_string(),
        });
    }
    Ok(e)
}

// ----------------------------------------------------------------------------
// Evaluator
// ----------------------------------------------------------------------------

enum Env<'a> {
    Empty,
    Cons {
        name: &'a str,
        value: Value,
        rest: &'a Env<'a>,
    },
}

impl<'a> Env<'a> {
    fn lookup(&self, name: &str) -> Option<&Value> {
        match self {
            Env::Empty => None,
            Env::Cons { name: n, value, rest } => {
                if *n == name {
                    Some(value)
                } else {
                    rest.lookup(name)
                }
            }
        }
    }
}

fn type_name(v: &Value) -> &'static str {
    match v {
        Value::Int(_) => "int",
        Value::Bool(_) => "bool",
    }
}

fn type_err_binary(op: &str, l: &Value, r: &Value) -> EvalError {
    EvalError::TypeError(format!(
        "operator '{}' is not defined for {} and {}",
        op,
        type_name(l),
        type_name(r),
    ))
}

fn eval_in(expr: &Expr, env: &Env) -> Result<Value, EvalError> {
    match expr {
        Expr::Int(n) => Ok(Value::Int(*n)),
        Expr::Bool(b) => Ok(Value::Bool(*b)),
        Expr::Ident(name) => match env.lookup(name) {
            Some(v) => Ok(v.clone()),
            None => Err(EvalError::UnboundIdentifier(name.clone())),
        },
        Expr::Let { name, value, body } => {
            let v = eval_in(value, env)?;
            let new_env = Env::Cons {
                name: name.as_str(),
                value: v,
                rest: env,
            };
            eval_in(body, &new_env)
        }
        Expr::If { cond, then_branch, else_branch } => {
            let c = eval_in(cond, env)?;
            match c {
                Value::Bool(true) => eval_in(then_branch, env),
                Value::Bool(false) => eval_in(else_branch, env),
                Value::Int(_) => Err(EvalError::TypeError(
                    "'if' condition must be a bool, got int".to_string(),
                )),
            }
        }
        Expr::Unary { op, expr } => {
            let v = eval_in(expr, env)?;
            match op {
                UnaryOp::Neg => match v {
                    Value::Int(n) => Ok(Value::Int(n.wrapping_neg())),
                    Value::Bool(_) => Err(EvalError::TypeError(
                        "unary '-' requires int, got bool".to_string(),
                    )),
                },
                UnaryOp::Not => match v {
                    Value::Bool(b) => Ok(Value::Bool(!b)),
                    Value::Int(_) => Err(EvalError::TypeError(
                        "'not' requires bool, got int".to_string(),
                    )),
                },
            }
        }
        Expr::Binary { op, lhs, rhs } => {
            // Short-circuit logical operators.
            if matches!(op, BinOp::And | BinOp::Or) {
                let l = eval_in(lhs, env)?;
                let lb = match l {
                    Value::Bool(b) => b,
                    Value::Int(_) => {
                        let name = if matches!(op, BinOp::And) { "and" } else { "or" };
                        return Err(EvalError::TypeError(format!(
                            "'{}' requires bool, got int on left",
                            name
                        )));
                    }
                };
                let short = match op {
                    BinOp::And => !lb,
                    BinOp::Or => lb,
                    _ => unreachable!(),
                };
                if short {
                    return Ok(Value::Bool(lb));
                }
                let r = eval_in(rhs, env)?;
                return match r {
                    Value::Bool(b) => Ok(Value::Bool(b)),
                    Value::Int(_) => {
                        let name = if matches!(op, BinOp::And) { "and" } else { "or" };
                        Err(EvalError::TypeError(format!(
                            "'{}' requires bool, got int on right",
                            name
                        )))
                    }
                };
            }

            let l = eval_in(lhs, env)?;
            let r = eval_in(rhs, env)?;
            match op {
                BinOp::Add => match (&l, &r) {
                    (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a.wrapping_add(*b))),
                    _ => Err(type_err_binary("+", &l, &r)),
                },
                BinOp::Sub => match (&l, &r) {
                    (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a.wrapping_sub(*b))),
                    _ => Err(type_err_binary("-", &l, &r)),
                },
                BinOp::Mul => match (&l, &r) {
                    (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a.wrapping_mul(*b))),
                    _ => Err(type_err_binary("*", &l, &r)),
                },
                BinOp::Div => match (&l, &r) {
                    (Value::Int(_), Value::Int(0)) => Err(EvalError::DivisionByZero),
                    (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a.wrapping_div(*b))),
                    _ => Err(type_err_binary("/", &l, &r)),
                },
                BinOp::Eq => match (&l, &r) {
                    (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a == b)),
                    (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a == b)),
                    _ => Err(type_err_binary("==", &l, &r)),
                },
                BinOp::Neq => match (&l, &r) {
                    (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a != b)),
                    (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a != b)),
                    _ => Err(type_err_binary("!=", &l, &r)),
                },
                BinOp::Lt => match (&l, &r) {
                    (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a < b)),
                    _ => Err(type_err_binary("<", &l, &r)),
                },
                BinOp::Le => match (&l, &r) {
                    (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a <= b)),
                    _ => Err(type_err_binary("<=", &l, &r)),
                },
                BinOp::Gt => match (&l, &r) {
                    (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a > b)),
                    _ => Err(type_err_binary(">", &l, &r)),
                },
                BinOp::Ge => match (&l, &r) {
                    (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a >= b)),
                    _ => Err(type_err_binary(">=", &l, &r)),
                },
                BinOp::And | BinOp::Or => unreachable!("handled above"),
            }
        }
    }
}

/// Evaluate `expr` to a [`Value`].
pub fn eval(expr: &Expr) -> Result<Value, EvalError> {
    eval_in(expr, &Env::Empty)
}
