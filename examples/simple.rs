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
            (name: "Joe", surname: "Kowalski", age: 35, job: "Police Officer"),
            (name: "Croki", surname: "Actimel", age: 135, job: "Pilot"),
            # Jobless :(
            (name: "Bob", surname: "Bob", age: 88), 
            (name: "Suzuki", surname: "Satoru", age: 45, job: "Salaryman");
        
        
        from Person get @Id, *;
        # RowAttribute --^   ^-- Gets all rows.

        from Person get @Id, job;
        #      Just a row ----^
    "#;

    let mut database = Database::new();

    let tokens = TokenIter::new(src);
    let commands = CommandIter::new(tokens);
    for command in commands
        .collect::<Result<Vec<_>, _>>()
        .expect("syntax error")
    {
        let output = match database.run_command(command).expect("run error") {
            CommandRunOutput::RowsInserted { identifier, count } => {
                format!("Inserted {count} rows into table \"{identifier}\".")
            }
            CommandRunOutput::TableCreated { identifier } => {
                format!("Table \"{identifier}\" created.")
            }
            CommandRunOutput::Selection { rows } => rows
                .into_iter()
                .map(|row| {
                    format!(
                        "{}\n",
                        row.into_iter()
                            .map(|value| value.to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                })
                .collect::<String>(),
        };

        println!("{output}");
    }
}
