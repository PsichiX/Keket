use crate::{
    database::path::AssetPath,
    fetch::{AssetBytesAreReadyToProcess, AssetFetch},
};
use anput::bundle::DynamicBundle;
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
    fn load_bytes(&self, path: AssetPath) -> Result<DynamicBundle, Box<dyn Error>> {
        let file_path = self.root.join(path.path());
        let bytes = std::fs::read(&file_path)
            .map_err(|error| format!("Failed to load `{:?}` file bytes: {}", file_path, error))?;
        let mut bundle = DynamicBundle::default();
        bundle
            .add_component(AssetBytesAreReadyToProcess(bytes))
            .ok()
            .unwrap();
        bundle.add_component(AssetFromFile).ok().unwrap();
        bundle
            .add_component(std::fs::metadata(&file_path)?)
            .ok()
            .unwrap();
        bundle.add_component(file_path).ok().unwrap();
        Ok(bundle)
    }
}
