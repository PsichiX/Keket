pub mod collections;
pub mod container;
pub mod deferred;
pub mod fallback;
pub mod file;
#[cfg(feature = "hotreload")]
pub mod hotreload;
pub mod router;

use crate::database::{
    events::{AssetEvent, AssetEventBindings, AssetEventKind},
    handle::AssetHandle,
    path::AssetPath,
};
use anput::{bundle::DynamicBundle, world::World};
use std::error::Error;

pub struct AssetAwaitsResolution;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct AssetBytesAreReadyToProcess(pub Vec<u8>);

pub trait AssetFetch: Send + Sync + 'static {
    fn load_bytes(&self, path: AssetPath) -> Result<DynamicBundle, Box<dyn Error>>;

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

    pub fn maintain(&mut self, storage: &mut World) -> Result<(), Box<dyn Error>> {
        self.fetch.maintain(storage)
    }
}
