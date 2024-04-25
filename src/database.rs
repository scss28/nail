use crate::{
    command::{ColumnDefinition, Command, Expression, RowAttribute, Selection},
    Ty, Value,
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum CommandRunOutput {
    RowsInserted { identifier: String, count: usize },
    TableCreated { identifier: String },
    Selection { rows: Vec<Vec<Value>> },
}

#[derive(Debug, Clone)]
pub enum CommandRunError {
    NoSuchTable(String),
    NoSuchColumn(String),
    IncorrectTy { column: String },
    NonOptionalColumn { column: String },
}

pub struct Database {
    tables: HashMap<String, Table>,
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
            } => self.run_new(identifier, definitions),
            Command::Insert {
                identifier,
                insertions,
            } => self.run_insert(identifier, insertions),
            Command::Get {
                identifier,
                selections,
            } => self.run_get(identifier, selections),
        }
    }

    pub fn run_new(
        &mut self,
        identifier: String,
        definitions: Vec<ColumnDefinition>,
    ) -> Result<CommandRunOutput, CommandRunError> {
        let mut columns = Vec::new();
        for ColumnDefinition {
            identifier,
            optional,
            ty,
        } in definitions
        {
            columns.push(Column {
                identifier,
                ty,
                optional,
                values: Vec::new(),
            });
        }

        self.tables.insert(identifier.clone(), Table { columns });
        Ok(CommandRunOutput::TableCreated { identifier })
    }

    pub fn run_insert(
        &mut self,
        identifier: String,
        insertions: Vec<HashMap<String, Expression>>,
    ) -> Result<CommandRunOutput, CommandRunError> {
        let mut evaluated_insertions = Vec::new();
        for insertion in insertions {
            evaluated_insertions.push(
                insertion
                    .into_iter()
                    .map(|(identifier, expression)| {
                        (
                            identifier,
                            match expression {
                                Expression::Value(value) => value,
                            },
                        )
                    })
                    .collect::<HashMap<_, _>>(),
            );
        }

        let Some(table) = self.tables.get_mut(&identifier) else {
            return Err(CommandRunError::NoSuchTable(identifier));
        };

        // Validate insertions.
        for insertion in &evaluated_insertions {
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
                let Some((ty, optional, _)) = columns.remove(identifier) else {
                    return Err(CommandRunError::NoSuchColumn(identifier.clone()));
                };

                if value.ty() == Ty::Nil && *optional {
                    continue;
                }

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

        let count = evaluated_insertions.len();
        for insertion in evaluated_insertions {
            let mut columns = table
                .columns
                .iter_mut()
                .map(
                    |Column {
                         identifier, values, ..
                     }| (identifier, values),
                )
                .collect::<HashMap<_, _>>();

            for (identifier, value) in insertion {
                // This should get validated before. That's why here it should be unreachable.
                let Some(values) = columns.remove(&identifier) else {
                    unreachable!();
                };

                values.push(value);
            }

            for values in columns.values_mut() {
                values.push(Value::Nil);
            }
        }

        Ok(CommandRunOutput::RowsInserted { identifier, count })
    }

    pub fn run_get(
        &self,
        identifier: String,
        selections: Vec<Selection>,
    ) -> Result<CommandRunOutput, CommandRunError> {
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
                            return Err(CommandRunError::NoSuchColumn(get_identifier.clone()));
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

        Ok(CommandRunOutput::Selection { rows })
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
