use crate::{
    database::path::AssetPath,
    fetch::{AssetBytesAreReadyToProcess, AssetFetch},
};
use anput::bundle::DynamicBundle;
use std::{error::Error, path::PathBuf};

fn load_file_bundle(file_path: PathBuf) -> Result<DynamicBundle, Box<dyn Error>> {
    let bytes = std::fs::read(&file_path)
        .map_err(|error| format!("Failed to load `{:?}` file bytes: {}", file_path, error))?;
    let metadata = std::fs::metadata(&file_path)?;
    let mut bundle = DynamicBundle::default();
    bundle
        .add_component(AssetBytesAreReadyToProcess(bytes))
        .map_err(|_| {
            format!(
                "Failed to add bytes to bundle for asset file: {:?}",
                file_path
            )
        })?;
    bundle.add_component(AssetFromFile).map_err(|_| {
        format!(
            "Failed to add marker to bundle for asset file: {:?}",
            file_path
        )
    })?;
    bundle.add_component(metadata).map_err(|_| {
        format!(
            "Failed to add metadata to bundle for asset file: {:?}",
            file_path
        )
    })?;
    bundle.add_component(file_path.clone()).map_err(|_| {
        format!(
            "Failed to add file system path to bundle for asset file: {:?}",
            file_path
        )
    })?;
    Ok(bundle)
}

/// Marker component for assets that originate from files.
pub struct AssetFromFile;

/// An implementation of the `AssetFetch` trait that loads assets from the
/// file system using absolute paths.
#[derive(Debug, Default, Clone)]
pub struct AbsoluteFileAssetFetch;

impl AssetFetch for AbsoluteFileAssetFetch {
    fn load_bytes(&self, path: AssetPath) -> Result<DynamicBundle, Box<dyn Error>> {
        load_file_bundle(PathBuf::from(path.path()))
    }
}

/// An implementation of the `AssetFetch` trait that loads assets from the
/// file system using specified root path.
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
