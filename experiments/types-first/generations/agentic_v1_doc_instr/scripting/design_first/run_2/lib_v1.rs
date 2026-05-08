//! A tiny expression-language evaluator.

use std::fmt;

/// A parsed expression AST node.
#[derive(Debug, Clone, PartialEq)]
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
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Binary {
        op: BinOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
}

/// Unary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
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

/// An error encountered while parsing.
#[derive(Clone, PartialEq, Eq)]
pub struct ParseError {
    /// Byte offset within the input where parsing failed.
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
        write!(
            f,
            "ParseError {{ offset: {}, message: {:?} }}",
            self.offset, self.message
        )
    }
}

/// An error encountered while evaluating.
#[derive(Clone, PartialEq, Eq)]
pub enum EvalError {
    /// A type mismatch (e.g. `1 + true`, or non-bool condition in `if`).
    TypeError(String),
    /// Division (or modulo) by zero.
    DivisionByZero,
    /// Use of an identifier not in scope.
    UnboundIdent(String),
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvalError::TypeError(msg) => write!(f, "type error: {}", msg),
            EvalError::DivisionByZero => write!(f, "division by zero"),
            EvalError::UnboundIdent(name) => write!(f, "unbound identifier: {}", name),
        }
    }
}

impl fmt::Debug for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvalError::TypeError(msg) => write!(f, "TypeError({:?})", msg),
            EvalError::DivisionByZero => write!(f, "DivisionByZero"),
            EvalError::UnboundIdent(name) => write!(f, "UnboundIdent({:?})", name),
        }
    }
}

// ===== Tokenizer =====

#[derive(Debug, Clone, PartialEq, Eq)]
enum Tok {
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
    Eq,
    EqEq,
    NotEq,
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
    tok: Tok,
    offset: usize,
}

fn is_ident_start(c: u8) -> bool {
    matches!(c, b'a'..=b'z' | b'_')
}

fn is_ident_cont(c: u8) -> bool {
    matches!(c, b'a'..=b'z' | b'0'..=b'9' | b'_')
}

fn tokenize(input: &str) -> Result<Vec<Token>, ParseError> {
    let bytes = input.as_bytes();
    let mut pos = 0;
    let mut out = Vec::new();
    loop {
        while pos < bytes.len() && bytes[pos].is_ascii_whitespace() {
            pos += 1;
        }
        if pos >= bytes.len() {
            out.push(Token { tok: Tok::Eof, offset: pos });
            return Ok(out);
        }
        let start = pos;
        let c = bytes[pos];
        let tok = match c {
            b'0'..=b'9' => {
                let mut end = pos;
                while end < bytes.len() && bytes[end].is_ascii_digit() {
                    end += 1;
                }
                let s = &input[pos..end];
                let val = s.parse::<i64>().map_err(|_| ParseError {
                    offset: pos,
                    message: format!("invalid integer literal {:?}", s),
                })?;
                pos = end;
                Tok::Int(val)
            }
            c if is_ident_start(c) => {
                let mut end = pos;
                while end < bytes.len() && is_ident_cont(bytes[end]) {
                    end += 1;
                }
                let s = &input[pos..end];
                pos = end;
                match s {
                    "let" => Tok::Let,
                    "in" => Tok::In,
                    "if" => Tok::If,
                    "then" => Tok::Then,
                    "else" => Tok::Else,
                    "or" => Tok::Or,
                    "and" => Tok::And,
                    "not" => Tok::Not,
                    "true" => Tok::True,
                    "false" => Tok::False,
                    _ => Tok::Ident(s.to_string()),
                }
            }
            b'=' => {
                if pos + 1 < bytes.len() && bytes[pos + 1] == b'=' {
                    pos += 2;
                    Tok::EqEq
                } else {
                    pos += 1;
                    Tok::Eq
                }
            }
            b'!' => {
                if pos + 1 < bytes.len() && bytes[pos + 1] == b'=' {
                    pos += 2;
                    Tok::NotEq
                } else {
                    return Err(ParseError {
                        offset: pos,
                        message: "expected '!=' but found stray '!'".into(),
                    });
                }
            }
            b'<' => {
                if pos + 1 < bytes.len() && bytes[pos + 1] == b'=' {
                    pos += 2;
                    Tok::Le
                } else {
                    pos += 1;
                    Tok::Lt
                }
            }
            b'>' => {
                if pos + 1 < bytes.len() && bytes[pos + 1] == b'=' {
                    pos += 2;
                    Tok::Ge
                } else {
                    pos += 1;
                    Tok::Gt
                }
            }
            b'+' => {
                pos += 1;
                Tok::Plus
            }
            b'-' => {
                pos += 1;
                Tok::Minus
            }
            b'*' => {
                pos += 1;
                Tok::Star
            }
            b'/' => {
                pos += 1;
                Tok::Slash
            }
            b'(' => {
                pos += 1;
                Tok::LParen
            }
            b')' => {
                pos += 1;
                Tok::RParen
            }
            _ => {
                return Err(ParseError {
                    offset: pos,
                    message: format!("unexpected character {:?}", c as char),
                });
            }
        };
        out.push(Token { tok, offset: start });
    }
}

// ===== Parser =====

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

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        match self.peek().tok {
            Tok::Let => self.parse_let(),
            Tok::If => self.parse_if(),
            _ => self.parse_or(),
        }
    }

    fn parse_let(&mut self) -> Result<Expr, ParseError> {
        self.advance(); // let
        let name_tok = self.advance();
        let name = match name_tok.tok {
            Tok::Ident(n) => n,
            _ => {
                return Err(ParseError {
                    offset: name_tok.offset,
                    message: "expected identifier after 'let'".into(),
                });
            }
        };
        let eq_tok = self.advance();
        if !matches!(eq_tok.tok, Tok::Eq) {
            return Err(ParseError {
                offset: eq_tok.offset,
                message: "expected '=' after let identifier".into(),
            });
        }
        let value = self.parse_expr()?;
        let in_tok = self.advance();
        if !matches!(in_tok.tok, Tok::In) {
            return Err(ParseError {
                offset: in_tok.offset,
                message: "expected 'in' in let-expression".into(),
            });
        }
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
        let t = self.advance();
        if !matches!(t.tok, Tok::Then) {
            return Err(ParseError {
                offset: t.offset,
                message: "expected 'then' in if-expression".into(),
            });
        }
        let then_branch = self.parse_expr()?;
        let e = self.advance();
        if !matches!(e.tok, Tok::Else) {
            return Err(ParseError {
                offset: e.offset,
                message: "expected 'else' in if-expression".into(),
            });
        }
        let else_branch = self.parse_expr()?;
        Ok(Expr::If {
            cond: Box::new(cond),
            then_branch: Box::new(then_branch),
            else_branch: Box::new(else_branch),
        })
    }

    fn parse_or(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_and()?;
        while matches!(self.peek().tok, Tok::Or) {
            self.advance();
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
        while matches!(self.peek().tok, Tok::And) {
            self.advance();
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
        if matches!(self.peek().tok, Tok::Not) {
            self.advance();
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
        let op = match self.peek().tok {
            Tok::EqEq => BinOp::Eq,
            Tok::NotEq => BinOp::Ne,
            Tok::Lt => BinOp::Lt,
            Tok::Le => BinOp::Le,
            Tok::Gt => BinOp::Gt,
            Tok::Ge => BinOp::Ge,
            _ => return Ok(lhs),
        };
        self.advance();
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
            let op = match self.peek().tok {
                Tok::Plus => BinOp::Add,
                Tok::Minus => BinOp::Sub,
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
            let op = match self.peek().tok {
                Tok::Star => BinOp::Mul,
                Tok::Slash => BinOp::Div,
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
        if matches!(self.peek().tok, Tok::Minus) {
            self.advance();
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
        let t = self.advance();
        match t.tok {
            Tok::Int(n) => Ok(Expr::Int(n)),
            Tok::True => Ok(Expr::Bool(true)),
            Tok::False => Ok(Expr::Bool(false)),
            Tok::Ident(s) => Ok(Expr::Ident(s)),
            Tok::LParen => {
                let inner = self.parse_expr()?;
                let close = self.advance();
                if !matches!(close.tok, Tok::RParen) {
                    return Err(ParseError {
                        offset: close.offset,
                        message: "expected ')'".into(),
                    });
                }
                Ok(inner)
            }
            _ => Err(ParseError {
                offset: t.offset,
                message: "expected an expression atom".into(),
            }),
        }
    }
}

/// Parse the input source into an `Expr`.
pub fn parse(input: &str) -> Result<Expr, ParseError> {
    let tokens = tokenize(input)?;
    let mut parser = Parser { tokens, pos: 0 };
    let expr = parser.parse_expr()?;
    if !matches!(parser.peek().tok, Tok::Eof) {
        return Err(ParseError {
            offset: parser.peek().offset,
            message: "unexpected trailing input".into(),
        });
    }
    Ok(expr)
}

// ===== Evaluator =====

fn eval_with_env(expr: &Expr, env: &mut Vec<(String, Value)>) -> Result<Value, EvalError> {
    match expr {
        Expr::Int(n) => Ok(Value::Int(*n)),
        Expr::Bool(b) => Ok(Value::Bool(*b)),
        Expr::Ident(name) => {
            for (n, v) in env.iter().rev() {
                if n == name {
                    return Ok(v.clone());
                }
            }
            Err(EvalError::UnboundIdent(name.clone()))
        }
        Expr::Let { name, value, body } => {
            let v = eval_with_env(value, env)?;
            env.push((name.clone(), v));
            let result = eval_with_env(body, env);
            env.pop();
            result
        }
        Expr::If {
            cond,
            then_branch,
            else_branch,
        } => match eval_with_env(cond, env)? {
            Value::Bool(true) => eval_with_env(then_branch, env),
            Value::Bool(false) => eval_with_env(else_branch, env),
            Value::Int(_) => Err(EvalError::TypeError(
                "if condition must be a bool".into(),
            )),
        },
        Expr::Unary { op, expr } => {
            let v = eval_with_env(expr, env)?;
            match (op, v) {
                (UnaryOp::Neg, Value::Int(n)) => Ok(Value::Int(n.wrapping_neg())),
                (UnaryOp::Neg, Value::Bool(_)) => Err(EvalError::TypeError(
                    "unary '-' requires an integer".into(),
                )),
                (UnaryOp::Not, Value::Bool(b)) => Ok(Value::Bool(!b)),
                (UnaryOp::Not, Value::Int(_)) => Err(EvalError::TypeError(
                    "'not' requires a bool".into(),
                )),
            }
        }
        Expr::Binary { op, lhs, rhs } => eval_binary(*op, lhs, rhs, env),
    }
}

fn eval_binary(
    op: BinOp,
    lhs: &Expr,
    rhs: &Expr,
    env: &mut Vec<(String, Value)>,
) -> Result<Value, EvalError> {
    // Short-circuit logical operators.
    match op {
        BinOp::Or => {
            let l = eval_with_env(lhs, env)?;
            let lb = match l {
                Value::Bool(b) => b,
                Value::Int(_) => {
                    return Err(EvalError::TypeError("'or' requires a bool".into()));
                }
            };
            if lb {
                return Ok(Value::Bool(true));
            }
            let r = eval_with_env(rhs, env)?;
            match r {
                Value::Bool(b) => Ok(Value::Bool(b)),
                Value::Int(_) => Err(EvalError::TypeError("'or' requires a bool".into())),
            }
        }
        BinOp::And => {
            let l = eval_with_env(lhs, env)?;
            let lb = match l {
                Value::Bool(b) => b,
                Value::Int(_) => {
                    return Err(EvalError::TypeError("'and' requires a bool".into()));
                }
            };
            if !lb {
                return Ok(Value::Bool(false));
            }
            let r = eval_with_env(rhs, env)?;
            match r {
                Value::Bool(b) => Ok(Value::Bool(b)),
                Value::Int(_) => Err(EvalError::TypeError("'and' requires a bool".into())),
            }
        }
        _ => {}
    }

    let l = eval_with_env(lhs, env)?;
    let r = eval_with_env(rhs, env)?;

    match op {
        BinOp::Eq => match (l, r) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a == b)),
            (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a == b)),
            _ => Err(EvalError::TypeError(
                "'==' requires operands of the same type".into(),
            )),
        },
        BinOp::Ne => match (l, r) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a != b)),
            (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a != b)),
            _ => Err(EvalError::TypeError(
                "'!=' requires operands of the same type".into(),
            )),
        },
        BinOp::Lt => int_pair(l, r, "<").map(|(a, b)| Value::Bool(a < b)),
        BinOp::Le => int_pair(l, r, "<=").map(|(a, b)| Value::Bool(a <= b)),
        BinOp::Gt => int_pair(l, r, ">").map(|(a, b)| Value::Bool(a > b)),
        BinOp::Ge => int_pair(l, r, ">=").map(|(a, b)| Value::Bool(a >= b)),
        BinOp::Add => int_pair(l, r, "+").map(|(a, b)| Value::Int(a.wrapping_add(b))),
        BinOp::Sub => int_pair(l, r, "-").map(|(a, b)| Value::Int(a.wrapping_sub(b))),
        BinOp::Mul => int_pair(l, r, "*").map(|(a, b)| Value::Int(a.wrapping_mul(b))),
        BinOp::Div => {
            let (a, b) = int_pair(l, r, "/")?;
            if b == 0 {
                Err(EvalError::DivisionByZero)
            } else {
                Ok(Value::Int(a.wrapping_div(b)))
            }
        }
        BinOp::Or | BinOp::And => unreachable!(),
    }
}

fn int_pair(l: Value, r: Value, op: &str) -> Result<(i64, i64), EvalError> {
    match (l, r) {
        (Value::Int(a), Value::Int(b)) => Ok((a, b)),
        _ => Err(EvalError::TypeError(format!(
            "'{}' requires integer operands",
            op
        ))),
    }
}

/// Evaluate an expression to a `Value`.
pub fn eval(expr: &Expr) -> Result<Value, EvalError> {
    let mut env = Vec::new();
    eval_with_env(expr, &mut env)
}
