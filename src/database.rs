use std::collections::HashMap;

use crate::{
    command::{ColumnDefinition, Command, Expression, Insertion, RowAttribute, Selection},
    Ty, Value,
};

pub struct Database {
    tables: HashMap<String, Table>,
}

#[derive(Debug, Clone)]
pub enum CommandRunOutput {
    RowsInserted {
        identifier: String,
        count: usize,
    },
    TableCreated {
        identifier: String,
    },
    Selection {
        headers: Vec<String>,
        rows: Vec<Vec<Value>>,
    },
}

#[derive(Debug, Clone)]
pub enum CommandRunError {
    NoSuchTable(String),
    NoSuchColumn(String),
    IncorrectTy { column: String },
    NonOptionalColumn { column: String },
}

impl Database {
    pub fn new() -> Self {
        Self {
            tables: HashMap::new(),
        }
    }

    pub fn run_command(&mut self, command: Command) -> Result<CommandRunOutput, CommandRunError> {
        match command {
            Command::New {
                identifier,
                definitions,
            } => {
                self.tables.insert(
                    identifier.clone(),
                    Table {
                        columns: definitions
                            .into_iter()
                            .map(
                                |ColumnDefinition {
                                     identifier,
                                     optional,
                                     ty,
                                 }| Column {
                                    identifier,
                                    ty,
                                    optional,
                                    values: Vec::new(),
                                },
                            )
                            .collect(),
                    },
                );

                Ok(CommandRunOutput::TableCreated { identifier })
            }
            Command::Insert {
                identifier,
                insertions,
            } => {
                let insertions = insertions
                    .into_iter()
                    .map(|insertions| {
                        insertions
                            .into_iter()
                            .map(
                                |Insertion {
                                     identifier,
                                     expression,
                                 }| {
                                    (identifier, evaluate_expression(expression))
                                },
                            )
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>();

                let Some(table) = self.tables.get_mut(&identifier) else {
                    return Err(CommandRunError::NoSuchTable(identifier));
                };

                // Validate insertions.
                for insertion in &insertions {
                    let mut columns = table
                        .columns
                        .iter()
                        .map(
                            |Column {
                                 identifier,
                                 ty,
                                 optional,
                                 values,
                             }| (identifier, (ty, optional, values)),
                        )
                        .collect::<HashMap<_, _>>();

                    for (identifier, value) in insertion {
                        let Some((ty, _, _)) = columns.remove(identifier) else {
                            return Err(CommandRunError::NoSuchColumn(identifier.clone()));
                        };

                        if value.ty() != *ty {
                            return Err(CommandRunError::IncorrectTy {
                                column: identifier.clone(),
                            });
                        }
                    }

                    for (identifier, (_, optional, _)) in columns {
                        if !*optional {
                            return Err(CommandRunError::NonOptionalColumn {
                                column: identifier.clone(),
                            });
                        }
                    }
                }

                let count = insertions.len();

                // Insert...
                for insertion in insertions {
                    let mut columns = table
                        .columns
                        .iter_mut()
                        .map(
                            |Column {
                                 identifier,
                                 ty,
                                 optional,
                                 values,
                             }| (identifier, (ty, optional, values)),
                        )
                        .collect::<HashMap<_, _>>();

                    for (identifier, value) in insertion {
                        // This should get validated before. That's why here it should be unreachable.
                        let Some((ty, _, values)) = columns.remove(&identifier) else {
                            unreachable!();
                        };

                        // Same thing here.
                        if value.ty() != *ty {
                            unreachable!();
                        }

                        values.push(value);
                    }

                    for (_, (_, optional, values)) in columns {
                        // Same thing here.
                        if !*optional {
                            unreachable!();
                        }

                        values.push(Value::Nil);
                    }
                }

                Ok(CommandRunOutput::RowsInserted { identifier, count })
            }
            Command::Get {
                identifier,
                selections,
            } => {
                let Some(table) = self.tables.get(&identifier) else {
                    return Err(CommandRunError::NoSuchTable(identifier));
                };

                let mut rows = Vec::new();
                for i in 0..table.len() {
                    let mut row = Vec::new();
                    for selection in &selections {
                        match selection {
                            Selection::Column(get_identifier) => {
                                let Some(value) = table.columns.iter().find_map(
                                    |Column {
                                         identifier, values, ..
                                     }| {
                                        if identifier != get_identifier {
                                            return None;
                                        }

                                        Some(&values[i])
                                    },
                                ) else {
                                    return Err(CommandRunError::NoSuchColumn(
                                        get_identifier.clone(),
                                    ));
                                };

                                row.push(value.clone());
                            }
                            Selection::RowAttribute(attribute) => match attribute {
                                RowAttribute::Id => {
                                    row.push(Value::Int(i as i32));
                                }
                            },
                            Selection::All => {
                                for Column { values, .. } in &table.columns {
                                    row.push(values[i].clone());
                                }
                            }
                        }
                    }

                    rows.push(row);
                }

                Ok(CommandRunOutput::Selection {
                    headers: table
                        .columns
                        .iter()
                        .map(|Column { identifier, .. }| identifier.clone())
                        .collect(),
                    rows,
                })
            }
        }
    }
}

fn evaluate_expression(expression: Expression) -> Value {
    match expression {
        Expression::Literal(value) => value,
    }
}

pub struct Table {
    columns: Vec<Column>,
}

impl Table {
    pub fn len(&self) -> usize {
        let Some(first) = self.columns.first() else {
            return 0;
        };

        first.values.len()
    }
}

pub struct Column {
    identifier: String,
    ty: Ty,
    optional: bool,
    values: Vec<Value>,
}
