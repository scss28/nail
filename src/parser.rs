use std::{collections::HashMap, ops::Range, str::FromStr};

use parse_display_derive::Display;

use crate::{command::RowAttribute, Ty};

use super::{
    command::{ColumnDefinition, Command, Expression, Selection},
    lexer::{self, TokenIter, TokenizeError},
    token::{Keyword, Token},
    Value,
};

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

macro_rules! expect_token {
    ($expr:expr, $msg:expr, $($pat:pat => $pat_expr:expr),* $(,)?) => {
        match $expr {
            $(
                Some(Ok($pat)) => Ok($pat_expr),
            )*
            Some(Err(err)) => Err(err.into()),
            _ => Err(ParseError::ExpectedToken($msg.into())),
        }
    };
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

    fn next_expression(&mut self) -> Result<Expression, ParseError> {
        expect_token! {
            self.next_token(),
            "expression",
            Token::StrLiteral(str) => Expression::Value(Value::Str(str)),
            Token::IntLiteral(int) => Expression::Value(Value::Int(int)),
            Token::FloatLiteral(float) => Expression::Value(Value::Float(float)),
            Token::Keyword(Keyword::Nil) => Expression::Value(Value::Nil),
            Token::Keyword(Keyword::True) => Expression::Value(Value::Bool(true)),
            Token::Keyword(Keyword::False) => Expression::Value(Value::Bool(false)),
        }
    }

    fn next_insertion(&mut self) -> Result<HashMap<String, Expression>, ParseError> {
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

            let identifier = expect_token! {
                self.next_token(),
                "<identifier>",
                Token::Identifier(identifier)
                    | Token::StrLiteral(identifier) => identifier
            }?;

            expect_token! {
                self.next_token(),
                ":",
                Token::Colon => {}
            }?;

            insertion.insert(identifier, self.next_expression()?);
        }

        Ok(insertion)
    }

    fn next_command(&mut self, token: Result<Token, TokenizeError>) -> Result<Command, ParseError> {
        let Token::Keyword(keyword) = token? else {
            return Err(ParseError::ExpectedToken(
                "from or insert or new".to_owned(),
            ));
        };

        let command = match keyword {
            Keyword::From => {
                let identifier = expect_token! {
                    self.next_token(),
                    "<identifier>",
                    Token::Identifier(identifier)
                        | Token::StrLiteral(identifier)  => identifier
                }?;

                expect_token! {
                    self.next_token(),
                    "get",
                    Token::Keyword(Keyword::Get) => {}
                }?;

                let mut selections = Vec::new();
                while let Some(token) = self.peek_token() {
                    if !selections.is_empty() {
                        let Ok(Token::Comma) = token else {
                            break;
                        };

                        _ = self.next_token();
                    }

                    selections.push(expect_token! {
                        self.next_token(),
                        "* or <column name> or @<row attribute>",
                        Token::Identifier(column)
                            | Token::StrLiteral(column) => {
                            let identifier = if let Some(
                                Ok(Token::Keyword(Keyword::As))
                            ) = self.peek_token() {
                                _ = self.next_token();
                                Some(expect_token! {
                                    self.next_token(),
                                    "<identifier>",
                                    Token::Identifier(identifier)
                                    | Token::StrLiteral(identifier) => {
                                        identifier
                                    }
                                }?)
                            } else {
                                None
                            };

                            Selection::Column {
                                column,
                                identifier
                            }
                        },
                        Token::Star => Selection::All,
                        Token::At => {
                            let attribute = expect_token! {
                                self.next_token(),
                                "<row attribute>",
                                Token::Identifier(identifier)
                                    | Token::StrLiteral(identifier) => identifier
                            }?;

                            let Ok(attribute) = RowAttribute::from_str(&attribute) else  {
                                return Err(ParseError::NoSuchRowAttribute);
                            };

                            let identifier = if let Some(
                                Ok(Token::Keyword(Keyword::As))
                            ) = self.peek_token() {
                                _ = self.next_token();
                                Some(expect_token! {
                                    self.next_token(),
                                    "<identifier>",
                                    Token::Identifier(identifier)
                                    | Token::StrLiteral(identifier) => {
                                        identifier
                                    }
                                }?)
                            } else {
                                None
                            };


                            Selection::RowAttribute {
                                attribute,
                                identifier
                            }
                        }
                    }?);
                }

                let filter = match self.peek_token() {
                    Some(Ok(Token::Keyword(Keyword::Where))) => {
                        _ = self.next_token();
                        Some(Expression::Value(Value::Bool(true)))
                    }
                    _ => None,
                };

                Command::Get {
                    identifier,
                    selections,
                    filter,
                }
            }
            Keyword::New => {
                expect_token! {
                    self.next_token(),
                    "table",
                    Token::Keyword(Keyword::Table) => {}
                }?;

                let identifier = expect_token! {
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

                    let identifier = expect_token! {
                        self.next_token(),
                        "<identifier>",
                        Token::Identifier(identifier)
                            | Token::StrLiteral(identifier) => identifier
                    }?;

                    expect_token! {
                        self.next_token(),
                        ":",
                        Token::Colon => {}
                    }?;

                    let ty = expect_token! {
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

                Command::New {
                    identifier,
                    definitions,
                }
            }
            Keyword::Insert => {
                let identifier = expect_token! {
                    self.next_token(),
                    "<identifier>",
                    Token::Identifier(identifier)
                        | Token::StrLiteral(identifier) => identifier
                }?;

                let mut insertions = Vec::new();
                match self.peek_token() {
                    Some(Ok(Token::LeftSmooth)) => {
                        _ = self.next_token();
                        while !matches!(self.peek_token(), Some(Ok(Token::RightSmooth)) | None) {
                            insertions.push(self.next_insertion()?);
                            expect_token! {
                                self.next_token(),
                                ";",
                                Token::SemiColon => {}
                            }?;
                        }

                        expect_token! {
                            self.next_token(),
                            ")",
                            Token::RightSmooth => {}
                        }?;
                    }
                    _ => insertions.push(self.next_insertion()?),
                }

                Command::Insert {
                    identifier,
                    insertions,
                }
            }
            _ => return Err(ParseError::ExpectedToken("from / insert / new".to_owned())),
        };

        Ok(command)
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
