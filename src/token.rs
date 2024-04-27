use std::str::FromStr;

#[derive(Debug, Clone)]
pub enum Token {
    StrLiteral(String),
    IntLiteral(i32),
    FloatLiteral(f32),
    Identifier(String),
    Keyword(Keyword),
    Comma,
    Colon,
    SemiColon,
    At,
    LeftSmooth,
    RightSmooth,
    LeftCurly,
    RightCurly,
    QuestionMark,
    // Operators
    // ---------@
    DoubleEq,
    More,
    MoreEq,
    Less,
    LessEq,
    Plus,
    Minus,
    Star,
    Slash,
    DoubleAmpersand,
    DoublePipe,
}

#[derive(Debug, Clone, Copy)]
pub enum Keyword {
    Get,
    Select,
    Table,
    New,
    Insert,
    As,
    Where,
    Remove,
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
            "select" => Keyword::Select,
            "table" => Keyword::Table,
            "new" => Keyword::New,
            "insert" => Keyword::Insert,
            "as" => Keyword::As,
            "where" => Keyword::Where,
            "remove" => Keyword::Remove,
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
