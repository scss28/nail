use super::Value;
use std::{alloc, collections::HashMap};

#[derive(Debug, Clone)]
pub enum Command {
    New {
        identifier: Box<str>,
        columns: HashMap<Box<str>, (bool, Ty)>,
    },
    Insert {
        identifier: Box<str>,
        inserts: Vec<HashMap<Box<str>, Expression>>,
    },
    Get {
        identifier: Box<str>,
        selections: Vec<Selection>,
    },
}

#[derive(Debug, Clone)]
pub enum Expression {
    Literal(Value),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Ty {
    Str,
    Nil,
}

#[derive(Debug, Clone)]
pub enum Selection {
    Column(Box<str>),
    RowAttribute(Box<str>),
    All,
}
