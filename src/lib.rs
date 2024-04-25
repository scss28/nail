use std::{fmt::Display, str::FromStr};

pub mod prelude;

mod command;
mod database;
mod lexer;
mod parser;
mod token;

#[test]
fn main() {
    use database::{CommandRunOutput, Database};
    let src = r#"
        new Student
            name str,
            surname str,
            age int?;
        
        insert Student
            (name: "Gaming", surname: "buh", age: 66),
            (name: "Yuh", surname: "Yuh");

        get Student @Id, *;
    "#;

    let tokens = lexer::TokenIter::from(src.as_bytes());
    let mut commands = parser::CommandIter::new(tokens);
    let mut database = Database::new();
    while let Some(command) = commands.next() {
        let command = command.unwrap();
        match database.run_command(command).unwrap() {
            CommandRunOutput::RowsInserted { identifier, count } => {
                println!("Inserted {count} rows into table \"{identifier}\".");
            }
            CommandRunOutput::TableCreated { identifier } => {
                println!("Created \"{identifier}\" table.");
            }
            CommandRunOutput::Selection { rows, .. } => {
                for row in rows {
                    for value in row {
                        print!("{value} ");
                    }
                    println!();
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Ty {
    Str,
    Int,
    Float,
    Nil,
}

#[derive(Debug, Clone, Copy)]
pub struct NoSuchTypeError;
impl FromStr for Ty {
    type Err = NoSuchTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "str" => Ty::Str,
            "int" => Ty::Int,
            "float" => Ty::Float,
            _ => return Err(NoSuchTypeError),
        })
    }
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
