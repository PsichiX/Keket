use crate::{
    database::path::AssetPath,
    fetch::{AssetBytesAreReadyToProcess, AssetFetch},
};
use anput::bundle::DynamicBundle;
use std::{error::Error, path::PathBuf};

fn load_file_bundle(file_path: PathBuf) -> Result<DynamicBundle, Box<dyn Error>> {
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

/// Marker component for assets that originate from files.
pub struct AssetFromFile;

/// An implementation of the `AssetFetch` trait that loads assets from the file
/// system using absolute paths.
#[derive(Debug, Default, Clone)]
pub struct AbsoluteFileAssetFetch;

impl AssetFetch for AbsoluteFileAssetFetch {
    fn load_bytes(&self, path: AssetPath) -> Result<DynamicBundle, Box<dyn Error>> {
        load_file_bundle(PathBuf::from(path.path()))
    }
}

/// An implementation of the `AssetFetch` trait that loads assets from the file system.
#[derive(Debug, Default, Clone)]
pub struct FileAssetFetch {
    pub root: PathBuf,
}

impl FileAssetFetch {
    /// Sets the root directory for file-based asset fetching.
    ///
    /// # Arguments
    /// - `root`: The root path to set for fetching assets.
    ///
    /// # Returns
    /// - A modified `FileAssetFetch` instance with the new root directory.
    pub fn with_root(mut self, root: impl Into<PathBuf>) -> Self {
        self.root = root.into();
        self
    }
}

impl AssetFetch for FileAssetFetch {
    fn load_bytes(&self, path: AssetPath) -> Result<DynamicBundle, Box<dyn Error>> {
        load_file_bundle(self.root.join(path.path()))
    }
}
