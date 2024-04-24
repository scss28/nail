use std::{collections::HashMap, iter::Peekable, ops::Range};

use super::{
    command::{Command, Selection, Ty},
    lexer::{self, TokenIter, TokenizeError},
    token::{Keyword, Literal, Token},
};

#[derive(Debug, Clone)]
pub enum ParseError {
    TokenizeError(TokenizeError),
    ExpectedToken,
    UnexpectedToken,
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
        self.peeked.take().unwrap_or(self.tokens.next())
    }

    fn peek_token(&mut self) -> Option<&lexer::Result> {
        let tokens = &mut self.tokens;
        self.peeked.get_or_insert(tokens.next()).as_ref()
    }

    fn next_token_req(&mut self) -> Result<lexer::Result, ParseError> {
        self.next_token().ok_or(ParseError::ExpectedToken)
    }

    fn next_command(&mut self, token: Result<Token, TokenizeError>) -> Result<Command, ParseError> {
        let Token::Keyword(keyword) = token? else {
            return Err(ParseError::UnexpectedToken);
        };

        let command = match keyword {
            Keyword::From => {
                let (Token::Identifier(identifier) | Token::Literal(Literal::Str(identifier))) =
                    self.next_token_req()??
                else {
                    return Err(ParseError::UnexpectedToken);
                };

                let Token::Keyword(Keyword::Get) = self.next_token_req()?? else {
                    return Err(ParseError::UnexpectedToken);
                };

                let mut selection = Vec::new();
                while let Some(token) = self.peek_token() {
                    let token = token.clone()?;
                    if let Token::SemiColon = token {
                        break;
                    }

                    if !selection.is_empty() {
                        let Token::Comma = token else {
                            return Err(ParseError::UnexpectedToken);
                        };

                        _ = self.next_token();
                    }

                    selection.push(match self.next_token_req()?? {
                        Token::Identifier(identifier)
                        | Token::Literal(Literal::Str(identifier)) => Selection::Column(identifier),
                        Token::Star => Selection::All,
                        _ => return Err(ParseError::UnexpectedToken),
                    });
                }

                Command::Get {
                    identifier,
                    selection,
                }
            }
            Keyword::New => {
                let (Token::Identifier(identifier) | Token::Literal(Literal::Str(identifier))) =
                    self.next_token_req()??
                else {
                    return Err(ParseError::UnexpectedToken);
                };

                let mut columns = HashMap::new();
                while let Some(token) = self.next_token() {
                    let token = token.clone()?;
                    if let Token::SemiColon = token {
                        break;
                    }

                    if !columns.is_empty() {
                        let Token::Comma = token else {
                            return Err(ParseError::UnexpectedToken);
                        };

                        _ = self.next_token();
                    }

                    let (Token::Identifier(identifier) | Token::Literal(Literal::Str(identifier))) =
                        self.next_token_req()??
                    else {
                        return Err(ParseError::UnexpectedToken);
                    };

                    let ty = match self.next_token_req()?? {
                        Token::Keyword(Keyword::Str) => Ty::Str,
                        _ => return Err(ParseError::UnexpectedToken),
                    };

                    columns.insert(identifier, ty);
                }

                Command::New {
                    identifier,
                    columns,
                }
            }
            _ => return Err(ParseError::UnexpectedToken),
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
            _ => Err(ParseError::UnexpectedToken),
        })
    }
}
