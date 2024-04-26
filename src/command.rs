use std::collections::HashMap;

use crate::Ty;

use super::Value;

#[derive(Debug, Clone)]
pub enum Command {
    New {
        identifier: String,
        definitions: Vec<ColumnDefinition>,
    },
    Insert {
        identifier: String,
        insertions: Vec<HashMap<String, Expression>>,
    },
    Get {
        identifier: String,
        selections: Vec<Selection>,
        filter: Option<Expression>,
    },
}

#[derive(Debug, Clone)]
pub struct ColumnDefinition {
    pub identifier: String,
    pub optional: bool,
    pub ty: Ty,
}

#[derive(Debug, Clone)]
pub enum Selection {
    Column {
        column: String,
        identifier: Option<String>,
    },
    All,
}

#[derive(Debug, Clone, Copy)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Less,
    LessEq,
    More,
    MoreEq,
}

#[derive(Debug, Clone)]
pub enum Expression {
    Value(Value),
    Identifier(String),
    Enclosed(Box<Expression>),
    Operation {
        lhs: Box<Expression>,
        operator: Operator,
        rhs: Box<Expression>,
    },
}

impl Expression {}
