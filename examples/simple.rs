use nail::prelude::*;

fn main() {
    let src = br#"
        # To create a table simply:
        new table Person 
            Name: str, 
            Surname: str, 
            Age: int,
            KnowsKungFu: bool,
            Job: str?; # optional column

        # To insert a row simply:
        insert Person
            Name: "Joe",
            Surname: "Kowalski",
            Age: 35,
            KnowsKungFu: true,
            Job: "Police Officer";

        # To insert multiple rows simply:
        insert Person {
            Name: "Croki",
            Surname: "Actimel",
            Age: 135, 
            KnowsKungFu: false, 
            Job: "Pilot";
            
            # No job :(
            Name: "Bob",
            Surname: "Bob",
            Age: 9000,
            KnowsKungFu: false;

            Name: "Suzuki",
            Surname: "Satoru",
            Age: 45,
            KnowsKungFu: false,
            Job: "Salaryman";
        };

        # Gets all rows.
        get Person;
        get Person select Surname, Job where Age > 45;

        remove Person where KnowsKungFu;
        get Person;
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
