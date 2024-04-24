use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Command {
    New {
        identifier: Box<str>,
        columns: HashMap<Box<str>, Ty>,
    },
    Insert {
        identifier: Box<str>,
        values: Vec<Expression>,
    },
    Get {
        identifier: Box<str>,
        selection: Vec<Selection>,
    },
}

#[derive(Debug, Clone)]
pub enum Expression {
    Literal(),
}

#[derive(Debug, Clone)]
pub enum Ty {
    Str,
}

#[derive(Debug, Clone)]
pub enum Selection {
    Column(Box<str>),
    Id,
    All,
}
