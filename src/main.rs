mod interpreter;
mod span;

fn main() {
    let src = r#"
new Student name str, surname str;
# insert Student "Gaming", "Fungus";

#from Student get @id;
    "#;

    interpreter::run(src).expect("Running went oof");
}
