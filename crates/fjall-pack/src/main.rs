use fjall::{Config, PersistMode};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let keyspace = Config::new("./resources/database/").open()?;
    let items = keyspace.open_partition("assets", Default::default())?;
    items.insert("lorem.txt", std::fs::read("./resources/lorem.txt")?)?;
    items.insert("person.json", std::fs::read("./resources/person.json")?)?;
    items.insert("trash.bin", std::fs::read("./resources/trash.bin")?)?;
    items.insert("group.txt", std::fs::read("./resources/group.txt")?)?;
    keyspace.persist(PersistMode::SyncAll)?;
    Ok(())
}
