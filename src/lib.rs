use std::fmt::Display;

pub mod prelude;

mod command;
mod database;
mod lexer;
mod parser;
mod token;

#[derive(Debug, Clone, PartialEq)]
pub enum Ty {
    Str,
    Int,
    Float,
    Nil,
}

#[derive(Debug, Clone)]
pub enum Value {
    Str(String),
    Int(i32),
    Float(f32),
    Nil,
}

impl Value {
    pub fn ty(&self) -> Ty {
        match self {
            Value::Str(_) => Ty::Str,
            Value::Nil => Ty::Nil,
            Value::Int(_) => Ty::Int,
            Value::Float(_) => Ty::Float,
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Str(str) => write!(f, "\"{str}\""),
            Value::Nil => write!(f, "nil"),
            Value::Int(int) => write!(f, "{int}"),
            Value::Float(float) => write!(f, "{float}"),
        }
    }
}
