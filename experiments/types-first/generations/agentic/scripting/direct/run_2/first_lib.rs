use std::fmt;

#[derive(Debug)]
pub struct ParseError {
    pub offset: usize,
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "parse error at byte {}: {}", self.offset, self.message)
    }
}

#[derive(Debug)]
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

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Int(i64),
    Bool(bool),
    Ident(String),
    Let(String, Box<Expr>, Box<Expr>),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    Binary(BinOp, Box<Expr>, Box<Expr>),
    Unary(UnaryOp, Box<Expr>),
}

#[derive(Debug, Clone)]
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
}

#[derive(Debug, Clone)]
struct Token {
    tok: Tok,
    offset: usize,
}

fn tokenize(input: &str) -> Result<Vec<Token>, ParseError> {
    let bytes = input.as_bytes();
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        if c == b' ' || c == b'\t' || c == b'\n' || c == b'\r' {
            i += 1;
            continue;
        }
        let start = i;
        match c {
            b'+' => {
                tokens.push(Token { tok: Tok::Plus, offset: start });
                i += 1;
            }
            b'-' => {
                tokens.push(Token { tok: Tok::Minus, offset: start });
                i += 1;
            }
            b'*' => {
                tokens.push(Token { tok: Tok::Star, offset: start });
                i += 1;
            }
            b'/' => {
                tokens.push(Token { tok: Tok::Slash, offset: start });
                i += 1;
            }
            b'(' => {
                tokens.push(Token { tok: Tok::LParen, offset: start });
                i += 1;
            }
            b')' => {
                tokens.push(Token { tok: Tok::RParen, offset: start });
                i += 1;
            }
            b'=' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'=' {
                    tokens.push(Token { tok: Tok::EqEq, offset: start });
                    i += 2;
                } else {
                    tokens.push(Token { tok: Tok::Eq, offset: start });
                    i += 1;
                }
            }
            b'!' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'=' {
                    tokens.push(Token { tok: Tok::Ne, offset: start });
                    i += 2;
                } else {
                    return Err(ParseError {
                        offset: i,
                        message: "unexpected character '!'".to_string(),
                    });
                }
            }
            b'<' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'=' {
                    tokens.push(Token { tok: Tok::Le, offset: start });
                    i += 2;
                } else {
                    tokens.push(Token { tok: Tok::Lt, offset: start });
                    i += 1;
                }
            }
            b'>' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'=' {
                    tokens.push(Token { tok: Tok::Ge, offset: start });
                    i += 2;
                } else {
                    tokens.push(Token { tok: Tok::Gt, offset: start });
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
                    offset: i,
                    message: format!("invalid integer literal: {}", s),
                })?;
                tokens.push(Token { tok: Tok::Int(n), offset: start });
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
                tokens.push(Token { tok, offset: start });
                i = j;
            }
            _ => {
                return Err(ParseError {
                    offset: i,
                    message: format!("unexpected character: {:?}", c as char),
                });
            }
        }
    }
    Ok(tokens)
}

struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
    end_offset: usize,
}

impl<'a> Parser<'a> {
    fn peek(&self) -> Option<&Tok> {
        self.tokens.get(self.pos).map(|t| &t.tok)
    }

    fn current_offset(&self) -> usize {
        self.tokens
            .get(self.pos)
            .map(|t| t.offset)
            .unwrap_or(self.end_offset)
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        match self.peek() {
            Some(Tok::Let) => self.parse_let(),
            Some(Tok::If) => self.parse_if(),
            _ => self.parse_or(),
        }
    }

    fn parse_let(&mut self) -> Result<Expr, ParseError> {
        self.advance();
        let name = match self.peek().cloned() {
            Some(Tok::Ident(n)) => {
                self.advance();
                n
            }
            _ => {
                return Err(ParseError {
                    offset: self.current_offset(),
                    message: "expected identifier after 'let'".to_string(),
                });
            }
        };
        match self.peek() {
            Some(Tok::Eq) => self.advance(),
            _ => {
                return Err(ParseError {
                    offset: self.current_offset(),
                    message: "expected '=' in let binding".to_string(),
                });
            }
        }
        let value = self.parse_expr()?;
        match self.peek() {
            Some(Tok::In) => self.advance(),
            _ => {
                return Err(ParseError {
                    offset: self.current_offset(),
                    message: "expected 'in' in let expression".to_string(),
                });
            }
        }
        let body = self.parse_expr()?;
        Ok(Expr::Let(name, Box::new(value), Box::new(body)))
    }

    fn parse_if(&mut self) -> Result<Expr, ParseError> {
        self.advance();
        let cond = self.parse_expr()?;
        match self.peek() {
            Some(Tok::Then) => self.advance(),
            _ => {
                return Err(ParseError {
                    offset: self.current_offset(),
                    message: "expected 'then'".to_string(),
                });
            }
        }
        let then_branch = self.parse_expr()?;
        match self.peek() {
            Some(Tok::Else) => self.advance(),
            _ => {
                return Err(ParseError {
                    offset: self.current_offset(),
                    message: "expected 'else'".to_string(),
                });
            }
        }
        let else_branch = self.parse_expr()?;
        Ok(Expr::If(
            Box::new(cond),
            Box::new(then_branch),
            Box::new(else_branch),
        ))
    }

    fn parse_or(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_and()?;
        while matches!(self.peek(), Some(Tok::Or)) {
            self.advance();
            let right = self.parse_and()?;
            left = Expr::Binary(BinOp::Or, Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_not()?;
        while matches!(self.peek(), Some(Tok::And)) {
            self.advance();
            let right = self.parse_not()?;
            left = Expr::Binary(BinOp::And, Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_not(&mut self) -> Result<Expr, ParseError> {
        if matches!(self.peek(), Some(Tok::Not)) {
            self.advance();
            let inner = self.parse_not()?;
            Ok(Expr::Unary(UnaryOp::Not, Box::new(inner)))
        } else {
            self.parse_cmp()
        }
    }

    fn parse_cmp(&mut self) -> Result<Expr, ParseError> {
        let left = self.parse_add()?;
        let op = match self.peek() {
            Some(Tok::EqEq) => Some(BinOp::Eq),
            Some(Tok::Ne) => Some(BinOp::Ne),
            Some(Tok::Lt) => Some(BinOp::Lt),
            Some(Tok::Le) => Some(BinOp::Le),
            Some(Tok::Gt) => Some(BinOp::Gt),
            Some(Tok::Ge) => Some(BinOp::Ge),
            _ => None,
        };
        if let Some(op) = op {
            self.advance();
            let right = self.parse_add()?;
            Ok(Expr::Binary(op, Box::new(left), Box::new(right)))
        } else {
            Ok(left)
        }
    }

    fn parse_add(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_mul()?;
        loop {
            let op = match self.peek() {
                Some(Tok::Plus) => BinOp::Add,
                Some(Tok::Minus) => BinOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_mul()?;
            left = Expr::Binary(op, Box::new(left), Box::new(right));
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
            self.advance();
            let right = self.parse_unary()?;
            left = Expr::Binary(op, Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        if matches!(self.peek(), Some(Tok::Minus)) {
            self.advance();
            let inner = self.parse_unary()?;
            Ok(Expr::Unary(UnaryOp::Neg, Box::new(inner)))
        } else {
            self.parse_atom()
        }
    }

    fn parse_atom(&mut self) -> Result<Expr, ParseError> {
        let offset = self.current_offset();
        match self.peek().cloned() {
            Some(Tok::Int(n)) => {
                self.advance();
                Ok(Expr::Int(n))
            }
            Some(Tok::True) => {
                self.advance();
                Ok(Expr::Bool(true))
            }
            Some(Tok::False) => {
                self.advance();
                Ok(Expr::Bool(false))
            }
            Some(Tok::Ident(name)) => {
                self.advance();
                Ok(Expr::Ident(name))
            }
            Some(Tok::LParen) => {
                self.advance();
                let inner = self.parse_expr()?;
                match self.peek() {
                    Some(Tok::RParen) => {
                        self.advance();
                        Ok(inner)
                    }
                    _ => Err(ParseError {
                        offset: self.current_offset(),
                        message: "expected ')'".to_string(),
                    }),
                }
            }
            _ => Err(ParseError {
                offset,
                message: "expected expression".to_string(),
            }),
        }
    }
}

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
            message: "unexpected token after expression".to_string(),
        });
    }
    Ok(expr)
}

enum Env<'a> {
    Empty,
    Cons {
        name: &'a str,
        value: Value,
        parent: &'a Env<'a>,
    },
}

impl<'a> Env<'a> {
    fn lookup(&self, name: &str) -> Option<Value> {
        let mut cur = self;
        loop {
            match cur {
                Env::Empty => return None,
                Env::Cons { name: n, value, parent } => {
                    if *n == name {
                        return Some(*value);
                    }
                    cur = parent;
                }
            }
        }
    }
}

pub fn eval(expr: &Expr) -> Result<Value, EvalError> {
    eval_with(expr, &Env::Empty)
}

fn eval_with(expr: &Expr, env: &Env) -> Result<Value, EvalError> {
    match expr {
        Expr::Int(n) => Ok(Value::Int(*n)),
        Expr::Bool(b) => Ok(Value::Bool(*b)),
        Expr::Ident(name) => env
            .lookup(name)
            .ok_or_else(|| EvalError::UnboundIdentifier(name.clone())),
        Expr::Let(name, val, body) => {
            let v = eval_with(val, env)?;
            let new_env = Env::Cons {
                name: name.as_str(),
                value: v,
                parent: env,
            };
            eval_with(body, &new_env)
        }
        Expr::If(cond, t, e) => {
            let c = eval_with(cond, env)?;
            match c {
                Value::Bool(true) => eval_with(t, env),
                Value::Bool(false) => eval_with(e, env),
                Value::Int(_) => Err(EvalError::TypeError(
                    "if condition must be a bool".to_string(),
                )),
            }
        }
        Expr::Unary(op, inner) => {
            let v = eval_with(inner, env)?;
            match (op, v) {
                (UnaryOp::Neg, Value::Int(n)) => Ok(Value::Int(n.wrapping_neg())),
                (UnaryOp::Neg, Value::Bool(_)) => Err(EvalError::TypeError(
                    "unary '-' requires an int".to_string(),
                )),
                (UnaryOp::Not, Value::Bool(b)) => Ok(Value::Bool(!b)),
                (UnaryOp::Not, Value::Int(_)) => {
                    Err(EvalError::TypeError("'not' requires a bool".to_string()))
                }
            }
        }
        Expr::Binary(op, l, r) => {
            let lv = eval_with(l, env)?;
            let rv = eval_with(r, env)?;
            eval_binop(*op, lv, rv)
        }
    }
}

fn eval_binop(op: BinOp, lv: Value, rv: Value) -> Result<Value, EvalError> {
    match op {
        BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div => match (lv, rv) {
            (Value::Int(a), Value::Int(b)) => match op {
                BinOp::Add => Ok(Value::Int(a.wrapping_add(b))),
                BinOp::Sub => Ok(Value::Int(a.wrapping_sub(b))),
                BinOp::Mul => Ok(Value::Int(a.wrapping_mul(b))),
                BinOp::Div => {
                    if b == 0 {
                        Err(EvalError::DivisionByZero)
                    } else {
                        Ok(Value::Int(a.wrapping_div(b)))
                    }
                }
                _ => unreachable!(),
            },
            _ => Err(EvalError::TypeError(
                "arithmetic operands must be ints".to_string(),
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
                "ordering comparison operands must be ints".to_string(),
            )),
        },
        BinOp::Eq | BinOp::Ne => {
            let eq = match (lv, rv) {
                (Value::Int(a), Value::Int(b)) => a == b,
                (Value::Bool(a), Value::Bool(b)) => a == b,
                _ => {
                    return Err(EvalError::TypeError(
                        "'==' / '!=' require operands of the same type".to_string(),
                    ));
                }
            };
            let r = if matches!(op, BinOp::Eq) { eq } else { !eq };
            Ok(Value::Bool(r))
        }
        BinOp::And | BinOp::Or => match (lv, rv) {
            (Value::Bool(a), Value::Bool(b)) => {
                let r = match op {
                    BinOp::And => a && b,
                    BinOp::Or => a || b,
                    _ => unreachable!(),
                };
                Ok(Value::Bool(r))
            }
            _ => Err(EvalError::TypeError(
                "'and' / 'or' require bool operands".to_string(),
            )),
        },
    }
}
