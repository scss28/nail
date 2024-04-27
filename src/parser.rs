use super::{
    command::{ColumnDefinition, Command, Expression, Selection},
    lexer::{self, TokenIter, TokenizeError},
    token::{Keyword, Token},
    Value,
};
use crate::{command::Operator, Ty};
use parse_display_derive::Display;
use std::{collections::HashMap, ops::Range};

#[derive(Debug, Display, Clone)]
pub enum ParseError {
    #[display("{0}")]
    TokenizeError(TokenizeError),
    #[display("Expected: {0}.")]
    ExpectedToken(String),
    #[display("No such row attribute.")]
    NoSuchRowAttribute,
}

impl From<TokenizeError> for ParseError {
    fn from(value: TokenizeError) -> Self {
        Self::TokenizeError(value)
    }
}

pub struct CommandIter<'a> {
    tokens: TokenIter<'a>,
    peeked: Option<Option<lexer::Result>>,
}

impl<'a> CommandIter<'a> {
    pub fn new(tokens: TokenIter<'a>) -> Self {
        Self {
            tokens,
            peeked: None,
        }
    }

    pub fn src_pos(&self) -> Range<usize> {
        self.tokens.src_pos()
    }

    fn next_token(&mut self) -> Option<lexer::Result> {
        self.peeked.take().unwrap_or_else(|| self.tokens.next())
    }

    fn peek_token(&mut self) -> Option<&lexer::Result> {
        self.peeked
            .get_or_insert_with(|| self.tokens.next())
            .as_ref()
    }

    fn peek_value(&mut self) -> Option<Value> {
        Some(match self.peek_token() {
            Some(Ok(Token::StrLiteral(str))) => Value::Str(str.clone()),
            Some(Ok(Token::IntLiteral(int))) => Value::Int(*int),
            Some(Ok(Token::FloatLiteral(float))) => Value::Float(*float),
            Some(Ok(Token::Keyword(Keyword::Nil))) => Value::Nil,
            Some(Ok(Token::Keyword(Keyword::True))) => Value::Bool(true),
            Some(Ok(Token::Keyword(Keyword::False))) => Value::Bool(false),
            _ => return None,
        })
    }

    fn next_single_expression(&mut self) -> Result<Expression, ParseError> {
        if let Some(value) = self.peek_value() {
            _ = self.next_token();
            return Ok(Expression::Value(value));
        }

        crate::expect_token! {
            self.next_token(),
            "expression",
            Token::Identifier(identifier)
                | Token::StrLiteral(identifier) => Expression::Identifier(identifier),
            Token::LeftSmooth => {
                let expression = self.next_expression()?;
                crate::expect_token! {
                    self.next_token(),
                    ")",
                    Token::RightSmooth => {}
                }?;

                expression
            }
        }
    }

    fn next_expression(&mut self) -> Result<Expression, ParseError> {
        let mut expression = self.next_single_expression()?;
        loop {
            let operator = match self.peek_token() {
                Some(Ok(token)) => match Operator::try_from(token) {
                    Ok(operator) => {
                        _ = self.next_token();
                        operator
                    }
                    Err(_) => break,
                },
                _ => break,
            };

            expression = expression.extended(operator, self.next_single_expression()?);
        }

        Ok(expression)
    }

    fn next_insertion(&mut self) -> Result<HashMap<String, Value>, ParseError> {
        let mut insertion = HashMap::new();
        while let Some(token) = self.peek_token() {
            if let Ok(Token::SemiColon) = token {
                break;
            }

            if !insertion.is_empty() {
                let Ok(Token::Comma) = token else {
                    return Err(ParseError::ExpectedToken(",".to_owned()));
                };

                _ = self.next_token();
            }

            let identifier = crate::expect_token! {
                self.next_token(),
                "<identifier>",
                Token::Identifier(identifier)
                    | Token::StrLiteral(identifier) => identifier
            }?;

            crate::expect_token! {
                self.next_token(),
                ":",
                Token::Colon => {}
            }?;

            let Some(value) = self.peek_value() else {
                _ = self.next_token();
                return Err(ParseError::ExpectedToken("value".to_owned()));
            };

            _ = self.next_token();
            insertion.insert(identifier, value);
        }

        Ok(insertion)
    }

    fn next_command(&mut self, token: Result<Token, TokenizeError>) -> Result<Command, ParseError> {
        let Token::Keyword(keyword) = token? else {
            return Err(ParseError::ExpectedToken(
                "from or insert or new".to_owned(),
            ));
        };

        match keyword {
            Keyword::Get => {
                let identifier = crate::expect_token! {
                    self.next_token(),
                    "<identifier>",
                    Token::Identifier(identifier)
                        | Token::StrLiteral(identifier)  => identifier
                }?;

                let selections = match self.peek_token() {
                    Some(Ok(Token::Keyword(Keyword::Select))) => {
                        _ = self.next_token();

                        let mut selections = Vec::new();
                        while let Some(token) = self.peek_token() {
                            if !selections.is_empty() {
                                let Ok(Token::Comma) = token else {
                                    break;
                                };

                                _ = self.next_token();
                            }

                            selections.push(crate::expect_token! {
                                self.next_token(),
                                "* or <column name> or @<row attribute>",
                                Token::Identifier(identifier)
                                    | Token::StrLiteral(identifier) => {
                                    Selection::Identifier { identifier }
                                },
                                Token::Star => Selection::All,
                            }?);
                        }

                        selections
                    }
                    _ => vec![Selection::All],
                };

                let filter = match self.peek_token() {
                    Some(Ok(Token::Keyword(Keyword::Where))) => {
                        _ = self.next_token();
                        Some(self.next_expression()?)
                    }
                    _ => None,
                };

                Ok(Command::Get {
                    identifier,
                    selections,
                    filter,
                })
            }
            Keyword::New => {
                crate::expect_token! {
                    self.next_token(),
                    "table",
                    Token::Keyword(Keyword::Table) => {}
                }?;

                let identifier = crate::expect_token! {
                    self.next_token(),
                    "<identifier>",
                    Token::Identifier(identifier)
                        | Token::StrLiteral(identifier) => identifier
                }?;

                let mut definitions = Vec::new();
                while let Some(token) = self.peek_token() {
                    if let Ok(Token::SemiColon) = token {
                        break;
                    }

                    if !definitions.is_empty() {
                        let Ok(Token::Comma) = token else {
                            return Err(ParseError::ExpectedToken(",".to_owned()));
                        };

                        _ = self.next_token();
                    }

                    let identifier = crate::expect_token! {
                        self.next_token(),
                        "<identifier>",
                        Token::Identifier(identifier)
                            | Token::StrLiteral(identifier) => identifier
                    }?;

                    crate::expect_token! {
                        self.next_token(),
                        ":",
                        Token::Colon => {}
                    }?;

                    let ty = crate::expect_token! {
                        self.next_token(),
                        "<type>",
                        Token::Keyword(Keyword::Str) => Ty::Str,
                        Token::Keyword(Keyword::Int) => Ty::Int,
                        Token::Keyword(Keyword::Float) => Ty::Float,
                        Token::Keyword(Keyword::Bool) => Ty::Bool,
                    }?;

                    let optional = if matches!(self.peek_token(), Some(Ok(Token::QuestionMark))) {
                        _ = self.next_token();
                        true
                    } else {
                        false
                    };

                    definitions.push(ColumnDefinition {
                        identifier,
                        optional,
                        ty,
                    });
                }

                Ok(Command::New {
                    identifier,
                    definitions,
                })
            }
            Keyword::Insert => {
                let identifier = crate::expect_token! {
                    self.next_token(),
                    "<identifier>",
                    Token::Identifier(identifier)
                        | Token::StrLiteral(identifier) => identifier
                }?;

                let mut insertions = Vec::new();
                match self.peek_token() {
                    Some(Ok(Token::LeftCurly)) => {
                        _ = self.next_token();
                        while !matches!(self.peek_token(), Some(Ok(Token::RightCurly)) | None) {
                            insertions.push(self.next_insertion()?);
                            crate::expect_token! {
                                self.next_token(),
                                ";",
                                Token::SemiColon => {}
                            }?;
                        }

                        crate::expect_token! {
                            self.next_token(),
                            "}",
                            Token::RightCurly => {}
                        }?;
                    }
                    _ => insertions.push(self.next_insertion()?),
                }

                Ok(Command::Insert {
                    identifier,
                    insertions,
                })
            }
            Keyword::Remove => {
                let identifier = crate::expect_token! {
                    self.next_token(),
                    "<identifier>",
                    Token::Identifier(identifier)
                        | Token::StrLiteral(identifier) => identifier
                }?;

                crate::expect_token! {
                    self.next_token(),
                    "where",
                    Token::Keyword(Keyword::Where) => {}
                }?;

                let expression = self.next_expression()?;

                Ok(Command::Remove {
                    identifier,
                    expression,
                })
            }
            _ => Err(ParseError::ExpectedToken("from / insert / new".to_owned())),
        }
    }
}

impl<'a> Iterator for CommandIter<'a> {
    type Item = Result<Command, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        let token = self.next_token()?;
        let command = match self.next_command(token) {
            Ok(command) => command,
            Err(err) => return Some(Err(err)),
        };

        Some(match self.next_token() {
            Some(Ok(Token::SemiColon)) | None => Ok(command),
            Some(Err(err)) => Err(err.into()),
            _ => Err(ParseError::ExpectedToken(";".to_owned())),
        })
    }
}
