use std::fmt;

#[derive(Debug, Clone, PartialEq)]
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
    BinOp {
        op: BinOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    UnaryOp {
        op: UnaryOp,
        operand: Box<Expr>,
    },
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value {
    Int(i64),
    Bool(bool),
}

#[derive(Clone, PartialEq)]
pub struct ParseError {
    pub offset: usize,
    pub message: String,
}

#[derive(Clone, PartialEq)]
pub enum EvalError {
    TypeError(String),
    DivisionByZero,
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

#[derive(Debug, Clone, PartialEq)]
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
    Assign,
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

struct Tokenized {
    tok: Tok,
    pos: usize,
}

fn tokenize(input: &str) -> Result<Vec<Tokenized>, ParseError> {
    let bytes = input.as_bytes();
    let mut i = 0usize;
    let mut tokens = Vec::new();
    while i < bytes.len() {
        let b = bytes[i];
        if b.is_ascii_whitespace() {
            i += 1;
            continue;
        }
        let pos = i;
        if b.is_ascii_digit() {
            let start = i;
            while i < bytes.len() && bytes[i].is_ascii_digit() {
                i += 1;
            }
            let s = &input[start..i];
            let n: i64 = s.parse().map_err(|_| ParseError {
                offset: start,
                message: format!("invalid integer literal: {}", s),
            })?;
            tokens.push(Tokenized {
                tok: Tok::Int(n),
                pos,
            });
            continue;
        }
        if b == b'_' || b.is_ascii_lowercase() {
            let start = i;
            while i < bytes.len()
                && (bytes[i] == b'_'
                    || bytes[i].is_ascii_lowercase()
                    || bytes[i].is_ascii_digit())
            {
                i += 1;
            }
            let s = &input[start..i];
            let tok = match s {
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
            tokens.push(Tokenized { tok, pos });
            continue;
        }
        if i + 1 < bytes.len() {
            let two = &input[i..i + 2];
            let two_tok = match two {
                "==" => Some(Tok::EqEq),
                "!=" => Some(Tok::Ne),
                "<=" => Some(Tok::Le),
                ">=" => Some(Tok::Ge),
                _ => None,
            };
            if let Some(t) = two_tok {
                tokens.push(Tokenized { tok: t, pos });
                i += 2;
                continue;
            }
        }
        let single = match b {
            b'=' => Some(Tok::Assign),
            b'<' => Some(Tok::Lt),
            b'>' => Some(Tok::Gt),
            b'+' => Some(Tok::Plus),
            b'-' => Some(Tok::Minus),
            b'*' => Some(Tok::Star),
            b'/' => Some(Tok::Slash),
            b'(' => Some(Tok::LParen),
            b')' => Some(Tok::RParen),
            _ => None,
        };
        if let Some(t) = single {
            tokens.push(Tokenized { tok: t, pos });
            i += 1;
            continue;
        }
        return Err(ParseError {
            offset: pos,
            message: format!("unexpected character: {:?}", b as char),
        });
    }
    tokens.push(Tokenized {
        tok: Tok::Eof,
        pos: input.len(),
    });
    Ok(tokens)
}

struct Parser {
    tokens: Vec<Tokenized>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> &Tok {
        &self.tokens[self.pos].tok
    }

    fn peek_pos(&self) -> usize {
        self.tokens[self.pos].pos
    }

    fn advance(&mut self) {
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        match self.peek() {
            Tok::KwLet => self.parse_let(),
            Tok::KwIf => self.parse_if(),
            _ => self.parse_or(),
        }
    }

    fn parse_let(&mut self) -> Result<Expr, ParseError> {
        self.advance(); // 'let'
        let name = match self.peek() {
            Tok::Ident(s) => s.clone(),
            _ => {
                return Err(ParseError {
                    offset: self.peek_pos(),
                    message: "expected identifier after 'let'".to_string(),
                });
            }
        };
        self.advance();
        if !matches!(self.peek(), Tok::Assign) {
            return Err(ParseError {
                offset: self.peek_pos(),
                message: "expected '='".to_string(),
            });
        }
        self.advance();
        let value = self.parse_expr()?;
        if !matches!(self.peek(), Tok::KwIn) {
            return Err(ParseError {
                offset: self.peek_pos(),
                message: "expected 'in'".to_string(),
            });
        }
        self.advance();
        let body = self.parse_expr()?;
        Ok(Expr::Let {
            name,
            value: Box::new(value),
            body: Box::new(body),
        })
    }

    fn parse_if(&mut self) -> Result<Expr, ParseError> {
        self.advance(); // 'if'
        let cond = self.parse_expr()?;
        if !matches!(self.peek(), Tok::KwThen) {
            return Err(ParseError {
                offset: self.peek_pos(),
                message: "expected 'then'".to_string(),
            });
        }
        self.advance();
        let then_branch = self.parse_expr()?;
        if !matches!(self.peek(), Tok::KwElse) {
            return Err(ParseError {
                offset: self.peek_pos(),
                message: "expected 'else'".to_string(),
            });
        }
        self.advance();
        let else_branch = self.parse_expr()?;
        Ok(Expr::If {
            cond: Box::new(cond),
            then_branch: Box::new(then_branch),
            else_branch: Box::new(else_branch),
        })
    }

    fn parse_or(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_and()?;
        while matches!(self.peek(), Tok::KwOr) {
            self.advance();
            let rhs = self.parse_and()?;
            lhs = Expr::BinOp {
                op: BinOp::Or,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            };
        }
        Ok(lhs)
    }

    fn parse_and(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_not()?;
        while matches!(self.peek(), Tok::KwAnd) {
            self.advance();
            let rhs = self.parse_not()?;
            lhs = Expr::BinOp {
                op: BinOp::And,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            };
        }
        Ok(lhs)
    }

    fn parse_not(&mut self) -> Result<Expr, ParseError> {
        if matches!(self.peek(), Tok::KwNot) {
            self.advance();
            let operand = self.parse_not()?;
            Ok(Expr::UnaryOp {
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
            Tok::EqEq => Some(BinOp::Eq),
            Tok::Ne => Some(BinOp::Ne),
            Tok::Lt => Some(BinOp::Lt),
            Tok::Le => Some(BinOp::Le),
            Tok::Gt => Some(BinOp::Gt),
            Tok::Ge => Some(BinOp::Ge),
            _ => None,
        };
        if let Some(op) = op {
            self.advance();
            let rhs = self.parse_add()?;
            Ok(Expr::BinOp {
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
                Tok::Plus => BinOp::Add,
                Tok::Minus => BinOp::Sub,
                _ => break,
            };
            self.advance();
            let rhs = self.parse_mul()?;
            lhs = Expr::BinOp {
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
                Tok::Star => BinOp::Mul,
                Tok::Slash => BinOp::Div,
                _ => break,
            };
            self.advance();
            let rhs = self.parse_unary()?;
            lhs = Expr::BinOp {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            };
        }
        Ok(lhs)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        if matches!(self.peek(), Tok::Minus) {
            self.advance();
            let operand = self.parse_unary()?;
            Ok(Expr::UnaryOp {
                op: UnaryOp::Neg,
                operand: Box::new(operand),
            })
        } else {
            self.parse_atom()
        }
    }

    fn parse_atom(&mut self) -> Result<Expr, ParseError> {
        let pos = self.peek_pos();
        match self.peek().clone() {
            Tok::Int(n) => {
                self.advance();
                Ok(Expr::Int(n))
            }
            Tok::KwTrue => {
                self.advance();
                Ok(Expr::Bool(true))
            }
            Tok::KwFalse => {
                self.advance();
                Ok(Expr::Bool(false))
            }
            Tok::Ident(s) => {
                self.advance();
                Ok(Expr::Var(s))
            }
            Tok::LParen => {
                self.advance();
                let e = self.parse_expr()?;
                if !matches!(self.peek(), Tok::RParen) {
                    return Err(ParseError {
                        offset: self.peek_pos(),
                        message: "expected ')'".to_string(),
                    });
                }
                self.advance();
                Ok(e)
            }
            _ => Err(ParseError {
                offset: pos,
                message: "expected expression".to_string(),
            }),
        }
    }
}

pub fn parse(input: &str) -> Result<Expr, ParseError> {
    let tokens = tokenize(input)?;
    let mut parser = Parser { tokens, pos: 0 };
    let expr = parser.parse_expr()?;
    if !matches!(parser.peek(), Tok::Eof) {
        return Err(ParseError {
            offset: parser.peek_pos(),
            message: "unexpected token after expression".to_string(),
        });
    }
    Ok(expr)
}

pub fn eval(expr: &Expr) -> Result<Value, EvalError> {
    let mut env: Vec<(String, Value)> = Vec::new();
    eval_with(expr, &mut env)
}

fn eval_with(expr: &Expr, env: &mut Vec<(String, Value)>) -> Result<Value, EvalError> {
    match expr {
        Expr::Int(n) => Ok(Value::Int(*n)),
        Expr::Bool(b) => Ok(Value::Bool(*b)),
        Expr::Var(name) => {
            for (n, v) in env.iter().rev() {
                if n == name {
                    return Ok(*v);
                }
            }
            Err(EvalError::UnboundIdentifier(name.clone()))
        }
        Expr::Let { name, value, body } => {
            let v = eval_with(value, env)?;
            env.push((name.clone(), v));
            let result = eval_with(body, env);
            env.pop();
            result
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
        Expr::UnaryOp { op, operand } => {
            let v = eval_with(operand, env)?;
            match (op, v) {
                (UnaryOp::Neg, Value::Int(n)) => Ok(Value::Int(n.wrapping_neg())),
                (UnaryOp::Neg, _) => {
                    Err(EvalError::TypeError("unary '-' requires integer".to_string()))
                }
                (UnaryOp::Not, Value::Bool(b)) => Ok(Value::Bool(!b)),
                (UnaryOp::Not, _) => {
                    Err(EvalError::TypeError("'not' requires bool".to_string()))
                }
            }
        }
        Expr::BinOp { op, lhs, rhs } => {
            let l = eval_with(lhs, env)?;
            let r = eval_with(rhs, env)?;
            apply_binop(*op, l, r)
        }
    }
}

fn apply_binop(op: BinOp, l: Value, r: Value) -> Result<Value, EvalError> {
    match op {
        BinOp::Or => match (l, r) {
            (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a || b)),
            _ => Err(EvalError::TypeError(
                "'or' requires bool operands".to_string(),
            )),
        },
        BinOp::And => match (l, r) {
            (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a && b)),
            _ => Err(EvalError::TypeError(
                "'and' requires bool operands".to_string(),
            )),
        },
        BinOp::Eq => match (l, r) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a == b)),
            (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a == b)),
            _ => Err(EvalError::TypeError(
                "'==' requires same-typed operands".to_string(),
            )),
        },
        BinOp::Ne => match (l, r) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a != b)),
            (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a != b)),
            _ => Err(EvalError::TypeError(
                "'!=' requires same-typed operands".to_string(),
            )),
        },
        BinOp::Lt => match (l, r) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a < b)),
            _ => Err(EvalError::TypeError(
                "'<' requires integer operands".to_string(),
            )),
        },
        BinOp::Le => match (l, r) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a <= b)),
            _ => Err(EvalError::TypeError(
                "'<=' requires integer operands".to_string(),
            )),
        },
        BinOp::Gt => match (l, r) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a > b)),
            _ => Err(EvalError::TypeError(
                "'>' requires integer operands".to_string(),
            )),
        },
        BinOp::Ge => match (l, r) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a >= b)),
            _ => Err(EvalError::TypeError(
                "'>=' requires integer operands".to_string(),
            )),
        },
        BinOp::Add => match (l, r) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a.wrapping_add(b))),
            _ => Err(EvalError::TypeError(
                "'+' requires integer operands".to_string(),
            )),
        },
        BinOp::Sub => match (l, r) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a.wrapping_sub(b))),
            _ => Err(EvalError::TypeError(
                "'-' requires integer operands".to_string(),
            )),
        },
        BinOp::Mul => match (l, r) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a.wrapping_mul(b))),
            _ => Err(EvalError::TypeError(
                "'*' requires integer operands".to_string(),
            )),
        },
        BinOp::Div => match (l, r) {
            (Value::Int(_), Value::Int(0)) => Err(EvalError::DivisionByZero),
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a.wrapping_div(b))),
            _ => Err(EvalError::TypeError(
                "'/' requires integer operands".to_string(),
            )),
        },
    }
}
