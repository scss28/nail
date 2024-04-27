use parse_display_derive::Display;
use terrors::OneOf;

use crate::{
    command::{ColumnDefinition, Command, Expression, Operator, Selection},
    Ty, Value,
};
use std::{collections::HashMap, fmt::Display};

#[derive(Debug, Clone)]
pub enum CommandRunOutput {
    RowsInserted {
        identifier: String,
        count: usize,
        errs: Vec<InsertError>,
    },
    TableCreated {
        identifier: String,
    },
    Selection {
        table: Table,
    },
    Removed {
        count: usize,
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
                    writeln!(f, "Insertion failed: {err}")?;
                }

                write!(
                    f,
                    "Inserted {count} {} into table \"{identifier}\".",
                    if *count == 1 { "row" } else { "rows" }
                )?;
                Ok(())
            }
            CommandRunOutput::TableCreated { identifier } => {
                write!(f, "Table \"{identifier}\" created.")
            }
            CommandRunOutput::Selection { table } => write!(f, "{table}"),
            CommandRunOutput::Removed { count } => {
                write!(
                    f,
                    "Removed {count} {}.",
                    if *count == 1 { "row" } else { "rows" }
                )
            }
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

const ID_IDENTIFIER: &str = "Id";

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
    ) -> Result<
        CommandRunOutput,
        OneOf<(
            NoSuchTableError,
            InsertionError,
            NoSuchColumnError,
            ExpectedBoolError,
            CannotEvaluateError,
            IdInsertError,
        )>,
    > {
        match command {
            Command::New {
                identifier,
                definitions,
            } => {
                let mut columns = vec![Column {
                    identifier: ID_IDENTIFIER.to_owned(),
                    ty: Ty::Int,
                    optional: false,
                    values: Vec::new(),
                }];

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
                filter,
            } => {
                let Some(table) = self.tables.get(&identifier) else {
                    return Err(OneOf::new(NoSuchTableError(identifier)));
                };

                let table = table.get(selections, filter).map_err(OneOf::broaden)?;
                Ok(CommandRunOutput::Selection { table })
            }
            Command::Remove {
                identifier,
                expression,
            } => {
                let Some(table) = self.tables.get_mut(&identifier) else {
                    return Err(OneOf::new(NoSuchTableError(identifier)));
                };

                let count = table.remove(expression).map_err(OneOf::broaden)?;
                Ok(CommandRunOutput::Removed { count })
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct CannotEvaluateError {
    pub lhs: Value,
    pub operator: Operator,
    pub rhs: Value,
}

impl Display for CannotEvaluateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Cannot ")?;
        match self.operator {
            Operator::And => {
                write!(f, "\"and\"")?;
            }
            Operator::Or => {
                write!(f, "\"or\"")?;
            }
            Operator::Eq
            | Operator::Less
            | Operator::LessEq
            | Operator::More
            | Operator::MoreEq => {
                write!(f, "compare")?;
            }
            Operator::Add => {
                write!(f, "add")?;
            }
            Operator::Sub => {
                write!(f, "subtract")?;
            }
            Operator::Mul => {
                write!(f, "multiply")?;
            }
            Operator::Div => {
                write!(f, "divide")?;
            }
        }

        write!(f, " {} and {}", self.lhs.ty(), self.rhs.ty())
    }
}

#[derive(Debug, Display, Clone, Copy)]
#[display("Expected a bool in a \"where\".")]
pub struct ExpectedBoolError;

#[derive(Debug, Display, Clone, Copy)]
#[display("Expected a single value.")]
pub struct ExpectedValueError;

#[derive(Debug, Display, Clone, Copy)]
#[display("Column \"id\" is only inserted automatically.")]
pub struct IdInsertError;

pub type InsertError = OneOf<(InsertionError, NoSuchColumnError, IdInsertError)>;

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

    pub fn row(&self, index: usize) -> Option<HashMap<String, Value>> {
        if index >= self.height() {
            return None;
        }

        let mut row = HashMap::new();
        for column in &self.columns {
            let Column {
                identifier, values, ..
            } = column;
            row.insert(identifier.clone(), values[index].clone());
        }

        Some(row)
    }

    pub fn column(&self, identifier: &str) -> Option<&Column> {
        self.columns.iter().find(
            |Column {
                 identifier: column_identifier,
                 ..
             }| *column_identifier == identifier,
        )
    }

    /// # Panics
    /// if `index < 0` or `index > self.height()`
    pub fn remove_row(&mut self, index: usize) {
        for column in &mut self.columns {
            column.values.remove(index);
        }
    }

    pub fn get(
        &self,
        selections: Vec<Selection>,
        filter: Option<Expression>,
    ) -> Result<Table, OneOf<(NoSuchColumnError, CannotEvaluateError, ExpectedBoolError)>> {
        let mut columns = Vec::new();
        for selection in &selections {
            match selection {
                Selection::Identifier { identifier } => {
                    let Some(Column {
                        identifier,
                        ty,
                        optional,
                        ..
                    }) = self.column(identifier)
                    else {
                        return Err(OneOf::new(NoSuchColumnError(identifier.clone())));
                    };

                    columns.push(Column {
                        identifier: identifier.clone(),
                        ty: *ty,
                        optional: *optional,
                        values: Vec::new(),
                    })
                }
                Selection::All => {
                    for Column {
                        identifier,
                        ty,
                        optional,
                        ..
                    } in &self.columns
                    {
                        columns.push(Column {
                            identifier: identifier.clone(),
                            ty: *ty,
                            optional: *optional,
                            values: Vec::new(),
                        });
                    }
                }
            }
        }

        for i in 0..self.height() {
            let row = self.row(i).unwrap(); // 0..self.height() must exist
            if let Some(expression) = filter.clone() {
                let Value::Bool(bool) = Self::evaluate(expression, &row).map_err(OneOf::broaden)?
                else {
                    return Err(OneOf::new(ExpectedBoolError));
                };

                if !bool {
                    continue;
                }
            }

            for Column {
                identifier, values, ..
            } in &mut columns
            {
                let Some(value) = row.get(identifier) else {
                    unreachable!();
                };

                values.push(value.clone());
            }
        }

        Ok(Table { columns })
    }

    pub fn insert(&mut self, mut insertion: HashMap<String, Value>) -> Result<(), InsertError> {
        // Insertion validation.
        // ---------------------@
        if insertion.contains_key(ID_IDENTIFIER) {
            return Err(OneOf::new(IdInsertError));
        }

        let last_id = {
            if self.height() == 0 {
                0
            } else {
                let Value::Int(id) = self
                    .column(ID_IDENTIFIER)
                    .expect("column Id does not exist?")
                    .values
                    .last()
                    .expect("?")
                else {
                    panic!("Id is not an int?")
                };

                *id as i32 + 1
            }
        };
        insertion.insert(ID_IDENTIFIER.to_owned(), Value::Int(last_id));

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

    pub fn remove(
        &mut self,
        expression: Expression,
    ) -> Result<usize, OneOf<(ExpectedBoolError, CannotEvaluateError, NoSuchColumnError)>> {
        let mut remove_indices = Vec::new();
        for i in 0..self.height() {
            let row = self.row(i).expect("?");
            let Value::Bool(bool) =
                Self::evaluate(expression.clone(), &row).map_err(OneOf::broaden)?
            else {
                return Err(OneOf::new(ExpectedBoolError));
            };

            if bool {
                remove_indices.push(i);
            }
        }

        let count = remove_indices.len();
        for (i, index) in remove_indices.into_iter().enumerate() {
            // Need to offset indices after every removal.
            self.remove_row(index - i);
        }

        Ok(count)
    }

    fn evaluate(
        expression: Expression,
        row: &HashMap<String, Value>,
    ) -> Result<Value, OneOf<(CannotEvaluateError, NoSuchColumnError)>> {
        match expression {
            Expression::Value(value) => Ok(value),
            Expression::Identifier(identifer) => {
                let Some(value) = row.get(&identifer) else {
                    return Err(OneOf::new(NoSuchColumnError(identifer)));
                };

                Ok(value.clone())
            }
            Expression::Enclosed(expression) => Self::evaluate(*expression, row),
            Expression::Operation { lhs, operator, rhs } => {
                crate::operator_map! {
                    Self::evaluate(*lhs, row)?,
                    operator,
                    Self::evaluate(*rhs, row)?,
                    Add {
                        Int(lhs), Int(rhs) => Value::Int(lhs + rhs)
                        Float(lhs), Float(rhs) => Value::Float(lhs + rhs)
                    }
                    Sub {
                        Int(lhs), Int(rhs) => Value::Int(lhs - rhs)
                        Float(lhs), Float(rhs) => Value::Float(lhs - rhs)
                    }
                    Mul {
                        Int(lhs), Int(rhs) => Value::Int(lhs * rhs)
                        Float(lhs), Float(rhs) => Value::Float(lhs * rhs)
                    }
                    Div {
                        Int(lhs), Int(rhs) => Value::Int(lhs / rhs)
                        Float(lhs), Float(rhs) => Value::Float(lhs / rhs)
                    }
                    Eq {
                        Int(lhs), Int(rhs) => Value::Bool(lhs == rhs)
                        Float(lhs), Float(rhs) => Value::Bool(lhs == rhs)
                        Str(lhs), Str(rhs) => Value::Bool(lhs == rhs)
                    }
                    Less {
                        Int(lhs), Int(rhs) => Value::Bool(lhs < rhs)
                        Float(lhs), Float(rhs) => Value::Bool(lhs < rhs)
                    }
                    LessEq {
                        Int(lhs), Int(rhs) => Value::Bool(lhs <= rhs)
                        Float(lhs), Float(rhs) => Value::Bool(lhs <= rhs)
                    }
                    More {
                        Int(lhs), Int(rhs) => Value::Bool(lhs > rhs)
                        Float(lhs), Float(rhs) => Value::Bool(lhs == rhs)
                    }
                    MoreEq {
                        Int(lhs), Int(rhs) => Value::Bool(lhs >= rhs)
                        Float(lhs), Float(rhs) => Value::Bool(lhs >= rhs)
                    }
                    And {
                        Bool(lhs), Bool(rhs) => Value::Bool(lhs && rhs)
                    }
                    Or {
                        Bool(lhs), Bool(rhs) => Value::Bool(lhs || rhs)
                    }
                }
            }
        }
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
        write!(f, "+")?;
        for max_width in &max_widths {
            for _ in 0..max_width + PADDING * 2 {
                write!(f, "-")?;
            }

            write!(f, "+")?;
        }

        for j in 0..self.height() {
            writeln!(f)?;
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
