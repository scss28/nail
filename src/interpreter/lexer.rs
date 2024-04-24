use std::{ops::Range, str::FromStr};

use super::token::{Keyword, Literal, Token};

#[derive(Debug, Clone, Copy)]
pub enum TokenizeError {
    NonTerminatedStr,
    NonUTF8,
    UnexpectedCharacter,
}

pub struct TokenIter<'a> {
    bytes: &'a [u8],
    last_token_index: usize,
    index: usize,
}

impl<'a> TokenIter<'a> {
    fn next_byte(&mut self) -> Option<u8> {
        let byte = self.bytes.get(self.index)?;
        self.index += 1;
        Some(*byte)
    }

    fn next_byte_if(&mut self, f: impl Fn(u8) -> bool) -> Option<u8> {
        if self.peek_byte().is_some_and(f) {
            return self.next_byte();
        }

        None
    }

    fn peek_byte(&self) -> Option<u8> {
        self.bytes.get(self.index).copied()
    }

    fn next_token(&mut self, byte: u8) -> Result<Token, TokenizeError> {
        match byte {
            b'a'..=b'z' | b'A'..=b'Z' => {
                let mut bytes = vec![byte];
                while let Some(byte) = self.next_byte_if(
                    |byte| matches!(byte, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_'),
                ) {
                    bytes.push(byte);
                }

                // Safe since it can only have utf-8 bytes.
                let str = unsafe { String::from_utf8_unchecked(bytes) };
                if let Ok(keyword) = Keyword::from_str(&str) {
                    return Ok(Token::Keyword(keyword));
                }

                Ok(Token::Identifier(str.into_boxed_str()))
            }
            b'"' => {
                let mut bytes = Vec::new();
                while let Some(byte) = self.next_byte() {
                    match byte {
                        b'"' => {
                            let Ok(str) = String::from_utf8(bytes) else {
                                return Err(TokenizeError::NonUTF8);
                            };

                            return Ok(Token::Literal(Literal::Str(str.into_boxed_str())));
                        }
                        byte => bytes.push(byte),
                    }
                }

                Err(TokenizeError::NonTerminatedStr)
            }
            b'*' => Ok(Token::Star),
            b',' => Ok(Token::Comma),
            b':' => Ok(Token::Colon),
            b';' => Ok(Token::SemiColon),
            _ => Err(TokenizeError::UnexpectedCharacter),
        }
    }
}

impl<'a> From<&'a [u8]> for TokenIter<'a> {
    fn from(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            last_token_index: 0,
            index: 0,
        }
    }
}

impl<'a> Iterator for TokenIter<'a> {
    type Item = Result<Token, TokenizeError>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.peek_byte()?.is_ascii_whitespace() {
            _ = self.next_byte();
        }

        self.last_token_index = self.index;
        let byte = self.next_byte()?;
        Some(self.next_token(byte))
    }
}
