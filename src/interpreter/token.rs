use std::str::FromStr;

use super::Value;

#[derive(Debug, Clone)]
pub enum Token {
    Literal(Value),
    Identifier(Box<str>),
    Keyword(Keyword),
    Star,
    Comma,
    Colon,
    SemiColon,
    At,
    LeftSmooth,
    RightSmooth,
    QuestionMark,
}

#[derive(Debug, Clone, Copy)]
pub enum Keyword {
    Get,
    From,
    Table,
    Str,
    New,
    Insert,
}

#[derive(Debug, Clone, Copy)]
pub struct NoSuchKeywordError;
impl FromStr for Keyword {
    type Err = NoSuchKeywordError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "get" => Keyword::Get,
            "from" => Keyword::From,
            "table" => Keyword::Table,
            "str" => Keyword::Str,
            "new" => Keyword::New,
            "insert" => Keyword::Insert,
            _ => return Err(NoSuchKeywordError),
        })
    }
}
