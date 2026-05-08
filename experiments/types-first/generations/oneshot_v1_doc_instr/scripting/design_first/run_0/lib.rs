use std::fmt;

// ===========================================================================
// Abstract syntax
// ===========================================================================

/// A parsed expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    /// Integer literal.
    Int(i64),
    /// Boolean literal.
    Bool(bool),
    /// Identifier reference.
    Var(String),
    /// `let <name> = <value> in <body>`.
    Let {
        name: String,
        value: Box<Expr>,
        body: Box<Expr>,
    },
    /// `if <cond> then <then_branch> else <else_branch>`.
    If {
        cond: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Box<Expr>,
    },
    /// Binary operation.
    BinOp {
        op: BinOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    /// Unary operation.
    UnOp {
        op: UnOp,
        operand: Box<Expr>,
    },
}

/// All binary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Or, And,
    Eq, Ne, Lt, Le, Gt, Ge,
    Add, Sub, Mul, Div,
}

/// All unary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    Not,
    Neg,
}

// ===========================================================================
// Runtime values
// ===========================================================================

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

// ===========================================================================
// Errors
// ===========================================================================

/// What kind of parse error occurred.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseErrorKind {
    UnexpectedChar(char),
    UnexpectedToken,
    UnexpectedEof,
    IntegerOverflow,
    Expected(&'static str),
    ExpectedIdentifier,
    TrailingInput,
}

impl fmt::Display for ParseErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseErrorKind::UnexpectedChar(c) => write!(f, "unexpected character {:?}", c),
            ParseErrorKind::UnexpectedToken => write!(f, "unexpected token"),
            ParseErrorKind::UnexpectedEof => write!(f, "unexpected end of input"),
            ParseErrorKind::IntegerOverflow => write!(f, "integer literal does not fit in i64"),
            ParseErrorKind::Expected(s) => write!(f, "expected {}", s),
            ParseErrorKind::ExpectedIdentifier => write!(f, "expected identifier"),
            ParseErrorKind::TrailingInput => write!(f, "unexpected trailing input"),
        }
    }
}

/// A parse error, including the byte offset within the input where it occurred.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub offset: usize,
    pub kind: ParseErrorKind,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "parse error at byte offset {}: {}", self.offset, self.kind)
    }
}

/// A runtime evaluation error.
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

// ===========================================================================
// Lexer
// ===========================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
enum Tok {
    Int(i64),
    Ident(String),
    Let, In, If, Then, Else, Or, And, Not, True, False,
    Eq, EqEq, NotEq, Lt, Le, Gt, Ge,
    Plus, Minus, Star, Slash,
    LParen, RParen,
    Eof,
}

impl Tok {
    fn description(&self) -> &'static str {
        match self {
            Tok::Int(_) => "integer literal",
            Tok::Ident(_) => "identifier",
            Tok::Let => "let",
            Tok::In => "in",
            Tok::If => "if",
            Tok::Then => "then",
            Tok::Else => "else",
            Tok::Or => "or",
            Tok::And => "and",
            Tok::Not => "not",
            Tok::True => "true",
            Tok::False => "false",
            Tok::Eq => "=",
            Tok::EqEq => "==",
            Tok::NotEq => "!=",
            Tok::Lt => "<",
            Tok::Le => "<=",
            Tok::Gt => ">",
            Tok::Ge => ">=",
            Tok::Plus => "+",
            Tok::Minus => "-",
            Tok::Star => "*",
            Tok::Slash => "/",
            Tok::LParen => "(",
            Tok::RParen => ")",
            Tok::Eof => "end of input",
        }
    }
}

// ===========================================================================
// Parser
// ===========================================================================

struct Parser<'a> {
    input: &'a str,
    bytes: &'a [u8],
    pos: usize,
    /// The lookahead token.
    cur: Tok,
    /// Byte offset where `cur` starts in the input.
    cur_offset: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Result<Self, ParseError> {
        let mut p = Parser {
            input,
            bytes: input.as_bytes(),
            pos: 0,
            cur: Tok::Eof,
            cur_offset: 0,
        };
        p.advance()?;
        Ok(p)
    }

    fn skip_ws(&mut self) {
        while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
    }

    /// Read the next token, updating `self.cur` and `self.cur_offset`.
    fn advance(&mut self) -> Result<(), ParseError> {
        self.skip_ws();
        let start = self.pos;
        if self.pos >= self.bytes.len() {
            self.cur = Tok::Eof;
            self.cur_offset = start;
            return Ok(());
        }

        let b = self.bytes[self.pos];
        let tok = match b {
            b'+' => { self.pos += 1; Tok::Plus }
            b'-' => { self.pos += 1; Tok::Minus }
            b'*' => { self.pos += 1; Tok::Star }
            b'/' => { self.pos += 1; Tok::Slash }
            b'(' => { self.pos += 1; Tok::LParen }
            b')' => { self.pos += 1; Tok::RParen }
            b'=' => {
                self.pos += 1;
                if self.pos < self.bytes.len() && self.bytes[self.pos] == b'=' {
                    self.pos += 1;
                    Tok::EqEq
                } else {
                    Tok::Eq
                }
            }
            b'!' => {
                self.pos += 1;
                if self.pos < self.bytes.len() && self.bytes[self.pos] == b'=' {
                    self.pos += 1;
                    Tok::NotEq
                } else {
                    return Err(ParseError {
                        offset: start,
                        kind: ParseErrorKind::UnexpectedChar('!'),
                    });
                }
            }
            b'<' => {
                self.pos += 1;
                if self.pos < self.bytes.len() && self.bytes[self.pos] == b'=' {
                    self.pos += 1;
                    Tok::Le
                } else {
                    Tok::Lt
                }
            }
            b'>' => {
                self.pos += 1;
                if self.pos < self.bytes.len() && self.bytes[self.pos] == b'=' {
                    self.pos += 1;
                    Tok::Ge
                } else {
                    Tok::Gt
                }
            }
            b'0'..=b'9' => {
                let mut end = self.pos;
                while end < self.bytes.len() && self.bytes[end].is_ascii_digit() {
                    end += 1;
                }
                let s = &self.input[self.pos..end];
                let n: i64 = s.parse().map_err(|_| ParseError {
                    offset: start,
                    kind: ParseErrorKind::IntegerOverflow,
                })?;
                self.pos = end;
                Tok::Int(n)
            }
            c if (c >= b'a' && c <= b'z') || c == b'_' => {
                let mut end = self.pos;
                while end < self.bytes.len() {
                    let c = self.bytes[end];
                    let is_word = (c >= b'a' && c <= b'z')
                        || (c >= b'0' && c <= b'9')
                        || c == b'_';
                    if is_word {
                        end += 1;
                    } else {
                        break;
                    }
                }
                let s = &self.input[self.pos..end];
                self.pos = end;
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
            _ => {
                let c = self.input[self.pos..].chars().next().unwrap_or('\0');
                return Err(ParseError {
                    offset: start,
                    kind: ParseErrorKind::UnexpectedChar(c),
                });
            }
        };
        self.cur = tok;
        self.cur_offset = start;
        Ok(())
    }

    fn err(&self, kind: ParseErrorKind) -> ParseError {
        ParseError { offset: self.cur_offset, kind }
    }

    fn expect(&mut self, expected: &Tok, label: &'static str) -> Result<(), ParseError> {
        if &self.cur == expected {
            self.advance()
        } else {
            Err(self.err(ParseErrorKind::Expected(label)))
        }
    }

    // ----- Grammar -----------------------------------------------------------

    fn parse_program(&mut self) -> Result<Expr, ParseError> {
        let e = self.parse_expr()?;
        if self.cur != Tok::Eof {
            return Err(self.err(ParseErrorKind::TrailingInput));
        }
        Ok(e)
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        match self.cur {
            Tok::Let => self.parse_let(),
            Tok::If => self.parse_if(),
            _ => self.parse_or(),
        }
    }

    fn parse_let(&mut self) -> Result<Expr, ParseError> {
        // current token is `let`
        self.advance()?; // consume `let`
        let name = match &self.cur {
            Tok::Ident(s) => s.clone(),
            Tok::Eof => return Err(self.err(ParseErrorKind::UnexpectedEof)),
            _ => return Err(self.err(ParseErrorKind::ExpectedIdentifier)),
        };
        self.advance()?; // consume identifier
        self.expect(&Tok::Eq, "\"=\"")?;
        let value = self.parse_expr()?;
        self.expect(&Tok::In, "\"in\"")?;
        let body = self.parse_expr()?;
        Ok(Expr::Let {
            name,
            value: Box::new(value),
            body: Box::new(body),
        })
    }

    fn parse_if(&mut self) -> Result<Expr, ParseError> {
        self.advance()?; // consume `if`
        let cond = self.parse_expr()?;
        self.expect(&Tok::Then, "\"then\"")?;
        let then_branch = self.parse_expr()?;
        self.expect(&Tok::Else, "\"else\"")?;
        let else_branch = self.parse_expr()?;
        Ok(Expr::If {
            cond: Box::new(cond),
            then_branch: Box::new(then_branch),
            else_branch: Box::new(else_branch),
        })
    }

    fn parse_or(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_and()?;
        while self.cur == Tok::Or {
            self.advance()?;
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
        while self.cur == Tok::And {
            self.advance()?;
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
        if self.cur == Tok::Not {
            self.advance()?;
            let inner = self.parse_not()?;
            Ok(Expr::UnOp {
                op: UnOp::Not,
                operand: Box::new(inner),
            })
        } else {
            self.parse_cmp()
        }
    }

    fn parse_cmp(&mut self) -> Result<Expr, ParseError> {
        let lhs = self.parse_add()?;
        let op = match self.cur {
            Tok::EqEq => BinOp::Eq,
            Tok::NotEq => BinOp::Ne,
            Tok::Lt => BinOp::Lt,
            Tok::Le => BinOp::Le,
            Tok::Gt => BinOp::Gt,
            Tok::Ge => BinOp::Ge,
            _ => return Ok(lhs),
        };
        self.advance()?;
        let rhs = self.parse_add()?;
        Ok(Expr::BinOp {
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        })
    }

    fn parse_add(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_mul()?;
        loop {
            let op = match self.cur {
                Tok::Plus => BinOp::Add,
                Tok::Minus => BinOp::Sub,
                _ => break,
            };
            self.advance()?;
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
            let op = match self.cur {
                Tok::Star => BinOp::Mul,
                Tok::Slash => BinOp::Div,
                _ => break,
            };
            self.advance()?;
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
        if self.cur == Tok::Minus {
            self.advance()?;
            let inner = self.parse_unary()?;
            Ok(Expr::UnOp {
                op: UnOp::Neg,
                operand: Box::new(inner),
            })
        } else {
            self.parse_atom()
        }
    }

    fn parse_atom(&mut self) -> Result<Expr, ParseError> {
        // Take ownership of the current token so we can match Tok::Int/Ident
        // without holding a borrow on `self`.
        let tok = std::mem::replace(&mut self.cur, Tok::Eof);
        match tok {
            Tok::Int(n) => {
                self.advance()?;
                Ok(Expr::Int(n))
            }
            Tok::True => {
                self.advance()?;
                Ok(Expr::Bool(true))
            }
            Tok::False => {
                self.advance()?;
                Ok(Expr::Bool(false))
            }
            Tok::Ident(name) => {
                self.advance()?;
                Ok(Expr::Var(name))
            }
            Tok::LParen => {
                self.advance()?;
                let e = self.parse_expr()?;
                if self.cur != Tok::RParen {
                    return Err(self.err(ParseErrorKind::Expected("\")\"")));
                }
                self.advance()?;
                Ok(e)
            }
            Tok::Eof => {
                // restore for error reporting
                self.cur = Tok::Eof;
                Err(ParseError {
                    offset: self.cur_offset,
                    kind: ParseErrorKind::UnexpectedEof,
                })
            }
            other => {
                let off = self.cur_offset;
                // restore current token so subsequent error reporting still
                // shows it (mostly cosmetic; we error out here anyway).
                self.cur = other;
                Err(ParseError {
                    offset: off,
                    kind: ParseErrorKind::UnexpectedToken,
                })
            }
        }
    }
}

// ===========================================================================
// Public parse entry point
// ===========================================================================

/// Parse an input string into an [`Expr`].
pub fn parse(input: &str) -> Result<Expr, ParseError> {
    let mut p = Parser::new(input)?;
    p.parse_program()
}

// ===========================================================================
// Evaluator
// ===========================================================================

/// Evaluate a parsed expression in the empty environment.
pub fn eval(expr: &Expr) -> Result<Value, EvalError> {
    let mut env: Vec<(String, Value)> = Vec::new();
    eval_in(expr, &mut env)
}

fn lookup(env: &[(String, Value)], name: &str) -> Option<Value> {
    for (n, v) in env.iter().rev() {
        if n == name {
            return Some(v.clone());
        }
    }
    None
}

fn eval_in(expr: &Expr, env: &mut Vec<(String, Value)>) -> Result<Value, EvalError> {
    match expr {
        Expr::Int(n) => Ok(Value::Int(*n)),
        Expr::Bool(b) => Ok(Value::Bool(*b)),

        Expr::Var(name) => lookup(env, name)
            .ok_or_else(|| EvalError::UnboundIdentifier(name.clone())),

        Expr::Let { name, value, body } => {
            let v = eval_in(value, env)?;
            env.push((name.clone(), v));
            let result = eval_in(body, env);
            env.pop();
            result
        }

        Expr::If { cond, then_branch, else_branch } => {
            match eval_in(cond, env)? {
                Value::Bool(true) => eval_in(then_branch, env),
                Value::Bool(false) => eval_in(else_branch, env),
                Value::Int(_) => Err(EvalError::TypeError(
                    "if condition must be a boolean".to_string(),
                )),
            }
        }

        Expr::UnOp { op, operand } => {
            let v = eval_in(operand, env)?;
            match (op, v) {
                (UnOp::Not, Value::Bool(b)) => Ok(Value::Bool(!b)),
                (UnOp::Not, Value::Int(_)) => Err(EvalError::TypeError(
                    "operator `not` requires a boolean".to_string(),
                )),
                (UnOp::Neg, Value::Int(n)) => Ok(Value::Int(n.wrapping_neg())),
                (UnOp::Neg, Value::Bool(_)) => Err(EvalError::TypeError(
                    "unary `-` requires an integer".to_string(),
                )),
            }
        }

        Expr::BinOp { op, lhs, rhs } => eval_binop(*op, lhs, rhs, env),
    }
}

fn eval_binop(
    op: BinOp,
    lhs: &Expr,
    rhs: &Expr,
    env: &mut Vec<(String, Value)>,
) -> Result<Value, EvalError> {
    // Short-circuit boolean operators.
    match op {
        BinOp::And => {
            let l = eval_in(lhs, env)?;
            let l = match l {
                Value::Bool(b) => b,
                Value::Int(_) => {
                    return Err(EvalError::TypeError(
                        "operator `and` requires booleans".to_string(),
                    ))
                }
            };
            if !l {
                return Ok(Value::Bool(false));
            }
            let r = eval_in(rhs, env)?;
            match r {
                Value::Bool(b) => Ok(Value::Bool(b)),
                Value::Int(_) => Err(EvalError::TypeError(
                    "operator `and` requires booleans".to_string(),
                )),
            }
        }
        BinOp::Or => {
            let l = eval_in(lhs, env)?;
            let l = match l {
                Value::Bool(b) => b,
                Value::Int(_) => {
                    return Err(EvalError::TypeError(
                        "operator `or` requires booleans".to_string(),
                    ))
                }
            };
            if l {
                return Ok(Value::Bool(true));
            }
            let r = eval_in(rhs, env)?;
            match r {
                Value::Bool(b) => Ok(Value::Bool(b)),
                Value::Int(_) => Err(EvalError::TypeError(
                    "operator `or` requires booleans".to_string(),
                )),
            }
        }

        // Strict (both sides evaluated) operators below.
        BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div => {
            let l = eval_in(lhs, env)?;
            let r = eval_in(rhs, env)?;
            let (a, b) = match (l, r) {
                (Value::Int(a), Value::Int(b)) => (a, b),
                _ => {
                    return Err(EvalError::TypeError(format!(
                        "arithmetic operator {} requires two integers",
                        binop_symbol(op)
                    )))
                }
            };
            match op {
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
            }
        }

        BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
            let l = eval_in(lhs, env)?;
            let r = eval_in(rhs, env)?;
            let (a, b) = match (l, r) {
                (Value::Int(a), Value::Int(b)) => (a, b),
                _ => {
                    return Err(EvalError::TypeError(format!(
                        "comparison operator {} requires two integers",
                        binop_symbol(op)
                    )))
                }
            };
            let result = match op {
                BinOp::Lt => a < b,
                BinOp::Le => a <= b,
                BinOp::Gt => a > b,
                BinOp::Ge => a >= b,
                _ => unreachable!(),
            };
            Ok(Value::Bool(result))
        }

        BinOp::Eq | BinOp::Ne => {
            let l = eval_in(lhs, env)?;
            let r = eval_in(rhs, env)?;
            let eq = match (l, r) {
                (Value::Int(a), Value::Int(b)) => a == b,
                (Value::Bool(a), Value::Bool(b)) => a == b,
                _ => {
                    return Err(EvalError::TypeError(format!(
                        "operator {} requires both operands to have the same type",
                        binop_symbol(op)
                    )))
                }
            };
            Ok(Value::Bool(if op == BinOp::Eq { eq } else { !eq }))
        }
    }
}

fn binop_symbol(op: BinOp) -> &'static str {
    match op {
        BinOp::Or => "or",
        BinOp::And => "and",
        BinOp::Eq => "==",
        BinOp::Ne => "!=",
        BinOp::Lt => "<",
        BinOp::Le => "<=",
        BinOp::Gt => ">",
        BinOp::Ge => ">=",
        BinOp::Add => "+",
        BinOp::Sub => "-",
        BinOp::Mul => "*",
        BinOp::Div => "/",
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn run(src: &str) -> Result<Value, String> {
        let e = parse(src).map_err(|e| e.to_string())?;
        eval(&e).map_err(|e| e.to_string())
    }

    #[test]
    fn arithmetic() {
        assert_eq!(run("1 + 2 * 3").unwrap(), Value::Int(7));
        assert_eq!(run("(1 + 2) * 3").unwrap(), Value::Int(9));
        assert_eq!(run("10 - 4 - 3").unwrap(), Value::Int(3)); // left-assoc
        assert_eq!(run("-3 + 5").unwrap(), Value::Int(2));
    }

    #[test]
    fn booleans_and_short_circuit() {
        assert_eq!(run("true and false").unwrap(), Value::Bool(false));
        assert_eq!(run("true or false").unwrap(), Value::Bool(true));
        assert_eq!(run("not true").unwrap(), Value::Bool(false));
        // short-circuit: rhs would be a type error but is never evaluated
        assert_eq!(run("false and (1 + true == 0)").unwrap(), Value::Bool(false));
        assert_eq!(run("true or (1 + true == 0)").unwrap(), Value::Bool(true));
    }

    #[test]
    fn comparisons() {
        assert_eq!(run("1 < 2").unwrap(), Value::Bool(true));
        assert_eq!(run("2 == 2").unwrap(), Value::Bool(true));
        assert_eq!(run("true == false").unwrap(), Value::Bool(false));
        assert!(run("1 == true").is_err());
        assert!(run("true < false").is_err());
    }

    #[test]
    fn let_and_if() {
        assert_eq!(
            run("let x = 1 + 2 in let y = x * 4 in y - x").unwrap(),
            Value::Int(9),
        );
        assert_eq!(
            run("if 1 < 2 then 10 else 20").unwrap(),
            Value::Int(10),
        );
        assert!(run("if 1 then 10 else 20").is_err()); // bad cond type
    }

    #[test]
    fn unbound() {
        match run("x + 1") {
            Err(s) => assert!(s.contains("unbound")),
            _ => panic!(),
        }
    }

    #[test]
    fn division_by_zero() {
        match run("10 / (5 - 5)") {
            Err(s) => assert!(s.contains("division by zero")),
            _ => panic!(),
        }
    }

    #[test]
    fn parse_error_offset() {
        let e = parse("1 + + 2").unwrap_err();
        // The second `+` is at byte offset 4.
        assert_eq!(e.offset, 4);
    }

    #[test]
    fn value_display() {
        assert_eq!(Value::Int(42).to_string(), "42");
        assert_eq!(Value::Int(-3).to_string(), "-3");
        assert_eq!(Value::Bool(true).to_string(), "true");
        assert_eq!(Value::Bool(false).to_string(), "false");
    }

    #[test]
    fn keywords_are_not_identifiers() {
        // `let` cannot be used as an identifier; this should be a parse error.
        assert!(parse("let let = 1 in let").is_err());
    }
}
