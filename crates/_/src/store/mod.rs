pub mod file;
pub mod future;

use crate::database::{
    events::{AssetEvent, AssetEventBindings, AssetEventKind},
    handle::AssetHandle,
    path::AssetPath,
};
use anput::{bundle::DynamicBundle, world::World};
use std::error::Error;

/// Marker type for assets that are awaiting storing.
pub struct AssetAwaitsStoring;

/// Represents raw byte data that is ready to be stored.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct AssetBytesAreReadyToStore(pub Vec<u8>);

/// Marker component used to signify that the asset store for an asset is
/// asynchronous and it's pending completion.
pub struct AssetAwaitsAsyncStore;

/// Defines the interface for storing asset data to an external source.
pub trait AssetStore: Send + Sync + 'static {
    /// Saves the raw bytes of an asset given its path.
    ///
    /// # Arguments
    /// - `path`: The path to the asset.
    /// - `bytes`: The byte data to be saved.
    ///
    /// # Returns
    /// - A `DynamicBundle` containing additional asset data or an error if saving fails.
    fn save_bytes(&self, path: AssetPath, bytes: Vec<u8>) -> Result<DynamicBundle, Box<dyn Error>>;

    /// Maintains the store's state.
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

pub(crate) struct AssetStoreEngine {
    store: Box<dyn AssetStore>,
}

impl AssetStoreEngine {
    pub fn new(store: impl AssetStore) -> Self {
        Self {
            store: Box::new(store),
        }
    }

    pub fn into_inner(self) -> Box<dyn AssetStore> {
        self.store
    }

    pub fn save_bytes(
        &self,
        handle: AssetHandle,
        path: AssetPath,
        bytes: Vec<u8>,
        storage: &mut World,
    ) -> Result<(), Box<dyn Error>> {
        let result = self.store.save_bytes(path.clone(), bytes);
        if result.is_err() {
            if let Ok(mut bindings) =
                storage.component_mut::<true, AssetEventBindings>(handle.entity())
            {
                bindings.dispatch(AssetEvent {
                    handle,
                    kind: AssetEventKind::BytesStoringFailed,
                    path: path.into_static(),
                })?;
            }
        }
        storage.insert(handle.entity(), result?)?;
        Ok(())
    }

    pub fn maintain(&mut self, storage: &mut World) -> Result<(), Box<dyn Error>> {
        self.store.maintain(storage)
    }
}
