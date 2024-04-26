use std::str::FromStr;

#[derive(Debug, Clone)]
pub enum Token {
    StrLiteral(String),
    IntLiteral(i32),
    FloatLiteral(f32),
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
    Eq,
    More,
    MoreEq,
    Less,
    LessEq,
}

#[derive(Debug, Clone, Copy)]
pub enum Keyword {
    Get,
    From,
    Table,
    New,
    Insert,
    As,
    Where,
    // Types
    // -----@
    Str,
    Int,
    Float,
    Bool,
    Nil,
    // Bool literals
    // -------------@
    True,
    False,
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
            "as" => Keyword::As,
            "where" => Keyword::Where,
            // Types
            // -----@
            "str" => Keyword::Str,
            "int" => Keyword::Int,
            "float" => Keyword::Float,
            "nil" => Keyword::Nil,
            "bool" => Keyword::Bool,
            // Bool literals
            // -------------@
            "true" => Keyword::True,
            "false" => Keyword::False,
            _ => return Err(NoSuchKeywordError),
        })
    }
}
