use nail::prelude::*;

fn main() {
    let src = br#"
        # To create a table simply:
        new table Person 
            name: str, 
            surname: str, 
            age: int,
            job: str?; # optional column

        # To insert some rows simply:
        insert Person
            name: "Joe", surname: "Kowalski", age: 35, job: "Police Officer";
        insert Person
            name: "Croki", surname: "Actimel", age: 135, job: "Pilot";
        insert Person
            name: "Bob", surname: "Bob", age: 88;
        insert Person
            name: "Suzuki", surname: "Satoru", age: 45, job: "Salaryman";


        from Person get @Id, *;
        # RowAttribute --^   ^-- Gets all rows.

        from Person get @Id, job as "Jabba job";
        #      Just a row ----^
    "#;

    let mut database = Database::new();

    let tokens = TokenIter::new(src);
    let commands = CommandIter::new(tokens);
    for command in commands
        .collect::<Result<Vec<_>, _>>()
        .expect("syntax error")
    {
        let output = database.run_command(command).expect("run error");
        println!("{output}");
    }
}
