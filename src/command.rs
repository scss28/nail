use std::collections::HashMap;

use crate::{token::Token, Ty};

use super::Value;

#[derive(Debug, Clone)]
pub enum Command {
    New {
        identifier: String,
        definitions: Vec<ColumnDefinition>,
    },
    Insert {
        identifier: String,
        insertions: Vec<HashMap<String, Value>>,
    },
    Get {
        identifier: String,
        selections: Vec<Selection>,
        filter: Option<Expression>,
    },
    Remove {
        identifier: String,
        expression: Expression,
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
    Identifier { identifier: String },
    All,
}

crate::operator! {
    #[0]
    And | Or,
    #[1]
    Eq
    | Less
    | LessEq
    | More
    | MoreEq,
    #[2]
    Add | Sub,
    #[3]
    Mul | Div
}

#[derive(Debug, Clone, Copy)]
pub struct NoSuchOperatorError;
impl TryFrom<&Token> for Operator {
    type Error = NoSuchOperatorError;

    fn try_from(value: &Token) -> Result<Self, Self::Error> {
        Ok(match value {
            Token::Plus => Operator::Add,
            Token::Minus => Operator::Sub,
            Token::Star => Operator::Mul,
            Token::Slash => Operator::Div,
            Token::DoubleEq => Operator::Eq,
            Token::Less => Operator::Less,
            Token::LessEq => Operator::LessEq,
            Token::More => Operator::More,
            Token::MoreEq => Operator::MoreEq,
            Token::DoubleAmpersand => Operator::And,
            Token::DoublePipe => Operator::Or,
            _ => return Err(NoSuchOperatorError),
        })
    }
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

impl Expression {
    pub fn extended(self, operator: Operator, rhs: Expression) -> Self {
        match self {
            Expression::Operation {
                lhs,
                operator: self_operator,
                rhs: self_right,
            } if operator.precedence() > self_operator.precedence() => Expression::Operation {
                lhs,
                operator: self_operator,
                rhs: Box::new(self_right.extended(operator, rhs)),
            },
            _ => Expression::Operation {
                lhs: Box::new(self),
                operator,
                rhs: Box::new(rhs),
            },
        }
    }
}
