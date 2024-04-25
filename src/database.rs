use parse_display_derive::Display;
use terrors::OneOf;

use crate::{
    command::{ColumnDefinition, Command, Expression, RowAttribute, Selection},
    Ty, Value,
};
use std::{collections::HashMap, fmt::Display};

#[derive(Debug, Clone)]
pub enum CommandRunOutput {
    RowsInserted {
        identifier: String,
        count: usize,
        errs: Vec<OneOf<(InsertionError, NoSuchColumnError)>>,
    },
    TableCreated {
        identifier: String,
    },
    Selection {
        table: Table,
    },
}

impl Display for CommandRunOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandRunOutput::RowsInserted {
                identifier,
                count,
                errs,
            } => {
                for err in errs {
                    write!(f, "Insertion failed: {err}")?;
                }

                write!(f, "\nInserted {count} rows into table \"{identifier}\".")?;
                Ok(())
            }
            CommandRunOutput::TableCreated { identifier } => {
                write!(f, "Table \"{identifier}\" created.")
            }
            CommandRunOutput::Selection { table } => write!(f, "{table}"),
        }
    }
}

#[derive(Debug, Display, Clone)]
#[display("Table \"{0}\" does not exist.")]
pub struct NoSuchTableError(String);

#[derive(Debug, Display, Clone)]
#[display("Column \"{0}\" does not exist.")]
pub struct NoSuchColumnError(String);

#[derive(Debug, Display, Clone)]
pub enum InsertionError {
    #[display("Column \"{column}\" expects a type: {ty}.")]
    IncorrectTy { column: String, ty: Ty },
    #[display("Column \"{column}\" is not optional.")]
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

    pub fn run_command(
        &mut self,
        command: Command,
    ) -> Result<CommandRunOutput, OneOf<(NoSuchTableError, InsertionError, NoSuchColumnError)>>
    {
        match command {
            Command::New {
                identifier,
                definitions,
            } => {
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
            Command::Insert {
                identifier,
                insertions,
            } => {
                let Some(table) = self.tables.get_mut(&identifier) else {
                    return Err(OneOf::new(NoSuchTableError(identifier)));
                };

                let mut errs = Vec::new();
                let mut count = 0;
                for insertion in insertions {
                    let Err(err) = table.insert(insertion) else {
                        count += 1;
                        continue;
                    };

                    errs.push(err);
                }

                Ok(CommandRunOutput::RowsInserted {
                    identifier,
                    count,
                    errs,
                })
            }
            Command::Get {
                identifier,
                selections,
            } => {
                let Some(table) = self.tables.get(&identifier) else {
                    return Err(OneOf::new(NoSuchTableError(identifier)));
                };

                let table = table.get(selections).map_err(OneOf::broaden)?;
                Ok(CommandRunOutput::Selection { table })
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Table {
    columns: Vec<Column>,
}

impl Table {
    pub fn new() -> Self {
        Self {
            columns: Vec::new(),
        }
    }

    pub fn width(&self) -> usize {
        self.columns.len()
    }

    pub fn height(&self) -> usize {
        let Some(first) = self.columns.first() else {
            return 0;
        };

        first.values.len()
    }

    pub fn get(&self, selections: Vec<Selection>) -> Result<Table, OneOf<(NoSuchColumnError,)>> {
        let mut columns = Vec::new();
        for selection in selections {
            match selection {
                Selection::Column { column, identifier } => {
                    let Some(mut column) = self
                        .columns
                        .iter()
                        .find(
                            |Column {
                                 identifier: column_identifier,
                                 ..
                             }| *column_identifier == column,
                        )
                        .cloned()
                    else {
                        return Err(NoSuchColumnError(column.clone()).into());
                    };

                    if let Some(identifier) = identifier {
                        column.identifier = identifier;
                    }

                    columns.push(column.clone());
                }
                Selection::RowAttribute {
                    attribute,
                    identifier,
                } => match attribute {
                    RowAttribute::Id => {
                        columns.push(Column {
                            identifier: identifier.unwrap_or("Id".into()),
                            optional: false,
                            ty: Ty::Int,
                            values: (0..self.height() as i32).map(|i| Value::Int(i)).collect(),
                        });
                    }
                },
                Selection::All => {
                    columns.extend(self.columns.clone());
                }
            }
        }

        Ok(Table { columns })
    }

    pub fn insert(
        &mut self,
        insertion: HashMap<String, Expression>,
    ) -> Result<(), OneOf<(InsertionError, NoSuchColumnError)>> {
        let insertion = insertion
            .into_iter()
            .map(|(identifier, expression)| {
                (
                    identifier,
                    match expression {
                        Expression::Value(value) => value,
                    },
                )
            })
            .collect::<HashMap<_, _>>();

        // Insertion validation.
        // ---------------------@
        let mut columns = self
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

        for (identifier, value) in &insertion {
            let Some((ty, optional, _)) = columns.remove(identifier) else {
                return Err(OneOf::new(NoSuchColumnError(identifier.clone())));
            };

            if value.ty() == Ty::Nil && *optional {
                continue;
            }

            if value.ty() != *ty {
                return Err(OneOf::new(InsertionError::IncorrectTy {
                    column: identifier.clone(),
                    ty: *ty,
                }));
            }
        }

        for (identifier, (_, optional, _)) in columns {
            if !*optional {
                return Err(OneOf::new(InsertionError::NonOptionalColumn {
                    column: identifier.clone(),
                }));
            }
        }
        // ---------------------@
        let mut columns = self
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

        Ok(())
    }
}

impl Display for Table {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut max_widths = Vec::with_capacity(self.columns.len());
        for Column {
            identifier, values, ..
        } in &self.columns
        {
            let max_width = values.into_iter().fold(identifier.len(), |acc, value| {
                acc.max(value.to_string().len())
            });
            max_widths.push(max_width);
        }

        const PADDING: usize = 1;

        write!(f, "|")?;
        for (Column { identifier, .. }, max_width) in self.columns.iter().zip(&max_widths) {
            for _ in 0..PADDING {
                write!(f, " ")?;
            }

            write!(f, "{identifier}")?;
            for _ in 0..max_width - identifier.len() + PADDING {
                write!(f, " ")?;
            }

            write!(f, "|")?;
        }
        writeln!(f)?;

        write!(f, " ")?;
        for max_width in &max_widths {
            for _ in 0..max_width + PADDING * 2 {
                write!(f, "-")?;
            }

            write!(f, " ")?;
        }
        writeln!(f)?;

        for j in 0..self.height() {
            write!(f, "|")?;
            for i in 0..self.width() {
                let value_str = self.columns[i].values[j].to_string();
                for _ in 0..PADDING {
                    write!(f, " ")?;
                }

                write!(f, "{value_str}")?;
                for _ in 0..max_widths[i] - value_str.len() + PADDING {
                    write!(f, " ")?;
                }

                write!(f, "|")?;
            }
            writeln!(f)?;
        }

        write!(f, " ")?;
        for max_width in &max_widths {
            for _ in 0..max_width + PADDING * 2 {
                write!(f, "-")?;
            }

            write!(f, " ")?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Column {
    identifier: String,
    ty: Ty,
    optional: bool,
    values: Vec<Value>,
}
