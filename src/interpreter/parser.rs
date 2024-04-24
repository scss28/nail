use std::{collections::HashMap, iter::Peekable, ops::Range, process::id};

use super::{
    command::{Command, Expression, Selection, Ty},
    lexer::{self, TokenIter, TokenizeError},
    token::{Keyword, Token},
    Value,
};

#[derive(Debug, Clone)]
pub enum ParseError {
    TokenizeError(TokenizeError),
    ExpectedToken(String),
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
            Token::Literal(literal) => Expression::Literal(literal)
        }
    }

    fn next_command(&mut self, token: Result<Token, TokenizeError>) -> Result<Command, ParseError> {
        let Token::Keyword(keyword) = token? else {
            return Err(ParseError::ExpectedToken("from / insert / new".to_owned()));
        };

        let command = match keyword {
            Keyword::From => {
                let identifier = expect_token! {
                    self.next_token(),
                    "identifier",
                    Token::Identifier(identifier)
                        | Token::Literal(Value::Str(identifier)) => identifier
                }?;

                expect_token! {
                    self.next_token(),
                    "get",
                    Token::Keyword(Keyword::Get) => {}
                }?;

                let mut selections = Vec::new();
                while let Some(token) = self.peek_token() {
                    if let Ok(Token::SemiColon) = token {
                        break;
                    }

                    if !selections.is_empty() {
                        let Ok(Token::Comma) = token else {
                            return Err(ParseError::ExpectedToken(",".to_owned()));
                        };

                        _ = self.next_token();
                    }

                    selections.push(expect_token! {
                        self.next_token(),
                        "* / column name",
                        Token::Identifier(identifier)
                            | Token::Literal(Value::Str(identifier)) => {
                            Selection::Column(identifier)
                        },
                        Token::Star => Selection::All,
                        Token::At => {
                            let attribute = expect_token! {
                                self.next_token(),
                                "row attribute",
                                Token::Identifier(identifier)
                                    | Token::Literal(Value::Str(identifier)) => identifier
                            }?;

                            Selection::RowAttribute(attribute)
                        }
                    }?);
                }

                Command::Get {
                    identifier,
                    selections,
                }
            }
            Keyword::New => {
                let identifier = expect_token! {
                    self.next_token(),
                    "identifier",
                    Token::Identifier(identifier)
                        | Token::Literal(Value::Str(identifier)) => identifier
                }?;

                let mut columns = HashMap::new();
                while let Some(token) = self.peek_token() {
                    if let Ok(Token::SemiColon) = token {
                        break;
                    }

                    if !columns.is_empty() {
                        let Ok(Token::Comma) = token else {
                            return Err(ParseError::ExpectedToken(",".to_owned()));
                        };

                        _ = self.next_token();
                    }

                    let identifier = expect_token! {
                        self.next_token(),
                        "identifier",
                        Token::Identifier(identifier)
                            | Token::Literal(Value::Str(identifier)) => identifier
                    }?;

                    let ty = expect_token! {
                        self.next_token(),
                        "type",
                        Token::Keyword(Keyword::Str) => Ty::Str,
                    }?;

                    let optional = if matches!(self.peek_token(), Some(Ok(Token::QuestionMark))) {
                        _ = self.next_token();
                        true
                    } else {
                        false
                    };

                    columns.insert(identifier, (optional, ty));
                }

                Command::New {
                    identifier,
                    columns,
                }
            }
            Keyword::Insert => {
                let identifier = expect_token! {
                    self.next_token(),
                    "identifier",
                    Token::Identifier(identifier)
                        | Token::Literal(Value::Str(identifier)) => identifier
                }?;

                let mut inserts = Vec::new();
                while let Some(token) = self.peek_token() {
                    if let Ok(Token::SemiColon) = token {
                        break;
                    }

                    if !inserts.is_empty() {
                        let Ok(Token::Comma) = token else {
                            return Err(ParseError::ExpectedToken(",".to_owned()));
                        };

                        _ = self.next_token();
                    }

                    expect_token! {
                        self.next_token(),
                        "(",
                        Token::LeftSmooth => {}
                    }?;

                    let mut expressions = HashMap::new();
                    while let Some(token) = self.peek_token() {
                        if let Ok(Token::RightSmooth) = token {
                            break;
                        }

                        if !expressions.is_empty() {
                            let Ok(Token::Comma) = token else {
                                return Err(ParseError::ExpectedToken(",".to_owned()));
                            };

                            _ = self.next_token();
                        }

                        let identifier = expect_token! {
                            self.next_token(),
                            "identifier",
                            Token::Identifier(identifier)
                                | Token::Literal(Value::Str(identifier)) => identifier
                        }?;

                        expect_token! {
                            self.next_token(),
                            ":",
                            Token::Colon => {}
                        }?;

                        expressions.insert(identifier, self.next_expression()?);
                    }

                    expect_token! {
                        self.next_token(),
                        ")",
                        Token::RightSmooth => {}
                    }?;

                    inserts.push(expressions);
                }

                Command::Insert {
                    identifier,
                    inserts,
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
        let command = self.next_command(token);
        Some(match self.next_token() {
            Some(Ok(Token::SemiColon)) | None => command,
            Some(Err(err)) => Err(err.into()),
            _ => Err(ParseError::ExpectedToken(";".to_owned())),
        })
    }
}
