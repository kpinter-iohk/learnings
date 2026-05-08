use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Int(i64),
    Bool(bool),
    Ident(String),
    Neg(Box<Expr>),
    Not(Box<Expr>),
    BinOp(BinOp, Box<Expr>, Box<Expr>),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    Let(String, Box<Expr>, Box<Expr>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Int(i64),
    Bool(bool),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(i) => write!(f, "{}", i),
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
    Eq,
    EqEq,
    Ne,
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

struct Lexer<'a> {
    src: &'a [u8],
    pos: usize,
}

impl<'a> Lexer<'a> {
    fn new(src: &'a str) -> Self {
        Self {
            src: src.as_bytes(),
            pos: 0,
        }
    }

    fn skip_ws(&mut self) {
        while self.pos < self.src.len() && self.src[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
    }

    fn peek_byte(&self) -> Option<u8> {
        self.src.get(self.pos).copied()
    }

    fn next_token(&mut self) -> Result<(Tok, usize), ParseError> {
        self.skip_ws();
        let start = self.pos;
        let Some(b) = self.peek_byte() else {
            return Ok((Tok::Eof, start));
        };
        match b {
            b'(' => {
                self.pos += 1;
                Ok((Tok::LParen, start))
            }
            b')' => {
                self.pos += 1;
                Ok((Tok::RParen, start))
            }
            b'+' => {
                self.pos += 1;
                Ok((Tok::Plus, start))
            }
            b'-' => {
                self.pos += 1;
                Ok((Tok::Minus, start))
            }
            b'*' => {
                self.pos += 1;
                Ok((Tok::Star, start))
            }
            b'/' => {
                self.pos += 1;
                Ok((Tok::Slash, start))
            }
            b'=' => {
                self.pos += 1;
                if self.peek_byte() == Some(b'=') {
                    self.pos += 1;
                    Ok((Tok::EqEq, start))
                } else {
                    Ok((Tok::Eq, start))
                }
            }
            b'!' => {
                self.pos += 1;
                if self.peek_byte() == Some(b'=') {
                    self.pos += 1;
                    Ok((Tok::Ne, start))
                } else {
                    Err(ParseError {
                        offset: start,
                        message: "expected '!='".to_string(),
                    })
                }
            }
            b'<' => {
                self.pos += 1;
                if self.peek_byte() == Some(b'=') {
                    self.pos += 1;
                    Ok((Tok::Le, start))
                } else {
                    Ok((Tok::Lt, start))
                }
            }
            b'>' => {
                self.pos += 1;
                if self.peek_byte() == Some(b'=') {
                    self.pos += 1;
                    Ok((Tok::Ge, start))
                } else {
                    Ok((Tok::Gt, start))
                }
            }
            b'0'..=b'9' => {
                let s = self.pos;
                while let Some(c) = self.peek_byte() {
                    if c.is_ascii_digit() {
                        self.pos += 1;
                    } else {
                        break;
                    }
                }
                let text = std::str::from_utf8(&self.src[s..self.pos]).unwrap();
                let n: i64 = text.parse().map_err(|_| ParseError {
                    offset: s,
                    message: format!("invalid integer literal: {}", text),
                })?;
                Ok((Tok::Int(n), start))
            }
            b'a'..=b'z' | b'_' => {
                let s = self.pos;
                while let Some(c) = self.peek_byte() {
                    if c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_' {
                        self.pos += 1;
                    } else {
                        break;
                    }
                }
                let text = std::str::from_utf8(&self.src[s..self.pos]).unwrap();
                let tok = match text {
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
                    _ => Tok::Ident(text.to_string()),
                };
                Ok((tok, start))
            }
            other => Err(ParseError {
                offset: start,
                message: format!("unexpected character: {:?}", other as char),
            }),
        }
    }
}

struct Parser<'a> {
    lexer: Lexer<'a>,
    current: (Tok, usize),
}

impl<'a> Parser<'a> {
    fn new(src: &'a str) -> Result<Self, ParseError> {
        let mut lex = Lexer::new(src);
        let cur = lex.next_token()?;
        Ok(Self {
            lexer: lex,
            current: cur,
        })
    }

    fn bump(&mut self) -> Result<(Tok, usize), ParseError> {
        let next = self.lexer.next_token()?;
        Ok(std::mem::replace(&mut self.current, next))
    }

    fn peek(&self) -> &Tok {
        &self.current.0
    }

    fn pos(&self) -> usize {
        self.current.1
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        match self.peek() {
            Tok::KwLet => self.parse_let(),
            Tok::KwIf => self.parse_if(),
            _ => self.parse_or(),
        }
    }

    fn parse_let(&mut self) -> Result<Expr, ParseError> {
        self.bump()?;
        let name = match self.bump()? {
            (Tok::Ident(s), _) => s,
            (_, off) => {
                return Err(ParseError {
                    offset: off,
                    message: "expected identifier after 'let'".to_string(),
                });
            }
        };
        match self.peek() {
            Tok::Eq => {
                self.bump()?;
            }
            _ => {
                return Err(ParseError {
                    offset: self.pos(),
                    message: "expected '=' in let binding".to_string(),
                });
            }
        }
        let value = self.parse_expr()?;
        match self.peek() {
            Tok::KwIn => {
                self.bump()?;
            }
            _ => {
                return Err(ParseError {
                    offset: self.pos(),
                    message: "expected 'in' in let binding".to_string(),
                });
            }
        }
        let body = self.parse_expr()?;
        Ok(Expr::Let(name, Box::new(value), Box::new(body)))
    }

    fn parse_if(&mut self) -> Result<Expr, ParseError> {
        self.bump()?;
        let cond = self.parse_expr()?;
        match self.peek() {
            Tok::KwThen => {
                self.bump()?;
            }
            _ => {
                return Err(ParseError {
                    offset: self.pos(),
                    message: "expected 'then'".to_string(),
                });
            }
        }
        let t = self.parse_expr()?;
        match self.peek() {
            Tok::KwElse => {
                self.bump()?;
            }
            _ => {
                return Err(ParseError {
                    offset: self.pos(),
                    message: "expected 'else'".to_string(),
                });
            }
        }
        let e = self.parse_expr()?;
        Ok(Expr::If(Box::new(cond), Box::new(t), Box::new(e)))
    }

    fn parse_or(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_and()?;
        while matches!(self.peek(), Tok::KwOr) {
            self.bump()?;
            let right = self.parse_and()?;
            left = Expr::BinOp(BinOp::Or, Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_not()?;
        while matches!(self.peek(), Tok::KwAnd) {
            self.bump()?;
            let right = self.parse_not()?;
            left = Expr::BinOp(BinOp::And, Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_not(&mut self) -> Result<Expr, ParseError> {
        if matches!(self.peek(), Tok::KwNot) {
            self.bump()?;
            let inner = self.parse_not()?;
            Ok(Expr::Not(Box::new(inner)))
        } else {
            self.parse_cmp()
        }
    }

    fn parse_cmp(&mut self) -> Result<Expr, ParseError> {
        let left = self.parse_add()?;
        let op = match self.peek() {
            Tok::EqEq => Some(BinOp::Eq),
            Tok::Ne => Some(BinOp::Ne),
            Tok::Lt => Some(BinOp::Lt),
            Tok::Le => Some(BinOp::Le),
            Tok::Gt => Some(BinOp::Gt),
            Tok::Ge => Some(BinOp::Ge),
            _ => None,
        };
        if let Some(op) = op {
            self.bump()?;
            let right = self.parse_add()?;
            Ok(Expr::BinOp(op, Box::new(left), Box::new(right)))
        } else {
            Ok(left)
        }
    }

    fn parse_add(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_mul()?;
        loop {
            let op = match self.peek() {
                Tok::Plus => BinOp::Add,
                Tok::Minus => BinOp::Sub,
                _ => break,
            };
            self.bump()?;
            let right = self.parse_mul()?;
            left = Expr::BinOp(op, Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_mul(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_unary()?;
        loop {
            let op = match self.peek() {
                Tok::Star => BinOp::Mul,
                Tok::Slash => BinOp::Div,
                _ => break,
            };
            self.bump()?;
            let right = self.parse_unary()?;
            left = Expr::BinOp(op, Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        if matches!(self.peek(), Tok::Minus) {
            self.bump()?;
            let inner = self.parse_unary()?;
            Ok(Expr::Neg(Box::new(inner)))
        } else {
            self.parse_atom()
        }
    }

    fn parse_atom(&mut self) -> Result<Expr, ParseError> {
        match self.peek() {
            Tok::Int(_) => {
                let (tok, _) = self.bump()?;
                if let Tok::Int(n) = tok {
                    Ok(Expr::Int(n))
                } else {
                    unreachable!()
                }
            }
            Tok::KwTrue => {
                self.bump()?;
                Ok(Expr::Bool(true))
            }
            Tok::KwFalse => {
                self.bump()?;
                Ok(Expr::Bool(false))
            }
            Tok::Ident(_) => {
                let (tok, _) = self.bump()?;
                if let Tok::Ident(s) = tok {
                    Ok(Expr::Ident(s))
                } else {
                    unreachable!()
                }
            }
            Tok::LParen => {
                self.bump()?;
                let e = self.parse_expr()?;
                match self.peek() {
                    Tok::RParen => {
                        self.bump()?;
                        Ok(e)
                    }
                    _ => Err(ParseError {
                        offset: self.pos(),
                        message: "expected ')'".to_string(),
                    }),
                }
            }
            _ => Err(ParseError {
                offset: self.pos(),
                message: "expected expression".to_string(),
            }),
        }
    }
}

pub fn parse(input: &str) -> Result<Expr, ParseError> {
    let mut p = Parser::new(input)?;
    let e = p.parse_expr()?;
    if !matches!(p.peek(), Tok::Eof) {
        return Err(ParseError {
            offset: p.pos(),
            message: "unexpected trailing input".to_string(),
        });
    }
    Ok(e)
}

pub fn eval(expr: &Expr) -> Result<Value, EvalError> {
    let mut env: Vec<(String, Value)> = Vec::new();
    eval_with(expr, &mut env)
}

fn eval_with(expr: &Expr, env: &mut Vec<(String, Value)>) -> Result<Value, EvalError> {
    match expr {
        Expr::Int(i) => Ok(Value::Int(*i)),
        Expr::Bool(b) => Ok(Value::Bool(*b)),
        Expr::Ident(name) => {
            for (n, v) in env.iter().rev() {
                if n == name {
                    return Ok(v.clone());
                }
            }
            Err(EvalError::UnboundIdentifier(name.clone()))
        }
        Expr::Neg(inner) => match eval_with(inner, env)? {
            Value::Int(i) => Ok(Value::Int(i.wrapping_neg())),
            Value::Bool(_) => Err(EvalError::TypeError(
                "unary '-' requires integer".to_string(),
            )),
        },
        Expr::Not(inner) => match eval_with(inner, env)? {
            Value::Bool(b) => Ok(Value::Bool(!b)),
            Value::Int(_) => Err(EvalError::TypeError("'not' requires boolean".to_string())),
        },
        Expr::BinOp(op, l, r) => eval_binop(*op, l, r, env),
        Expr::If(cond, t, e) => match eval_with(cond, env)? {
            Value::Bool(b) => eval_with(if b { t } else { e }, env),
            Value::Int(_) => Err(EvalError::TypeError(
                "'if' condition must be boolean".to_string(),
            )),
        },
        Expr::Let(name, val, body) => {
            let v = eval_with(val, env)?;
            env.push((name.clone(), v));
            let res = eval_with(body, env);
            env.pop();
            res
        }
    }
}

fn eval_binop(
    op: BinOp,
    l: &Expr,
    r: &Expr,
    env: &mut Vec<(String, Value)>,
) -> Result<Value, EvalError> {
    match op {
        BinOp::And => {
            let lv = eval_with(l, env)?;
            let lb = match lv {
                Value::Bool(b) => b,
                Value::Int(_) => {
                    return Err(EvalError::TypeError(
                        "'and' requires booleans".to_string(),
                    ));
                }
            };
            if !lb {
                return Ok(Value::Bool(false));
            }
            match eval_with(r, env)? {
                Value::Bool(b) => Ok(Value::Bool(b)),
                Value::Int(_) => Err(EvalError::TypeError(
                    "'and' requires booleans".to_string(),
                )),
            }
        }
        BinOp::Or => {
            let lv = eval_with(l, env)?;
            let lb = match lv {
                Value::Bool(b) => b,
                Value::Int(_) => {
                    return Err(EvalError::TypeError(
                        "'or' requires booleans".to_string(),
                    ));
                }
            };
            if lb {
                return Ok(Value::Bool(true));
            }
            match eval_with(r, env)? {
                Value::Bool(b) => Ok(Value::Bool(b)),
                Value::Int(_) => Err(EvalError::TypeError(
                    "'or' requires booleans".to_string(),
                )),
            }
        }
        _ => {
            let lv = eval_with(l, env)?;
            let rv = eval_with(r, env)?;
            match op {
                BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div => match (lv, rv) {
                    (Value::Int(a), Value::Int(b)) => {
                        let result = match op {
                            BinOp::Add => a.wrapping_add(b),
                            BinOp::Sub => a.wrapping_sub(b),
                            BinOp::Mul => a.wrapping_mul(b),
                            BinOp::Div => {
                                if b == 0 {
                                    return Err(EvalError::DivisionByZero);
                                }
                                a.wrapping_div(b)
                            }
                            _ => unreachable!(),
                        };
                        Ok(Value::Int(result))
                    }
                    _ => Err(EvalError::TypeError(
                        "arithmetic operator requires integer operands".to_string(),
                    )),
                },
                BinOp::Eq | BinOp::Ne => match (lv, rv) {
                    (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(if op == BinOp::Eq {
                        a == b
                    } else {
                        a != b
                    })),
                    (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(if op == BinOp::Eq {
                        a == b
                    } else {
                        a != b
                    })),
                    _ => Err(EvalError::TypeError(
                        "equality requires operands of the same type".to_string(),
                    )),
                },
                BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => match (lv, rv) {
                    (Value::Int(a), Value::Int(b)) => {
                        let r = match op {
                            BinOp::Lt => a < b,
                            BinOp::Le => a <= b,
                            BinOp::Gt => a > b,
                            BinOp::Ge => a >= b,
                            _ => unreachable!(),
                        };
                        Ok(Value::Bool(r))
                    }
                    _ => Err(EvalError::TypeError(
                        "ordering comparison requires integer operands".to_string(),
                    )),
                },
                _ => unreachable!(),
            }
        }
    }
}
