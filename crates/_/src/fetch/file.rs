use crate::{
    database::{path::AssetPath, reference::AssetRef},
    fetch::{deferred::DeferredAssetJob, AssetBytesAreReadyToProcess, AssetFetch},
};
use anput::world::World;
use std::{error::Error, fs::Metadata, path::PathBuf};

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

impl DeferredAssetJob for FileAssetFetch {
    type Result = (
        AssetBytesAreReadyToProcess,
        AssetFromFile,
        Metadata,
        PathBuf,
    );

    fn execute(&self, path: AssetPath) -> Result<Self::Result, String> {
        let file_path = self.root.join(path.path());
        let bytes = std::fs::read(&file_path)
            .map_err(|error| format!("Failed to load `{:?}` file bytes: {}", file_path, error))?;
        let metadata = std::fs::metadata(&file_path).map_err(|error| {
            format!("Failed to read `{:?}` file metadata: {}", file_path, error)
        })?;
        Ok((
            AssetBytesAreReadyToProcess(bytes),
            AssetFromFile,
            metadata,
            file_path,
        ))
    }
}
