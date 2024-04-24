mod interpreter;
mod span;

fn main() {
    let src = r#"
        new table Table x: str, y: str;
        from Table get x, y;
    "#;

    interpreter::run(src);
}
