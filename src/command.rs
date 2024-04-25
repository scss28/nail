use std::str::FromStr;

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
        insertions: Vec<Vec<Insertion>>,
    },
    Get {
        identifier: String,
        selections: Vec<Selection>,
    },
}

#[derive(Debug, Clone)]
pub struct Insertion {
    pub identifier: String,
    pub expression: Expression,
}

#[derive(Debug, Clone)]
pub struct ColumnDefinition {
    pub identifier: String,
    pub optional: bool,
    pub ty: Ty,
}

#[derive(Debug, Clone)]
pub enum Expression {
    Literal(Value),
}

#[derive(Debug, Clone)]
pub enum Selection {
    Column(String),
    RowAttribute(RowAttribute),
    All,
}

#[derive(Debug, Clone)]
pub enum RowAttribute {
    Id,
}

#[derive(Debug, Clone, Copy)]
pub struct NoSuchRowAttributeError;
impl FromStr for RowAttribute {
    type Err = NoSuchRowAttributeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "Id" => RowAttribute::Id,
            _ => return Err(NoSuchRowAttributeError),
        })
    }
}
