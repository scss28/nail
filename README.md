# nail
nail is a small query language. It allows for creating tables, inserting data and retrieving data.
## example
```
# To create a table simply:
new table Person 
    name: str, 
    surname: str, 
    age: int,
    job: str?; # optional column

# To insert some rows simply
insert Person (
    name: "Joe", surname: "Kowalski", age: 35, job: "Police Officer";
    name: "Croki", surname: "Actimel", age: 135, job: "Pilot";
    # No job :(
    name: "Bob", surname: "Bob", age: 9000;
    name: "Suzuki", surname: "Satoru", age: 45, job: "Salaryman";
);

from Person get @Id, *;
# RowAttribute --^   ^-- Gets all rows.

from Person get @Id, job as "Jabba job";
#   Just a column ----^
```
You can run this example by cloning the repository and running:\
`cargo run --example simple`
