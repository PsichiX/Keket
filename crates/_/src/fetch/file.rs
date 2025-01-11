use crate::{
    database::{path::AssetPath, reference::AssetRef},
    fetch::{AssetBytesAreReadyToProcess, AssetFetch},
};
use anput::world::World;
use std::{error::Error, path::PathBuf};

pub struct AssetFromFile;

#[derive(Debug, Default, Clone)]
pub struct FileAssetFetch {
    pub root: PathBuf,
}

impl FileAssetFetch {
    pub fn with_root(mut self, root: impl Into<PathBuf>) -> Self {
        self.root = root.into();
        self
    }
}

impl AssetFetch for FileAssetFetch {
    fn load_bytes(
        &mut self,
        reference: AssetRef,
        path: AssetPath,
        storage: &mut World,
    ) -> Result<(), Box<dyn Error>> {
        let file_path = self.root.join(path.path());
        let bytes = std::fs::read(&file_path)
            .map_err(|error| format!("Failed to load `{:?}` file bytes: {}", file_path, error))?;
        let bundle = (
            AssetBytesAreReadyToProcess(bytes),
            AssetFromFile,
            std::fs::metadata(&file_path)?,
            file_path,
        );
        storage.insert(reference.entity(), bundle)?;
        Ok(())
    }
}
