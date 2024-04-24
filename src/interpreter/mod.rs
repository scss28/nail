use std::{
    alloc, collections::HashMap, fmt::Display, process::id, ptr::NonNull, string::ParseError,
};

use self::command::{Command, Expression, Selection, Ty};

mod command;
mod lexer;
mod parser;
mod token;
mod util;

pub fn run(src: impl AsRef<[u8]>) -> Result<(), RunError> {
    let src = src.as_ref();
    let tokens = lexer::TokenIter::from(src);
    #[cfg(debug_assertions)]
    {
        println!("TOKENS: ");
        let mut tokens = tokens.clone();
        while let Some(token) = tokens.next() {
            let pos = tokens.src_pos();
            let (line, char) = util::src_position(src, pos.start);
            println!(
                "{:?} {:?} ({}:{})",
                token,
                std::str::from_utf8(&src[pos]).unwrap(),
                line,
                char
            )
        }
        println!();
    }

    let mut db = HashMap::new();
    let mut commands = parser::CommandIter::new(tokens);
    while let Some(command) = commands.next() {
        let command = match command {
            Ok(command) => command,
            Err(err) => {
                let pos = commands.src_pos();
                let (line, char) = util::src_position(src, pos.start);
                println!(
                    "{err:?} {:?} ({}:{})",
                    std::str::from_utf8(&src[pos]).unwrap(),
                    line,
                    char
                );

                continue;
            }
        };

        match command {
            Command::New {
                identifier,
                columns,
            } => {
                db.insert(
                    identifier,
                    columns
                        .into_iter()
                        .map(|(ident, (optional, ty))| {
                            (
                                ident,
                                Column {
                                    ty,
                                    optional,
                                    values: Vec::new(),
                                },
                            )
                        })
                        .collect::<HashMap<_, _>>(),
                );
            }
            Command::Insert {
                identifier,
                inserts,
            } => {
                let Some(table) = db.get_mut(&identifier) else {
                    println!("table \"{identifier}\" doesn't exist");
                    continue;
                };

                for mut insert in inserts {
                    for (identifier, column) in table.iter_mut() {
                        let Some(expression) = insert.remove(identifier) else {
                            if !column.optional {
                                println!("column \"{identifier}\" isn't optional");
                            }

                            column.values.push(Value::Nil);
                            continue;
                        };

                        match expression {
                            Expression::Literal(literal) => match literal {
                                Value::Str(str) => {
                                    if !matches!(column.ty, Ty::Str) {
                                        println!("column \"{identifier}\" isn't of type str");
                                    }

                                    column.values.push(Value::Str(str));
                                }
                                Value::Nil => {
                                    if !column.optional {
                                        println!("column \"{identifier}\" isn't optional");
                                    }

                                    column.values.push(Value::Nil);
                                }
                            },
                        }
                    }
                }
            }
            Command::Get {
                identifier,
                selections,
            } => {
                let Some(table) = db.get(&identifier) else {
                    println!("table \"{identifier}\" doesn't exist");
                    continue;
                };

                if table.is_empty() {
                    continue;
                }

                let values = table.into_iter().collect::<Vec<_>>();
                for i in 0..values[0].1.values.len() {
                    let mut row = HashMap::new();
                    for (identifier, Column { values, .. }) in &values {
                        row.insert(*identifier, &values[i]);
                    }

                    for selection in &selections {
                        match selection {
                            Selection::Column(identifier) => {
                                let Some(value) = row.get(identifier) else {
                                    print!("<not a column>, ");
                                    continue;
                                };

                                print!("{value}, ");
                            }
                            Selection::RowAttribute(attribute) => match &**attribute {
                                "id" => print!("{i}, "),
                                _ => print!("<not an attribute>, "),
                            },
                            Selection::All => {
                                for (_, value) in &row {
                                    print!("{value}, ");
                                }
                            }
                        }
                    }

                    println!();
                }
            }
        }
    }

    Ok(())
}

pub struct Column {
    ty: Ty,
    optional: bool,
    values: Vec<Value>,
}

#[derive(Debug, Clone)]
pub enum Value {
    Str(Box<str>),
    Nil,
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Str(str) => write!(f, "\"{str}\""),
            Value::Nil => write!(f, "nil"),
        }
    }
}

// pub struct Table {
//     headers: Box<[(Box<str>, Ty)]>,
//     rows: Vec<NonNull<u8>>,
// }

// impl Table {
//     pub fn new(headers: impl IntoIterator<Item = (Box<str>, Ty)>) -> Self {
//         Self {
//             headers: headers.into_iter().collect(),
//             rows: Vec::new(),
//         }
//     }

//     pub fn push(&mut self, expressions: impl IntoIterator<Item = (Box<str>, Expression)>) {
//         for (identifier, expression) in expressions {
//             let Some((index, ty)) =
//                 self.headers
//                     .iter()
//                     .enumerate()
//                     .find_map(|(i, (row_identifier, ty))| {
//                         if *row_identifier == identifier {
//                             Some((i, ty))
//                         } else {
//                             None
//                         }
//                     })
//             else {
//                 continue;
//             };

//             let (layout, offset) = self.layout(index).unwrap();
//             let ptr = unsafe { alloc::alloc(layout) };
//             match expression {
//                 Expression::Literal(literal) => match literal {
//                     Literal::Str(str) => {
//                         if *ty != Ty::Str {
//                             continue;
//                         }

//                         unsafe { *(ptr.wrapping_add(offset) as *mut Box<str>) = str };
//                     }
//                 },
//             }

//             self.rows.push(unsafe { NonNull::new_unchecked(ptr) });
//         }
//     }

//     fn display() {

//     }

//     fn layout(&self, index: usize) -> Option<(alloc::Layout, usize)> {
//         let Some((_, ty)) = self.headers.first() else {
//             return None;
//         };

//         let mut layout = ty.layout();
//         let mut offset = 0;
//         for (i, (_, ty)) in self.headers[1..].iter().enumerate() {
//             let ty_layout = ty.layout();
//             let ty_layout_size = ty_layout.size();
//             let extend_offset;
//             (layout, extend_offset) = layout.extend(ty.layout()).unwrap();

//             if i < index {
//                 offset += ty_layout_size + extend_offset;
//             }
//         }

//         Some((layout, offset))
//     }
// }

// impl Drop for Table {
//     fn drop(&mut self) {
//         let Some((layout, _)) = self.layout(0) else {
//             return;
//         };

//         for ptr in &self.rows {
//             unsafe { alloc::dealloc(ptr.as_ptr(), layout) };
//         }
//     }
// }

#[derive(Debug, Clone)]
pub enum RunError {
    ParseError(ParseError),
}

impl From<ParseError> for RunError {
    fn from(value: ParseError) -> Self {
        Self::ParseError(value)
    }
}
