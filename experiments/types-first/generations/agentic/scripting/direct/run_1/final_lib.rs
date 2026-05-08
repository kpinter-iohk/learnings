use std::fmt;

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone)]
pub struct ParseError {
    pub offset: usize,
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "parse error at offset {}: {}", self.offset, self.message)
    }
}

impl std::error::Error for ParseError {}

#[derive(Debug, Clone)]
pub enum EvalError {
    TypeError(String),
    DivisionByZero,
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

impl std::error::Error for EvalError {}

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Int(i64),
    Ident(String),
    True,
    False,
    Let,
    In,
    If,
    Then,
    Else,
    Or,
    And,
    Not,
    Plus,
    Minus,
    Star,
    Slash,
    EqEq,
    NotEq,
    Lt,
    Le,
    Gt,
    Ge,
    Assign,
    LParen,
    RParen,
}

struct Lexer<'a> {
    src: &'a str,
    pos: usize,
}

impl<'a> Lexer<'a> {
    fn new(src: &'a str) -> Self {
        Self { src, pos: 0 }
    }

    fn skip_ws(&mut self) {
        let bytes = self.src.as_bytes();
        while self.pos < bytes.len() {
            let b = bytes[self.pos];
            if b == b' ' || b == b'\t' || b == b'\n' || b == b'\r' || b == 0x0b || b == 0x0c {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn next_token(&mut self) -> Result<Option<(usize, Tok)>, ParseError> {
        self.skip_ws();
        let bytes = self.src.as_bytes();
        if self.pos >= bytes.len() {
            return Ok(None);
        }
        let start = self.pos;
        let b = bytes[start];
        let next_b = bytes.get(start + 1).copied();

        match (b, next_b) {
            (b'=', Some(b'=')) => {
                self.pos += 2;
                return Ok(Some((start, Tok::EqEq)));
            }
            (b'!', Some(b'=')) => {
                self.pos += 2;
                return Ok(Some((start, Tok::NotEq)));
            }
            (b'<', Some(b'=')) => {
                self.pos += 2;
                return Ok(Some((start, Tok::Le)));
            }
            (b'>', Some(b'=')) => {
                self.pos += 2;
                return Ok(Some((start, Tok::Ge)));
            }
            _ => {}
        }

        match b {
            b'+' => {
                self.pos += 1;
                Ok(Some((start, Tok::Plus)))
            }
            b'-' => {
                self.pos += 1;
                Ok(Some((start, Tok::Minus)))
            }
            b'*' => {
                self.pos += 1;
                Ok(Some((start, Tok::Star)))
            }
            b'/' => {
                self.pos += 1;
                Ok(Some((start, Tok::Slash)))
            }
            b'=' => {
                self.pos += 1;
                Ok(Some((start, Tok::Assign)))
            }
            b'<' => {
                self.pos += 1;
                Ok(Some((start, Tok::Lt)))
            }
            b'>' => {
                self.pos += 1;
                Ok(Some((start, Tok::Gt)))
            }
            b'!' => Err(ParseError {
                offset: start,
                message: "unexpected '!' (expected '!=')".to_string(),
            }),
            b'(' => {
                self.pos += 1;
                Ok(Some((start, Tok::LParen)))
            }
            b')' => {
                self.pos += 1;
                Ok(Some((start, Tok::RParen)))
            }
            b'0'..=b'9' => {
                let mut end = start;
                while end < bytes.len() && bytes[end].is_ascii_digit() {
                    end += 1;
                }
                let s = &self.src[start..end];
                self.pos = end;
                let n: i64 = s.parse().map_err(|_| ParseError {
                    offset: start,
                    message: format!("invalid integer literal {:?}", s),
                })?;
                Ok(Some((start, Tok::Int(n))))
            }
            b'a'..=b'z' | b'_' => {
                let mut end = start;
                while end < bytes.len() {
                    let c = bytes[end];
                    if c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_' {
                        end += 1;
                    } else {
                        break;
                    }
                }
                let s = &self.src[start..end];
                self.pos = end;
                let tok = match s {
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
                };
                Ok(Some((start, tok)))
            }
            _ => {
                let ch = self.src[start..].chars().next().unwrap_or('?');
                Err(ParseError {
                    offset: start,
                    message: format!("unexpected character {:?}", ch),
                })
            }
        }
    }
}

fn tokenize(src: &str) -> Result<Vec<(usize, Tok)>, ParseError> {
    let mut lexer = Lexer::new(src);
    let mut out = Vec::new();
    while let Some(t) = lexer.next_token()? {
        out.push(t);
    }
    Ok(out)
}

struct Parser {
    tokens: Vec<(usize, Tok)>,
    pos: usize,
    end_offset: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Tok> {
        self.tokens.get(self.pos).map(|(_, t)| t)
    }

    fn peek_offset(&self) -> usize {
        self.tokens
            .get(self.pos)
            .map(|(o, _)| *o)
            .unwrap_or(self.end_offset)
    }

    fn bump(&mut self) {
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
    }

    fn expect(&mut self, want: &Tok, what: &str) -> Result<(), ParseError> {
        match self.peek() {
            Some(t) if t == want => {
                self.pos += 1;
                Ok(())
            }
            _ => Err(ParseError {
                offset: self.peek_offset(),
                message: format!("expected {}", what),
            }),
        }
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        match self.peek() {
            Some(Tok::Let) => self.parse_let(),
            Some(Tok::If) => self.parse_if(),
            _ => self.parse_or(),
        }
    }

    fn parse_let(&mut self) -> Result<Expr, ParseError> {
        self.bump();
        let name = match self.peek().cloned() {
            Some(Tok::Ident(s)) => {
                self.bump();
                s
            }
            _ => {
                return Err(ParseError {
                    offset: self.peek_offset(),
                    message: "expected identifier after 'let'".to_string(),
                })
            }
        };
        self.expect(&Tok::Assign, "'='")?;
        let value = self.parse_expr()?;
        self.expect(&Tok::In, "'in'")?;
        let body = self.parse_expr()?;
        Ok(Expr::Let(name, Box::new(value), Box::new(body)))
    }

    fn parse_if(&mut self) -> Result<Expr, ParseError> {
        self.bump();
        let c = self.parse_expr()?;
        self.expect(&Tok::Then, "'then'")?;
        let t = self.parse_expr()?;
        self.expect(&Tok::Else, "'else'")?;
        let e = self.parse_expr()?;
        Ok(Expr::If(Box::new(c), Box::new(t), Box::new(e)))
    }

    fn parse_or(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_and()?;
        while matches!(self.peek(), Some(Tok::Or)) {
            self.bump();
            let right = self.parse_and()?;
            left = Expr::BinOp(BinOp::Or, Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_not()?;
        while matches!(self.peek(), Some(Tok::And)) {
            self.bump();
            let right = self.parse_not()?;
            left = Expr::BinOp(BinOp::And, Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_not(&mut self) -> Result<Expr, ParseError> {
        if matches!(self.peek(), Some(Tok::Not)) {
            self.bump();
            let inner = self.parse_not()?;
            return Ok(Expr::Not(Box::new(inner)));
        }
        self.parse_cmp()
    }

    fn parse_cmp(&mut self) -> Result<Expr, ParseError> {
        let left = self.parse_add()?;
        let op = match self.peek() {
            Some(Tok::EqEq) => BinOp::Eq,
            Some(Tok::NotEq) => BinOp::Ne,
            Some(Tok::Lt) => BinOp::Lt,
            Some(Tok::Le) => BinOp::Le,
            Some(Tok::Gt) => BinOp::Gt,
            Some(Tok::Ge) => BinOp::Ge,
            _ => return Ok(left),
        };
        self.bump();
        let right = self.parse_add()?;
        Ok(Expr::BinOp(op, Box::new(left), Box::new(right)))
    }

    fn parse_add(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_mul()?;
        loop {
            let op = match self.peek() {
                Some(Tok::Plus) => BinOp::Add,
                Some(Tok::Minus) => BinOp::Sub,
                _ => break,
            };
            self.bump();
            let right = self.parse_mul()?;
            left = Expr::BinOp(op, Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_mul(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_unary()?;
        loop {
            let op = match self.peek() {
                Some(Tok::Star) => BinOp::Mul,
                Some(Tok::Slash) => BinOp::Div,
                _ => break,
            };
            self.bump();
            let right = self.parse_unary()?;
            left = Expr::BinOp(op, Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        if matches!(self.peek(), Some(Tok::Minus)) {
            self.bump();
            let inner = self.parse_unary()?;
            return Ok(Expr::Neg(Box::new(inner)));
        }
        self.parse_atom()
    }

    fn parse_atom(&mut self) -> Result<Expr, ParseError> {
        let off = self.peek_offset();
        match self.peek().cloned() {
            Some(Tok::Int(n)) => {
                self.bump();
                Ok(Expr::Int(n))
            }
            Some(Tok::True) => {
                self.bump();
                Ok(Expr::Bool(true))
            }
            Some(Tok::False) => {
                self.bump();
                Ok(Expr::Bool(false))
            }
            Some(Tok::Ident(s)) => {
                self.bump();
                Ok(Expr::Ident(s))
            }
            Some(Tok::LParen) => {
                self.bump();
                let inner = self.parse_expr()?;
                self.expect(&Tok::RParen, "')'")?;
                Ok(inner)
            }
            _ => Err(ParseError {
                offset: off,
                message: "expected expression".to_string(),
            }),
        }
    }
}

pub fn parse(input: &str) -> Result<Expr, ParseError> {
    let toks = tokenize(input)?;
    let end = input.len();
    let mut p = Parser {
        tokens: toks,
        pos: 0,
        end_offset: end,
    };
    let expr = p.parse_expr()?;
    if p.pos < p.tokens.len() {
        return Err(ParseError {
            offset: p.peek_offset(),
            message: "unexpected trailing tokens".to_string(),
        });
    }
    Ok(expr)
}

pub fn eval(expr: &Expr) -> Result<Value, EvalError> {
    let mut env: Vec<(String, Value)> = Vec::new();
    eval_inner(expr, &mut env)
}

fn eval_inner(expr: &Expr, env: &mut Vec<(String, Value)>) -> Result<Value, EvalError> {
    match expr {
        Expr::Int(n) => Ok(Value::Int(*n)),
        Expr::Bool(b) => Ok(Value::Bool(*b)),
        Expr::Ident(name) => {
            for (k, v) in env.iter().rev() {
                if k == name {
                    return Ok(v.clone());
                }
            }
            Err(EvalError::UnboundIdent(name.clone()))
        }
        Expr::Neg(e) => match eval_inner(e, env)? {
            Value::Int(n) => Ok(Value::Int(n.wrapping_neg())),
            Value::Bool(_) => Err(EvalError::TypeError(
                "unary '-' expects an integer".to_string(),
            )),
        },
        Expr::Not(e) => match eval_inner(e, env)? {
            Value::Bool(b) => Ok(Value::Bool(!b)),
            Value::Int(_) => Err(EvalError::TypeError("'not' expects a bool".to_string())),
        },
        Expr::BinOp(op, l, r) => match op {
            BinOp::And => {
                let lv = eval_inner(l, env)?;
                let lb = match lv {
                    Value::Bool(b) => b,
                    Value::Int(_) => {
                        return Err(EvalError::TypeError(
                            "'and' expects bool operands".to_string(),
                        ))
                    }
                };
                if !lb {
                    return Ok(Value::Bool(false));
                }
                match eval_inner(r, env)? {
                    Value::Bool(b) => Ok(Value::Bool(b)),
                    Value::Int(_) => Err(EvalError::TypeError(
                        "'and' expects bool operands".to_string(),
                    )),
                }
            }
            BinOp::Or => {
                let lv = eval_inner(l, env)?;
                let lb = match lv {
                    Value::Bool(b) => b,
                    Value::Int(_) => {
                        return Err(EvalError::TypeError(
                            "'or' expects bool operands".to_string(),
                        ))
                    }
                };
                if lb {
                    return Ok(Value::Bool(true));
                }
                match eval_inner(r, env)? {
                    Value::Bool(b) => Ok(Value::Bool(b)),
                    Value::Int(_) => Err(EvalError::TypeError(
                        "'or' expects bool operands".to_string(),
                    )),
                }
            }
            _ => {
                let lv = eval_inner(l, env)?;
                let rv = eval_inner(r, env)?;
                apply_binop(*op, lv, rv)
            }
        },
        Expr::If(c, t, e) => match eval_inner(c, env)? {
            Value::Bool(true) => eval_inner(t, env),
            Value::Bool(false) => eval_inner(e, env),
            Value::Int(_) => Err(EvalError::TypeError(
                "'if' condition must be a bool".to_string(),
            )),
        },
        Expr::Let(name, value, body) => {
            let v = eval_inner(value, env)?;
            env.push((name.clone(), v));
            let r = eval_inner(body, env);
            env.pop();
            r
        }
    }
}

fn apply_binop(op: BinOp, l: Value, r: Value) -> Result<Value, EvalError> {
    use BinOp::*;
    match (op, l, r) {
        (Add, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a.wrapping_add(b))),
        (Sub, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a.wrapping_sub(b))),
        (Mul, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a.wrapping_mul(b))),
        (Div, Value::Int(_), Value::Int(0)) => Err(EvalError::DivisionByZero),
        (Div, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a.wrapping_div(b))),
        (Lt, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a < b)),
        (Le, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a <= b)),
        (Gt, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a > b)),
        (Ge, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a >= b)),
        (Eq, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a == b)),
        (Eq, Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a == b)),
        (Ne, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a != b)),
        (Ne, Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a != b)),
        (op, _, _) => Err(EvalError::TypeError(format!(
            "type mismatch in operator {:?}",
            op
        ))),
    }
}
