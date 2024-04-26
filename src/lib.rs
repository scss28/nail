use std::fmt::Display;

pub mod prelude;

mod command;
mod database;
mod lexer;
mod parser;
mod token;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Ty {
    Str,
    Int,
    Float,
    Bool,
    Nil,
}

impl Display for Ty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Ty::Str => "str",
                Ty::Int => "int",
                Ty::Float => "float",
                Ty::Nil => "nil",
                Ty::Bool => "bool",
            }
        )
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Str(String),
    Int(i32),
    Float(f32),
    Bool(bool),
    Nil,
}

impl Value {
    pub fn ty(&self) -> Ty {
        match self {
            Value::Str(_) => Ty::Str,
            Value::Nil => Ty::Nil,
            Value::Int(_) => Ty::Int,
            Value::Float(_) => Ty::Float,
            Value::Bool(_) => Ty::Bool,
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
            Value::Bool(bool) => write!(f, "{bool}"),
        }
    }
}
