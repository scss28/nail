use std::{collections::HashMap, ptr::NonNull};

use self::{
    lexer::TokenizeError,
    token::{Keyword, Literal, Token},
};

mod lexer;
mod parser;
mod token;

pub fn run(src: impl AsRef<[u8]>) -> Result<(), RunError> {
    let mut tables = HashMap::<_, HashMap<_, _>>::new();
    let mut tokens = lexer::TokenIter::from(src.as_ref()).peekable();
    while let Some(token) = tokens.next() {
        let Token::Keyword(keyword) = token? else {
            return Err(RunError::UnexpectedToken);
        };

        match keyword {
            Keyword::From => {
                let identifier = match tokens.next() {
                    Some(Ok(Token::Identifier(str) | Token::Literal(Literal::Str(str)))) => str,
                    Some(Err(err)) => return Err(err.into()),
                    _ => return Err(RunError::UnexpectedToken),
                };

                match tokens.next() {
                    Some(Ok(Token::Keyword(Keyword::Get))) => {}
                    Some(Err(err)) => return Err(err.into()),
                    _ => return Err(RunError::UnexpectedToken),
                }

                match tokens.peek() {
                    Some(Ok(Token::Star)) => {
                        _ = tokens.next();
                        let Some(table) = tables.get(&identifier) else {
                            return Err(RunError::NoSuchTable);
                        };

                        for column in table {
                            println!("{column:?}");
                        }
                    }
                    Some(Err(err)) => return Err(err.clone().into()),
                    _ => {
                        let mut column_identifiers = Vec::new();
                        while let Some(token) = tokens.peek() {
                            let token = token.clone()?;
                            if let Token::SemiColon = token {
                                break;
                            }

                            if !column_identifiers.is_empty() {
                                let Token::Comma = token else {
                                    return Err(RunError::ExpectedComma);
                                };

                                _ = tokens.next();
                            }

                            let identifier = match tokens.next() {
                                Some(Ok(
                                    Token::Identifier(str) | Token::Literal(Literal::Str(str)),
                                )) => str,
                                _ => return Err(RunError::UnexpectedToken),
                            };

                            column_identifiers.push(identifier);
                        }

                        let Some(table) = tables.get(&identifier) else {
                            return Err(RunError::NoSuchTable);
                        };

                        for identifier in column_identifiers {
                            let Some(column) = table.get(&identifier) else {
                                return Err(RunError::NoSuchColumn);
                            };

                            println!("{column:?}");
                        }
                    }
                }
            }
            Keyword::New => {
                let identifier = match tokens.next() {
                    Some(Ok(Token::Identifier(str) | Token::Literal(Literal::Str(str)))) => str,
                    Some(Err(err)) => return Err(err.into()),
                    _ => return Err(RunError::UnexpectedToken),
                };

                let mut columns = HashMap::new();
                while let Some(token) = tokens.peek() {
                    let token = token.clone()?;
                    if let Token::SemiColon = token {
                        break;
                    }

                    if !columns.is_empty() {
                        let Token::Comma = token else {
                            return Err(RunError::ExpectedComma);
                        };

                        _ = tokens.next();
                    }

                    let identifier = match tokens.next() {
                        Some(Ok(Token::Identifier(str) | Token::Literal(Literal::Str(str)))) => str,
                        _ => return Err(RunError::UnexpectedToken),
                    };

                    let column = match tokens.next() {
                        Some(Ok(Token::Keyword(Keyword::Str))) => Column::Str(Vec::new()),
                        Some(Err(err)) => return Err(err.into()),
                        _ => return Err(RunError::UnexpectedToken),
                    };

                    columns.insert(identifier, column);
                }

                tables.insert(identifier, columns);
            }
            _ => return Err(RunError::UnexpectedToken),
        }

        match tokens.next() {
            Some(Ok(Token::SemiColon)) => {}
            None => break,
            _ => return Err(RunError::ExpectedSemicolon),
        }
    }

    Ok(())
}

#[derive(Debug, Clone)]
pub enum RunError {
    TokenizeError(TokenizeError),
    UnexpectedToken,
    ExpectedSemicolon,
    ExpectedComma,
    NoSuchTable,
    NoSuchColumn,
}

impl From<TokenizeError> for RunError {
    fn from(value: TokenizeError) -> Self {
        Self::TokenizeError(value)
    }
}

#[derive(Debug, Clone)]
enum Column {
    Str(Vec<Box<str>>),
}
