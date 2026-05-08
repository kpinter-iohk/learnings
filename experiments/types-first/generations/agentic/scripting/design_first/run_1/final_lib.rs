use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
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
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Value {
    Int(i64),
    Bool(bool),
}

#[derive(Clone, PartialEq, Eq)]
pub struct ParseError {
    pub offset: usize,
    pub message: String,
}

#[derive(Clone, PartialEq, Eq)]
pub enum EvalError {
    TypeError(String),
    DivisionByZero,
    UnboundIdent(String),
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
        write!(
            f,
            "ParseError {{ offset: {}, message: {:?} }}",
            self.offset, self.message
        )
    }
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

// ---- Lexer ----

#[derive(Debug, Clone, PartialEq, Eq)]
enum Tok {
    Int(i64),
    Ident(String),
    // keywords
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
    // punctuation / operators
    Eq,    // =
    EqEq,  // ==
    NotEq, // !=
    Lt,
    LtEq,
    Gt,
    GtEq,
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

fn lex(input: &str) -> Result<Vec<Token>, ParseError> {
    let bytes = input.as_bytes();
    let mut i = 0;
    let mut out = Vec::new();
    while i < bytes.len() {
        let c = bytes[i];
        if c == b' ' || c == b'\t' || c == b'\n' || c == b'\r' {
            i += 1;
            continue;
        }
        let start = i;
        if c.is_ascii_digit() {
            let mut j = i;
            while j < bytes.len() && bytes[j].is_ascii_digit() {
                j += 1;
            }
            let s = &input[i..j];
            let n: i64 = s.parse().map_err(|_| ParseError {
                offset: start,
                message: format!("invalid integer literal: {}", s),
            })?;
            out.push(Token {
                tok: Tok::Int(n),
                offset: start,
            });
            i = j;
            continue;
        }
        if c == b'_' || c.is_ascii_lowercase() {
            let mut j = i;
            while j < bytes.len()
                && (bytes[j] == b'_'
                    || bytes[j].is_ascii_lowercase()
                    || bytes[j].is_ascii_digit())
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
            out.push(Token { tok, offset: start });
            i = j;
            continue;
        }
        // operators / punctuation
        match c {
            b'=' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'=' {
                    out.push(Token {
                        tok: Tok::EqEq,
                        offset: start,
                    });
                    i += 2;
                } else {
                    out.push(Token {
                        tok: Tok::Eq,
                        offset: start,
                    });
                    i += 1;
                }
            }
            b'!' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'=' {
                    out.push(Token {
                        tok: Tok::NotEq,
                        offset: start,
                    });
                    i += 2;
                } else {
                    return Err(ParseError {
                        offset: start,
                        message: "expected '=' after '!'".to_string(),
                    });
                }
            }
            b'<' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'=' {
                    out.push(Token {
                        tok: Tok::LtEq,
                        offset: start,
                    });
                    i += 2;
                } else {
                    out.push(Token {
                        tok: Tok::Lt,
                        offset: start,
                    });
                    i += 1;
                }
            }
            b'>' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'=' {
                    out.push(Token {
                        tok: Tok::GtEq,
                        offset: start,
                    });
                    i += 2;
                } else {
                    out.push(Token {
                        tok: Tok::Gt,
                        offset: start,
                    });
                    i += 1;
                }
            }
            b'+' => {
                out.push(Token {
                    tok: Tok::Plus,
                    offset: start,
                });
                i += 1;
            }
            b'-' => {
                out.push(Token {
                    tok: Tok::Minus,
                    offset: start,
                });
                i += 1;
            }
            b'*' => {
                out.push(Token {
                    tok: Tok::Star,
                    offset: start,
                });
                i += 1;
            }
            b'/' => {
                out.push(Token {
                    tok: Tok::Slash,
                    offset: start,
                });
                i += 1;
            }
            b'(' => {
                out.push(Token {
                    tok: Tok::LParen,
                    offset: start,
                });
                i += 1;
            }
            b')' => {
                out.push(Token {
                    tok: Tok::RParen,
                    offset: start,
                });
                i += 1;
            }
            _ => {
                return Err(ParseError {
                    offset: start,
                    message: format!("unexpected character: {:?}", c as char),
                });
            }
        }
    }
    Ok(out)
}

// ---- Parser ----

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    end_offset: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>, end_offset: usize) -> Self {
        Parser {
            tokens,
            pos: 0,
            end_offset,
        }
    }

    fn peek(&self) -> Option<&Tok> {
        self.tokens.get(self.pos).map(|t| &t.tok)
    }

    fn cur_offset(&self) -> usize {
        self.tokens
            .get(self.pos)
            .map(|t| t.offset)
            .unwrap_or(self.end_offset)
    }

    fn bump(&mut self) -> Option<&Token> {
        let t = self.tokens.get(self.pos);
        if t.is_some() {
            self.pos += 1;
        }
        t
    }

    fn eat(&mut self, t: &Tok) -> bool {
        if self.peek() == Some(t) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn expect(&mut self, t: &Tok, msg: &str) -> Result<(), ParseError> {
        if self.eat(t) {
            Ok(())
        } else {
            Err(ParseError {
                offset: self.cur_offset(),
                message: msg.to_string(),
            })
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
        self.bump(); // let
        let name = match self.peek().cloned() {
            Some(Tok::Ident(s)) => {
                self.bump();
                s
            }
            _ => {
                return Err(ParseError {
                    offset: self.cur_offset(),
                    message: "expected identifier after 'let'".to_string(),
                });
            }
        };
        self.expect(&Tok::Eq, "expected '=' in let")?;
        let value = self.parse_expr()?;
        self.expect(&Tok::In, "expected 'in' in let")?;
        let body = self.parse_expr()?;
        Ok(Expr::Let {
            name,
            value: Box::new(value),
            body: Box::new(body),
        })
    }

    fn parse_if(&mut self) -> Result<Expr, ParseError> {
        self.bump(); // if
        let cond = self.parse_expr()?;
        self.expect(&Tok::Then, "expected 'then' in if")?;
        let then_branch = self.parse_expr()?;
        self.expect(&Tok::Else, "expected 'else' in if")?;
        let else_branch = self.parse_expr()?;
        Ok(Expr::If {
            cond: Box::new(cond),
            then_branch: Box::new(then_branch),
            else_branch: Box::new(else_branch),
        })
    }

    fn parse_or(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_and()?;
        while self.eat(&Tok::Or) {
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
        while self.eat(&Tok::And) {
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
        if self.eat(&Tok::Not) {
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
        let op = match self.peek() {
            Some(Tok::EqEq) => Some(BinaryOp::Eq),
            Some(Tok::NotEq) => Some(BinaryOp::Ne),
            Some(Tok::Lt) => Some(BinaryOp::Lt),
            Some(Tok::LtEq) => Some(BinaryOp::Le),
            Some(Tok::Gt) => Some(BinaryOp::Gt),
            Some(Tok::GtEq) => Some(BinaryOp::Ge),
            _ => None,
        };
        if let Some(op) = op {
            self.bump();
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
            let op = match self.peek() {
                Some(Tok::Plus) => BinaryOp::Add,
                Some(Tok::Minus) => BinaryOp::Sub,
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
                Some(Tok::Star) => BinaryOp::Mul,
                Some(Tok::Slash) => BinaryOp::Div,
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
        if self.eat(&Tok::Minus) {
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
        let off = self.cur_offset();
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
                let e = self.parse_expr()?;
                self.expect(&Tok::RParen, "expected ')'")?;
                Ok(e)
            }
            _ => Err(ParseError {
                offset: off,
                message: "expected expression".to_string(),
            }),
        }
    }
}

pub fn parse(input: &str) -> Result<Expr, ParseError> {
    let tokens = lex(input)?;
    let end_offset = input.len();
    let mut p = Parser::new(tokens, end_offset);
    let e = p.parse_expr()?;
    if p.pos < p.tokens.len() {
        return Err(ParseError {
            offset: p.cur_offset(),
            message: "unexpected trailing input".to_string(),
        });
    }
    Ok(e)
}

// ---- Evaluator ----

fn lookup<'a>(env: &'a [(String, Value)], name: &str) -> Option<&'a Value> {
    for (k, v) in env.iter().rev() {
        if k == name {
            return Some(v);
        }
    }
    None
}

fn eval_with(expr: &Expr, env: &mut Vec<(String, Value)>) -> Result<Value, EvalError> {
    match expr {
        Expr::Int(n) => Ok(Value::Int(*n)),
        Expr::Bool(b) => Ok(Value::Bool(*b)),
        Expr::Ident(name) => lookup(env, name)
            .copied()
            .ok_or_else(|| EvalError::UnboundIdent(name.clone())),
        Expr::Let { name, value, body } => {
            let v = eval_with(value, env)?;
            env.push((name.clone(), v));
            let r = eval_with(body, env);
            env.pop();
            r
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
                    "if condition must be bool".to_string(),
                )),
            }
        }
        Expr::Unary { op, expr } => {
            let v = eval_with(expr, env)?;
            match (op, v) {
                (UnaryOp::Neg, Value::Int(n)) => Ok(Value::Int(-n)),
                (UnaryOp::Neg, Value::Bool(_)) => {
                    Err(EvalError::TypeError("'-' requires int".to_string()))
                }
                (UnaryOp::Not, Value::Bool(b)) => Ok(Value::Bool(!b)),
                (UnaryOp::Not, Value::Int(_)) => {
                    Err(EvalError::TypeError("'not' requires bool".to_string()))
                }
            }
        }
        Expr::Binary { op, lhs, rhs } => eval_binary(*op, lhs, rhs, env),
    }
}

fn eval_binary(
    op: BinaryOp,
    lhs: &Expr,
    rhs: &Expr,
    env: &mut Vec<(String, Value)>,
) -> Result<Value, EvalError> {
    // short-circuit for and/or
    match op {
        BinaryOp::And => {
            let l = eval_with(lhs, env)?;
            let lb = match l {
                Value::Bool(b) => b,
                Value::Int(_) => {
                    return Err(EvalError::TypeError("'and' requires bool".to_string()));
                }
            };
            if !lb {
                return Ok(Value::Bool(false));
            }
            let r = eval_with(rhs, env)?;
            match r {
                Value::Bool(b) => Ok(Value::Bool(b)),
                Value::Int(_) => Err(EvalError::TypeError("'and' requires bool".to_string())),
            }
        }
        BinaryOp::Or => {
            let l = eval_with(lhs, env)?;
            let lb = match l {
                Value::Bool(b) => b,
                Value::Int(_) => {
                    return Err(EvalError::TypeError("'or' requires bool".to_string()));
                }
            };
            if lb {
                return Ok(Value::Bool(true));
            }
            let r = eval_with(rhs, env)?;
            match r {
                Value::Bool(b) => Ok(Value::Bool(b)),
                Value::Int(_) => Err(EvalError::TypeError("'or' requires bool".to_string())),
            }
        }
        _ => {
            let l = eval_with(lhs, env)?;
            let r = eval_with(rhs, env)?;
            apply_binary(op, l, r)
        }
    }
}

fn apply_binary(op: BinaryOp, l: Value, r: Value) -> Result<Value, EvalError> {
    match op {
        BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div => {
            let (a, b) = match (l, r) {
                (Value::Int(a), Value::Int(b)) => (a, b),
                _ => {
                    return Err(EvalError::TypeError(
                        "arithmetic requires two ints".to_string(),
                    ));
                }
            };
            match op {
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
            }
        }
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
                "equality requires same-type operands".to_string(),
            )),
        },
        BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
            let (a, b) = match (l, r) {
                (Value::Int(a), Value::Int(b)) => (a, b),
                _ => {
                    return Err(EvalError::TypeError(
                        "ordering comparison requires two ints".to_string(),
                    ));
                }
            };
            let res = match op {
                BinaryOp::Lt => a < b,
                BinaryOp::Le => a <= b,
                BinaryOp::Gt => a > b,
                BinaryOp::Ge => a >= b,
                _ => unreachable!(),
            };
            Ok(Value::Bool(res))
        }
        BinaryOp::And | BinaryOp::Or => unreachable!(),
    }
}

pub fn eval(expr: &Expr) -> Result<Value, EvalError> {
    let mut env: Vec<(String, Value)> = Vec::new();
    eval_with(expr, &mut env)
}
