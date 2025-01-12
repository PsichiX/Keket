use keket::{database::path::AssetPath, fetch::container::ContainerPartialFetch};
use redb::{Database, TableDefinition};
use std::error::Error;

pub mod third_party {
    pub use redb;
}

pub struct RedbContainerPartialFetch {
    database: Database,
    default_table_name: String,
}

impl RedbContainerPartialFetch {
    pub fn new(database: Database, default_table_name: impl ToString) -> Self {
        Self {
            database,
            default_table_name: default_table_name.to_string(),
        }
    }
}

impl ContainerPartialFetch for RedbContainerPartialFetch {
    fn part(&mut self, path: AssetPath) -> Result<Vec<u8>, Box<dyn Error>> {
        let transaction = self.database.begin_read()?;
        let table_name = path.try_meta().unwrap_or(self.default_table_name.as_str());
        let table_definition = TableDefinition::<String, Vec<u8>>::new(table_name);
        let table = transaction.open_table(table_definition)?;
        let access = table.get(path.path().to_owned())?;
        let bytes = access.map(|access| access.value()).unwrap_or_default();
        Ok(bytes)
    }
}
