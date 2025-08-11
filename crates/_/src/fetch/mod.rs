pub mod collections;
pub mod container;
pub mod deferred;
pub mod extract;
pub mod fallback;
pub mod file;
pub mod future;
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

/// Marker component used to signify that the asset fetch for an asset is
/// asynchronous and it's pending completion.
pub struct AssetAwaitsAsyncFetch;

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

pub(crate) struct AssetFetchEngine {
    fetch: Box<dyn AssetFetch>,
}

impl AssetFetchEngine {
    pub fn new(fetch: impl AssetFetch) -> Self {
        Self {
            fetch: Box::new(fetch),
        }
    }

    pub fn into_inner(self) -> Box<dyn AssetFetch> {
        self.fetch
    }

    pub fn load_bytes(
        &self,
        handle: AssetHandle,
        path: AssetPath,
        storage: &mut World,
    ) -> Result<(), Box<dyn Error>> {
        let result = self.fetch.load_bytes(path.clone());
        if result.is_err()
            && let Ok(mut bindings) =
                storage.component_mut::<true, AssetEventBindings>(handle.entity())
        {
            bindings.dispatch(AssetEvent {
                handle,
                kind: AssetEventKind::BytesFetchingFailed,
                path: path.into_static(),
            })?;
        }
        storage.insert(handle.entity(), result?)?;
        Ok(())
    }

    pub fn maintain(&mut self, storage: &mut World) -> Result<(), Box<dyn Error>> {
        self.fetch.maintain(storage)
    }
}
