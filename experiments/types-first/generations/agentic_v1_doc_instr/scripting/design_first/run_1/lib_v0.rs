//! Tiny expression-language evaluator.
//!
//! See module-level grammar in the crate documentation. Public surface:
//! [`parse`], [`eval`], [`Expr`], [`Value`], [`ParseError`], [`EvalError`].

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
    /// Integer division (or modulo) by zero.
    DivisionByZero,
    /// Reference to an identifier that has not been bound by an enclosing `let`.
    UnboundIdentifier(String),
}

impl fmt::Display for Value {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl fmt::Debug for ParseError {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl fmt::Display for EvalError {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl fmt::Debug for EvalError {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

/// Parse `input` into an [`Expr`].
pub fn parse(_input: &str) -> Result<Expr, ParseError> {
    todo!()
}

/// Evaluate `expr` to a [`Value`].
pub fn eval(_expr: &Expr) -> Result<Value, EvalError> {
    todo!()
}
