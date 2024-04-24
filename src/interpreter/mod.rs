use std::string::ParseError;

mod command;
mod lexer;
mod parser;
mod token;
mod util;

pub fn run(src: impl AsRef<[u8]>) -> Result<(), RunError> {
    // let mut tables = HashMap::<_, HashMap<_, _>>::new();
    let src = src.as_ref();
    let tokens = lexer::TokenIter::from(src);
    #[cfg(debug_assertions)]
    {
        let mut tokens = tokens.clone();
        while let Some(token) = tokens.next() {
            let pos = tokens.src_pos();
            let (line, char) = util::src_position(src, pos.start);
            println!(
                "{:?} {:?} ({}:{})",
                token,
                std::str::from_utf8(&src[pos]).unwrap(),
                line,
                char
            )
        }
    }

    let mut commands = parser::CommandIter::new(tokens);
    while let Some(command) = commands.next() {
        match command {
            Ok(command) => println!("{command:?}"),
            Err(err) => {
                let pos = commands.src_pos();
                let (line, char) = util::src_position(src, pos.start);
                println!(
                    "{err:?} {:?} ({}:{})",
                    std::str::from_utf8(&src[pos]).unwrap(),
                    line,
                    char
                )
            }
        }
    }

    Ok(())
}

#[derive(Debug, Clone)]
pub enum RunError {
    ParseError(ParseError),
}

impl From<ParseError> for RunError {
    fn from(value: ParseError) -> Self {
        Self::ParseError(value)
    }
}
