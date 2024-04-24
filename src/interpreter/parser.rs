enum Statement {
    New {
        identifier: Box<str>,
        columns: Vec<(Box<str>, Ty)>,
    },
    Insert {
        identifier: Box<str>,
        values: Vec<Expression>,
    },
    Get {
        identifier: Box<str>,
        selections: Vec<Selection>,
    },
}

enum Expression {
    Literal(),
}

enum Ty {
    Str,
}

enum Selection {
    Column(Box<str>),
    RowAttribute(),
}

enum RowAttribute {
    Id,
}
