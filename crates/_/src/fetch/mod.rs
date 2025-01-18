pub mod collections;
pub mod container;
pub mod deferred;
pub mod fallback;
pub mod file;
#[cfg(feature = "hotreload")]
pub mod hotreload;
pub mod rewrite;
pub mod router;

use crate::database::{
    events::{AssetEvent, AssetEventBindings, AssetEventKind},
    handle::AssetHandle,
    path::AssetPath,
};
use anput::{bundle::DynamicBundle, world::World};
use std::error::Error;

/// Marker type for assets that are awaiting resolution of their path.
pub struct AssetAwaitsResolution;

/// Represents raw byte data that is ready to be processed.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct AssetBytesAreReadyToProcess(pub Vec<u8>);

/// Defines the interface for fetching asset data from an external source.
pub trait AssetFetch: Send + Sync + 'static {
    /// Loads the raw bytes of an asset given its path.
    ///
    /// # Arguments
    /// - `path`: The path to the asset.
    ///
    /// # Returns
    /// - A `DynamicBundle` containing the asset data or an error if loading fails.
    fn load_bytes(&self, path: AssetPath) -> Result<DynamicBundle, Box<dyn Error>>;

    /// Maintains the fetcher's state.
    ///
    /// Can be used for handling periodic or deferred operations.
    ///
    /// # Arguments
    /// - `storage`: The world storage where asset state is maintained.
    ///
    /// # Returns
    /// - `Ok(())` if maintenance succeeds, otherwise an error.
    #[allow(unused_variables)]
    fn maintain(&mut self, storage: &mut World) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}

/// Core engine that integrates the asset fetching system with the world state.
pub(crate) struct AssetFetchEngine {
    fetch: Box<dyn AssetFetch>,
}

impl AssetFetchEngine {
    /// Creates a new `AssetFetchEngine` with the specified fetcher.
    ///
    /// # Arguments
    /// - `fetch`: An instance of a type implementing the `AssetFetch` trait.
    ///
    /// # Returns
    /// - A new `AssetFetchEngine` instance.
    pub fn new(fetch: impl AssetFetch) -> Self {
        Self {
            fetch: Box::new(fetch),
        }
    }

    /// Consumes the engine and returns the inner fetcher.
    ///
    /// # Returns
    /// - A boxed instance of the fetcher.
    pub fn into_inner(self) -> Box<dyn AssetFetch> {
        self.fetch
    }

    /// Initiates the loading of asset bytes for the specified handle and path.
    ///
    /// # Arguments
    /// - `handle`: The handle representing the target asset.
    /// - `path`: The path to the asset being loaded.
    /// - `storage`: The world storage where asset state is maintained.
    ///
    /// # Returns
    /// - `Ok(())` if loading is initiated successfully, otherwise an error.
    pub fn load_bytes(
        &self,
        handle: AssetHandle,
        path: AssetPath,
        storage: &mut World,
    ) -> Result<(), Box<dyn Error>> {
        let result = self.fetch.load_bytes(path);
        if result.is_err() {
            if let Ok(mut bindings) =
                storage.component_mut::<true, AssetEventBindings>(handle.entity())
            {
                bindings.dispatch(AssetEvent {
                    handle,
                    kind: AssetEventKind::BytesFetchingFailed,
                })?;
            }
        }
        storage.insert(handle.entity(), result?)?;
        Ok(())
    }

    /// Maintains the fetcher and its associated state.
    ///
    /// # Arguments
    /// - `storage`: The world storage where asset state is maintained.
    ///
    /// # Returns
    /// - `Ok(())` if maintenance succeeds, otherwise an error.
    pub fn maintain(&mut self, storage: &mut World) -> Result<(), Box<dyn Error>> {
        self.fetch.maintain(storage)
    }
}
