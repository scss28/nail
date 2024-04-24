mod interpreter;
mod span;

fn main() {
    let src = r#"
        new Student
            name str,
            surname str,
            gamer str?;
        
        insert Student
            (name: "Gaming", surname: "Fungus"),
            (name: "Yuh", surname: "Yuh", gamer: "h");

        from Student get @id, name, surname, gamer;
    "#;

    interpreter::run(src).expect("Running went oof");
}
