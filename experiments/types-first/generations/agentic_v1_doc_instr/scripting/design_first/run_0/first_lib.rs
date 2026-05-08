//! A tiny expression-language evaluator.

use std::fmt;

/// A parsed expression node.
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
    Binary {
        op: BinOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Unary {
        op: UnaryOp,
        operand: Box<Expr>,
    },
}

/// Binary operators.
#[derive(Clone, Copy)]
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

/// Unary operators.
#[derive(Clone, Copy)]
pub enum UnaryOp {
    Neg,
    Not,
}

/// A runtime value: either a 64-bit integer or a boolean.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Value {
    Int(i64),
    Bool(bool),
}

/// An error produced by [`parse`], carrying the byte offset at which parsing failed.
pub struct ParseError {
    /// The byte offset within the input where parsing failed.
    pub offset: usize,
    /// A short human-readable message.
    pub message: String,
}

/// An error produced by [`eval`].
pub enum EvalError {
    /// Operands had wrong types for the operation (e.g., `1 + true`).
    TypeError(String),
    /// Integer division by zero.
    DivisionByZero,
    /// Reference to an identifier that is not in scope.
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

/// Parse the input string into an [`Expr`].
///
/// Returns a [`ParseError`] (with the byte offset of the failure) if the input
/// does not match the grammar.
pub fn parse(_input: &str) -> Result<Expr, ParseError> {
    todo!()
}

/// Evaluate an expression to a [`Value`].
///
/// Returns an [`EvalError`] for type mismatches, division by zero, or unbound
/// identifier references.
pub fn eval(_expr: &Expr) -> Result<Value, EvalError> {
    todo!()
}
