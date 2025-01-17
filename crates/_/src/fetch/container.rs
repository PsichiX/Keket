use crate::{
    database::path::AssetPath,
    fetch::{AssetBytesAreReadyToProcess, AssetFetch},
};
use anput::bundle::DynamicBundle;
use std::{error::Error, sync::RwLock};

/// A trait that allows a partial fetch to load bytes from a source asynchronously.
/// The implementation of this trait is responsible for fetching the asset bytes
/// given a specific `AssetPath`.
///
/// This is typically used for containerized assets where fetching logic may be
/// partially defined or provided by a function.
pub trait ContainerPartialFetch: Send + Sync + 'static {
    /// Loads bytes for a given asset path.
    ///
    /// # Arguments
    /// - `path`: The path of the asset to fetch.
    ///
    /// # Returns
    /// - `Ok(Vec<u8>)`: The bytes associated with the asset.
    /// - `Err(Box<dyn Error>)`: An error in fetching the asset bytes.
    fn load_bytes(&mut self, path: AssetPath) -> Result<Vec<u8>, Box<dyn Error>>;
}

impl<F> ContainerPartialFetch for F
where
    F: FnMut(AssetPath) -> Result<Vec<u8>, Box<dyn Error>> + Send + Sync + 'static,
{
    fn load_bytes(&mut self, path: AssetPath) -> Result<Vec<u8>, Box<dyn Error>> {
        self(path)
    }
}

/// A marker component used to identify assets that have been loaded from
/// a container or other source, but the component itself doesn't carry
/// data about the asset.
pub struct AssetFromContainer;

/// The `ContainerAssetFetch` struct is responsible for fetching asset bytes from a
/// container-like system. It wraps a `ContainerPartialFetch` trait object, which
/// handles the actual fetching of asset bytes for a given path.
///
/// The container fetch logic is backed by a `RwLock` to allow mutable access
/// to the `ContainerPartialFetch` while ensuring thread safety.
pub struct ContainerAssetFetch<Partial: ContainerPartialFetch> {
    partial: RwLock<Partial>,
}

impl<Partial: ContainerPartialFetch> ContainerAssetFetch<Partial> {
    /// Creates a new `ContainerAssetFetch` instance with a specified partial fetch.
    ///
    /// # Arguments
    /// - `partial`: The partial fetcher that implements the `ContainerPartialFetch` trait.
    ///
    /// # Returns
    /// - A new `ContainerAssetFetch` instance with the provided partial fetcher.
    pub fn new(partial: Partial) -> Self {
        Self {
            partial: RwLock::new(partial),
        }
    }
}

impl<Partial: ContainerPartialFetch> AssetFetch for ContainerAssetFetch<Partial> {
    fn load_bytes(&self, path: AssetPath) -> Result<DynamicBundle, Box<dyn Error>> {
        let bytes = self
            .partial
            .write()
            .map_err(|error| format!("{}", error))?
            .load_bytes(path.clone())?;
        let mut bundle = DynamicBundle::default();
        let _ = bundle.add_component(AssetBytesAreReadyToProcess(bytes));
        let _ = bundle.add_component(AssetFromContainer);
        Ok(bundle)
    }
}
