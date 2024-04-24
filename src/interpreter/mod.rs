use std::collections::HashMap;

use self::{
    lexer::TokenizeError,
    token::{Keyword, Literal, Token},
};

mod lexer;
mod parser;
mod token;

pub fn run(src: impl AsRef<[u8]>) -> Result<(), RunError> {
    let tables = HashMap::new();
    let mut tokens = lexer::TokenIter::from(src.as_ref());
    while let Some(token) = tokens.next() {
        let Token::Keyword(keyword) = token? else {
            return Err(RunError::UnexpectedToken);
        };

        match keyword {
            Keyword::From => {}
            Keyword::New => {
                match tokens.next() {
                    Some(Ok(Token::Keyword(Keyword::Table))) => {}
                    Some(Err(err)) => return Err(err.into()),
                    _ => return Err(RunError::UnexpectedToken),
                }

                let identifier = match tokens.next() {
                    Some(Ok(Token::Identifier(str) | Token::Literal(Literal::Str(str)))) => str,
                    Some(Err(err)) => return Err(err.into()),
                    _ => return Err(RunError::UnexpectedToken),
                };
            }
            _ => return Err(RunError::UnexpectedToken),
        }
    }

    Ok(())
}

#[derive(Debug, Clone)]
pub enum RunError {
    TokenizeError(TokenizeError),
    UnexpectedToken,
}

impl From<TokenizeError> for RunError {
    fn from(value: TokenizeError) -> Self {
        Self::TokenizeError(value)
    }
}

struct Table {
    columns: HashMap<Box<str>, Column>,
}

enum Column {
    Str(Vec<String>),
}
