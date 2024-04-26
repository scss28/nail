use nail::prelude::*;

fn main() {
    let src = br#"
        # To create a table simply:
        new table Person 
            name: str, 
            surname: str, 
            age: int,
            knows_kung_fu: bool,
            job: str?; # optional column

        # To insert some rows simply
        insert Person (
            name: "Joe", surname: "Kowalski", age: 35, knows_kung_fu: true, job: "Police Officer";
            name: "Croki", surname: "Actimel", age: 135,  knows_kung_fu: false, job: "Pilot";
            # No job :(
            name: "Bob", surname: "Bob",  knows_kung_fu: false, age: 9000;
            name: "Suzuki", surname: "Satoru", age: 45,  knows_kung_fu: false, job: "Salaryman";
        );

        #               v-- Identity of the row.
        from Person get I, *;
        #                  ^-- Gets all rows.

        # Just a column ----v
        from Person get I, job as "Jabba job" where age > 40;
        
    "#;

    let mut database = Database::new();

    let tokens = TokenIter::new(src);
    for command in CommandIter::new(tokens)
        .collect::<Result<Vec<_>, _>>()
        .expect("syntax error")
    {
        let output = database.run_command(command).expect("run error");
        println!("{output}\n");
    }
}
