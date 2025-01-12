use fjall::Keyspace;
use keket::{database::path::AssetPath, fetch::container::ContainerPartialFetch};
use std::error::Error;

pub mod third_party {
    pub use fjall;
}

pub struct FjallContainerPartialFetch {
    keyspace: Keyspace,
    default_partition_name: String,
}

impl FjallContainerPartialFetch {
    pub fn new(keyspace: Keyspace, default_partition_name: impl ToString) -> Self {
        Self {
            keyspace,
            default_partition_name: default_partition_name.to_string(),
        }
    }
}

impl ContainerPartialFetch for FjallContainerPartialFetch {
    fn part(&mut self, path: AssetPath) -> Result<Vec<u8>, Box<dyn Error>> {
        let partition_name = path
            .try_meta()
            .unwrap_or(self.default_partition_name.as_str());
        let items = self
            .keyspace
            .open_partition(partition_name, Default::default())?;
        let bytes = items
            .get(path.path())?
            .map(|slice| slice.to_vec())
            .unwrap_or_default();
        Ok(bytes)
    }
}
