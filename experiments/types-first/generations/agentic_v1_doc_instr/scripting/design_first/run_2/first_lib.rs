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
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
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
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl fmt::Debug for ParseError {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
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
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl fmt::Debug for EvalError {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

/// Parse the input source into an `Expr`.
pub fn parse(_input: &str) -> Result<Expr, ParseError> {
    todo!()
}

/// Evaluate an expression to a `Value`.
pub fn eval(_expr: &Expr) -> Result<Value, EvalError> {
    todo!()
}
