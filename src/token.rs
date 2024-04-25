use super::Value;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub enum Token {
    Literal(Value),
    Identifier(String),
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
    New,
    Insert,
    // Types
    // :
    Str,
    Int,
    Float,
    Nil,
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
            "new" => Keyword::New,
            "insert" => Keyword::Insert,
            "str" => Keyword::Str,
            "int" => Keyword::Int,
            "float" => Keyword::Float,
            "nil" => Keyword::Nil,
            _ => return Err(NoSuchKeywordError),
        })
    }
}
