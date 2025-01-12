use redb::{Database, TableDefinition};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let _ = std::fs::remove_file("./resources/database.redb");
    let db = Database::create("./resources/database.redb")?;
    let transaction = db.begin_write()?;
    {
        let table_definition = TableDefinition::<String, Vec<u8>>::new("assets");
        let mut table = transaction.open_table(table_definition)?;
        table.insert(
            "lorem.txt".to_owned(),
            std::fs::read("./resources/lorem.txt")?,
        )?;
        table.insert(
            "person.json".to_owned(),
            std::fs::read("./resources/person.json")?,
        )?;
        table.insert(
            "trash.bin".to_owned(),
            std::fs::read("./resources/trash.bin")?,
        )?;
        table.insert(
            "group.txt".to_owned(),
            std::fs::read("./resources/group.txt")?,
        )?;
    }
    transaction.commit()?;
    Ok(())
}
