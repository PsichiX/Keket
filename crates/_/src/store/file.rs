use crate::{database::path::AssetPath, store::AssetStore};
use anput::bundle::DynamicBundle;
use std::{error::Error, path::PathBuf};

fn save_file_bytes(file_path: PathBuf, bytes: Vec<u8>) -> Result<DynamicBundle, Box<dyn Error>> {
    std::fs::create_dir_all(file_path.parent().unwrap())?;
    std::fs::write(&file_path, bytes)?;
    Ok(DynamicBundle::default())
}

/// Implementation of the `AssetStore` trait that saves assets to the file
/// system using absolute paths.
#[derive(Debug, Default, Clone)]
pub struct AbsoluteFileAssetStore;

impl AssetStore for AbsoluteFileAssetStore {
    fn save_bytes(&self, path: AssetPath, bytes: Vec<u8>) -> Result<DynamicBundle, Box<dyn Error>> {
        save_file_bytes(PathBuf::from(path.path()), bytes)
    }
}

/// Implementation of the `AssetStore` trait that saves assets to the file
/// system using a specified root directory.
#[derive(Debug, Default, Clone)]
pub struct FileAssetStore {
    pub root: PathBuf,
}

impl FileAssetStore {
    /// Sets the root directory for file-based asset storage.
    ///
    /// # Arguments
    /// - `root`: The root path to set for storing assets.
    ///
    /// # Returns
    /// - A modified `FileAssetStore` instance with the new root directory.
    pub fn with_root(mut self, root: impl Into<PathBuf>) -> Self {
        self.root = root.into();
        self
    }
}

impl AssetStore for FileAssetStore {
    fn save_bytes(&self, path: AssetPath, bytes: Vec<u8>) -> Result<DynamicBundle, Box<dyn Error>> {
        let path = self.root.join(path.path());
        save_file_bytes(path, bytes)
    }
}
