use keket::{database::path::AssetPath, fetch::container::ContainerPartialFetch};
use redb::{Database, ReadableDatabase, TableDefinition};
use std::error::Error;

pub mod third_party {
    pub use redb;
}

/// `RedbContainerPartialFetch` represents an asset fetcher that retrieves asset data
/// stored in a Redb database.
/// The fetcher uses the asset's `AssetPath` to find the corresponding asset in the database,
/// reading the data from a specified table in the Redb database.
pub struct RedbContainerPartialFetch {
    database: Database,
    default_table_name: String,
}

impl RedbContainerPartialFetch {
    /// Creates a new `RedbContainerPartialFetch` instance using the provided database and default table name.
    ///
    /// # Arguments
    /// - `database`: An instance of the Redb `Database` to use for querying.
    /// - `default_table_name`: A string representing the default table name to use for querying.
    ///
    /// # Returns
    /// - `Self`: A new `RedbContainerPartialFetch` initialized with the given database and table name.
    pub fn new(database: Database, default_table_name: impl ToString) -> Self {
        Self {
            database,
            default_table_name: default_table_name.to_string(),
        }
    }
}

impl ContainerPartialFetch for RedbContainerPartialFetch {
    fn load_bytes(&mut self, path: AssetPath) -> Result<Vec<u8>, Box<dyn Error>> {
        let transaction = self.database.begin_read()?;
        let table_name = path.try_meta().unwrap_or(self.default_table_name.as_str());
        let table_definition = TableDefinition::<String, Vec<u8>>::new(table_name);
        let table = transaction.open_table(table_definition)?;
        let access = table.get(path.path().to_owned())?;
        let bytes = access.map(|access| access.value()).unwrap_or_default();
        Ok(bytes)
    }
}
