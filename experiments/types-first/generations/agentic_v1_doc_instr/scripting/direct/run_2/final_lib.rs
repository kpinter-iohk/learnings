use std::fmt;

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone)]
pub struct ParseError {
    pub offset: usize,
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "parse error at byte {}: {}", self.offset, self.message)
    }
}

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

#[derive(Debug, Clone)]
enum Token {
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
    Eof,
}

fn token_name(t: &Token) -> String {
    match t {
        Token::Int(n) => format!("integer {}", n),
        Token::Ident(s) => format!("identifier `{}`", s),
        Token::True => "`true`".to_string(),
        Token::False => "`false`".to_string(),
        Token::Let => "`let`".to_string(),
        Token::In => "`in`".to_string(),
        Token::If => "`if`".to_string(),
        Token::Then => "`then`".to_string(),
        Token::Else => "`else`".to_string(),
        Token::Or => "`or`".to_string(),
        Token::And => "`and`".to_string(),
        Token::Not => "`not`".to_string(),
        Token::Plus => "`+`".to_string(),
        Token::Minus => "`-`".to_string(),
        Token::Star => "`*`".to_string(),
        Token::Slash => "`/`".to_string(),
        Token::EqEq => "`==`".to_string(),
        Token::NotEq => "`!=`".to_string(),
        Token::Lt => "`<`".to_string(),
        Token::Le => "`<=`".to_string(),
        Token::Gt => "`>`".to_string(),
        Token::Ge => "`>=`".to_string(),
        Token::Assign => "`=`".to_string(),
        Token::LParen => "`(`".to_string(),
        Token::RParen => "`)`".to_string(),
        Token::Eof => "end of input".to_string(),
    }
}

struct Tokenizer<'a> {
    src: &'a [u8],
    pos: usize,
}

impl<'a> Tokenizer<'a> {
    fn new(input: &'a str) -> Self {
        Tokenizer {
            src: input.as_bytes(),
            pos: 0,
        }
    }

    fn skip_ws(&mut self) {
        while self.pos < self.src.len() && self.src[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
    }

    fn next_token(&mut self) -> Result<(Token, usize), ParseError> {
        self.skip_ws();
        let start = self.pos;
        if self.pos >= self.src.len() {
            return Ok((Token::Eof, start));
        }
        let c = self.src[self.pos];
        match c {
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
                if self.pos + 1 < self.src.len() && self.src[self.pos + 1] == b'=' {
                    self.pos += 2;
                    Ok((Token::EqEq, start))
                } else {
                    self.pos += 1;
                    Ok((Token::Assign, start))
                }
            }
            b'!' => {
                if self.pos + 1 < self.src.len() && self.src[self.pos + 1] == b'=' {
                    self.pos += 2;
                    Ok((Token::NotEq, start))
                } else {
                    Err(ParseError {
                        offset: start,
                        message: "expected `!=` after `!`".to_string(),
                    })
                }
            }
            b'<' => {
                if self.pos + 1 < self.src.len() && self.src[self.pos + 1] == b'=' {
                    self.pos += 2;
                    Ok((Token::Le, start))
                } else {
                    self.pos += 1;
                    Ok((Token::Lt, start))
                }
            }
            b'>' => {
                if self.pos + 1 < self.src.len() && self.src[self.pos + 1] == b'=' {
                    self.pos += 2;
                    Ok((Token::Ge, start))
                } else {
                    self.pos += 1;
                    Ok((Token::Gt, start))
                }
            }
            b'0'..=b'9' => {
                let s = self.pos;
                while self.pos < self.src.len() && self.src[self.pos].is_ascii_digit() {
                    self.pos += 1;
                }
                let text = std::str::from_utf8(&self.src[s..self.pos]).unwrap();
                match text.parse::<i64>() {
                    Ok(n) => Ok((Token::Int(n), start)),
                    Err(_) => Err(ParseError {
                        offset: start,
                        message: format!("integer literal out of range: `{}`", text),
                    }),
                }
            }
            b'a'..=b'z' | b'_' => {
                let s = self.pos;
                while self.pos < self.src.len()
                    && (self.src[self.pos].is_ascii_lowercase()
                        || self.src[self.pos].is_ascii_digit()
                        || self.src[self.pos] == b'_')
                {
                    self.pos += 1;
                }
                let text = std::str::from_utf8(&self.src[s..self.pos]).unwrap();
                let tok = match text {
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
                    _ => Token::Ident(text.to_string()),
                };
                Ok((tok, start))
            }
            _ => Err(ParseError {
                offset: start,
                message: format!("unexpected byte 0x{:02x}", c),
            }),
        }
    }

    fn tokenize(mut self) -> Result<Vec<(Token, usize)>, ParseError> {
        let mut out = Vec::new();
        loop {
            let (tok, off) = self.next_token()?;
            let is_eof = matches!(tok, Token::Eof);
            out.push((tok, off));
            if is_eof {
                break;
            }
        }
        Ok(out)
    }
}

struct Parser {
    tokens: Vec<(Token, usize)>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> &Token {
        &self.tokens[self.pos].0
    }

    fn peek_offset(&self) -> usize {
        self.tokens[self.pos].1
    }

    fn advance(&mut self) -> (Token, usize) {
        let t = self.tokens[self.pos].clone();
        if self.pos + 1 < self.tokens.len() {
            self.pos += 1;
        }
        t
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        match self.peek() {
            Token::Let => self.parse_let(),
            Token::If => self.parse_if(),
            _ => self.parse_or(),
        }
    }

    fn parse_let(&mut self) -> Result<Expr, ParseError> {
        self.advance();
        let (name, _) = match self.advance() {
            (Token::Ident(n), o) => (n, o),
            (other, o) => {
                return Err(ParseError {
                    offset: o,
                    message: format!("expected identifier after `let`, found {}", token_name(&other)),
                });
            }
        };
        match self.advance() {
            (Token::Assign, _) => {}
            (other, o) => {
                return Err(ParseError {
                    offset: o,
                    message: format!("expected `=` after `let` identifier, found {}", token_name(&other)),
                });
            }
        }
        let value = self.parse_expr()?;
        match self.advance() {
            (Token::In, _) => {}
            (other, o) => {
                return Err(ParseError {
                    offset: o,
                    message: format!("expected `in`, found {}", token_name(&other)),
                });
            }
        }
        let body = self.parse_expr()?;
        Ok(Expr::Let(name, Box::new(value), Box::new(body)))
    }

    fn parse_if(&mut self) -> Result<Expr, ParseError> {
        self.advance();
        let cond = self.parse_expr()?;
        match self.advance() {
            (Token::Then, _) => {}
            (other, o) => {
                return Err(ParseError {
                    offset: o,
                    message: format!("expected `then`, found {}", token_name(&other)),
                });
            }
        }
        let then_e = self.parse_expr()?;
        match self.advance() {
            (Token::Else, _) => {}
            (other, o) => {
                return Err(ParseError {
                    offset: o,
                    message: format!("expected `else`, found {}", token_name(&other)),
                });
            }
        }
        let else_e = self.parse_expr()?;
        Ok(Expr::If(Box::new(cond), Box::new(then_e), Box::new(else_e)))
    }

    fn parse_or(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_and()?;
        while matches!(self.peek(), Token::Or) {
            self.advance();
            let right = self.parse_and()?;
            left = Expr::BinOp(BinOp::Or, Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_not()?;
        while matches!(self.peek(), Token::And) {
            self.advance();
            let right = self.parse_not()?;
            left = Expr::BinOp(BinOp::And, Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_not(&mut self) -> Result<Expr, ParseError> {
        if matches!(self.peek(), Token::Not) {
            self.advance();
            let e = self.parse_not()?;
            Ok(Expr::Not(Box::new(e)))
        } else {
            self.parse_cmp()
        }
    }

    fn parse_cmp(&mut self) -> Result<Expr, ParseError> {
        let left = self.parse_add()?;
        let op = match self.peek() {
            Token::EqEq => Some(BinOp::Eq),
            Token::NotEq => Some(BinOp::Ne),
            Token::Lt => Some(BinOp::Lt),
            Token::Le => Some(BinOp::Le),
            Token::Gt => Some(BinOp::Gt),
            Token::Ge => Some(BinOp::Ge),
            _ => None,
        };
        if let Some(op) = op {
            self.advance();
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
                Token::Plus => BinOp::Add,
                Token::Minus => BinOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_mul()?;
            left = Expr::BinOp(op, Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_mul(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_unary()?;
        loop {
            let op = match self.peek() {
                Token::Star => BinOp::Mul,
                Token::Slash => BinOp::Div,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary()?;
            left = Expr::BinOp(op, Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        if matches!(self.peek(), Token::Minus) {
            self.advance();
            let e = self.parse_unary()?;
            Ok(Expr::Neg(Box::new(e)))
        } else {
            self.parse_atom()
        }
    }

    fn parse_atom(&mut self) -> Result<Expr, ParseError> {
        let (tok, offset) = self.advance();
        match tok {
            Token::Int(n) => Ok(Expr::Int(n)),
            Token::True => Ok(Expr::Bool(true)),
            Token::False => Ok(Expr::Bool(false)),
            Token::Ident(name) => Ok(Expr::Ident(name)),
            Token::LParen => {
                let e = self.parse_expr()?;
                match self.advance() {
                    (Token::RParen, _) => Ok(e),
                    (other, o) => Err(ParseError {
                        offset: o,
                        message: format!("expected `)`, found {}", token_name(&other)),
                    }),
                }
            }
            other => Err(ParseError {
                offset,
                message: format!("expected expression, found {}", token_name(&other)),
            }),
        }
    }
}

pub fn parse(input: &str) -> Result<Expr, ParseError> {
    let tokens = Tokenizer::new(input).tokenize()?;
    let mut parser = Parser { tokens, pos: 0 };
    let expr = parser.parse_expr()?;
    match parser.peek() {
        Token::Eof => Ok(expr),
        other => Err(ParseError {
            offset: parser.peek_offset(),
            message: format!("unexpected {} after expression", token_name(other)),
        }),
    }
}

enum Env<'a> {
    Empty,
    Cons(&'a str, Value, &'a Env<'a>),
}

impl<'a> Env<'a> {
    fn lookup(&self, name: &str) -> Option<Value> {
        match self {
            Env::Empty => None,
            Env::Cons(n, v, parent) => {
                if *n == name {
                    Some(*v)
                } else {
                    parent.lookup(name)
                }
            }
        }
    }
}

fn eval_with_env(expr: &Expr, env: &Env) -> Result<Value, EvalError> {
    match expr {
        Expr::Int(i) => Ok(Value::Int(*i)),
        Expr::Bool(b) => Ok(Value::Bool(*b)),
        Expr::Ident(name) => env
            .lookup(name)
            .ok_or_else(|| EvalError::UnboundIdent(name.clone())),
        Expr::Neg(e) => match eval_with_env(e, env)? {
            Value::Int(i) => Ok(Value::Int(i.wrapping_neg())),
            Value::Bool(_) => Err(EvalError::TypeError(
                "unary `-` requires an integer".to_string(),
            )),
        },
        Expr::Not(e) => match eval_with_env(e, env)? {
            Value::Bool(b) => Ok(Value::Bool(!b)),
            Value::Int(_) => Err(EvalError::TypeError(
                "`not` requires a boolean".to_string(),
            )),
        },
        Expr::BinOp(op, l, r) => match op {
            BinOp::And => {
                let lv = eval_with_env(l, env)?;
                let lb = match lv {
                    Value::Bool(b) => b,
                    _ => {
                        return Err(EvalError::TypeError(
                            "`and` requires booleans".to_string(),
                        ));
                    }
                };
                if !lb {
                    return Ok(Value::Bool(false));
                }
                match eval_with_env(r, env)? {
                    Value::Bool(b) => Ok(Value::Bool(b)),
                    _ => Err(EvalError::TypeError(
                        "`and` requires booleans".to_string(),
                    )),
                }
            }
            BinOp::Or => {
                let lv = eval_with_env(l, env)?;
                let lb = match lv {
                    Value::Bool(b) => b,
                    _ => {
                        return Err(EvalError::TypeError(
                            "`or` requires booleans".to_string(),
                        ));
                    }
                };
                if lb {
                    return Ok(Value::Bool(true));
                }
                match eval_with_env(r, env)? {
                    Value::Bool(b) => Ok(Value::Bool(b)),
                    _ => Err(EvalError::TypeError(
                        "`or` requires booleans".to_string(),
                    )),
                }
            }
            _ => {
                let lv = eval_with_env(l, env)?;
                let rv = eval_with_env(r, env)?;
                eval_binop(*op, lv, rv)
            }
        },
        Expr::If(c, t, e) => match eval_with_env(c, env)? {
            Value::Bool(true) => eval_with_env(t, env),
            Value::Bool(false) => eval_with_env(e, env),
            Value::Int(_) => Err(EvalError::TypeError(
                "`if` condition must be boolean".to_string(),
            )),
        },
        Expr::Let(name, value, body) => {
            let v = eval_with_env(value, env)?;
            let new_env = Env::Cons(name.as_str(), v, env);
            eval_with_env(body, &new_env)
        }
    }
}

fn eval_binop(op: BinOp, l: Value, r: Value) -> Result<Value, EvalError> {
    match op {
        BinOp::Add => match (l, r) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a.wrapping_add(b))),
            _ => Err(EvalError::TypeError("`+` requires integers".to_string())),
        },
        BinOp::Sub => match (l, r) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a.wrapping_sub(b))),
            _ => Err(EvalError::TypeError("`-` requires integers".to_string())),
        },
        BinOp::Mul => match (l, r) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a.wrapping_mul(b))),
            _ => Err(EvalError::TypeError("`*` requires integers".to_string())),
        },
        BinOp::Div => match (l, r) {
            (Value::Int(_), Value::Int(0)) => Err(EvalError::DivisionByZero),
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a.wrapping_div(b))),
            _ => Err(EvalError::TypeError("`/` requires integers".to_string())),
        },
        BinOp::Eq => match (l, r) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a == b)),
            (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a == b)),
            _ => Err(EvalError::TypeError(
                "`==` requires operands of the same type".to_string(),
            )),
        },
        BinOp::Ne => match (l, r) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a != b)),
            (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a != b)),
            _ => Err(EvalError::TypeError(
                "`!=` requires operands of the same type".to_string(),
            )),
        },
        BinOp::Lt => match (l, r) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a < b)),
            _ => Err(EvalError::TypeError("`<` requires integers".to_string())),
        },
        BinOp::Le => match (l, r) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a <= b)),
            _ => Err(EvalError::TypeError("`<=` requires integers".to_string())),
        },
        BinOp::Gt => match (l, r) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a > b)),
            _ => Err(EvalError::TypeError("`>` requires integers".to_string())),
        },
        BinOp::Ge => match (l, r) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a >= b)),
            _ => Err(EvalError::TypeError("`>=` requires integers".to_string())),
        },
        BinOp::And | BinOp::Or => unreachable!(),
    }
}

pub fn eval(expr: &Expr) -> Result<Value, EvalError> {
    eval_with_env(expr, &Env::Empty)
}
