pub mod collections;
pub mod container;
pub mod file;

use crate::database::{path::AssetPath, reference::AssetRef};
use anput::world::World;
use std::{
    error::Error,
    sync::{Arc, Mutex, RwLock},
};

pub struct AssetBytesAreLoading;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct AssetBytesAreReadyToProcess(pub Vec<u8>);

pub trait AssetFetch: Send + Sync {
    fn load_bytes(
        &mut self,
        path: AssetPath,
        storage: &mut World,
    ) -> Result<AssetRef, Box<dyn Error>>;

    #[allow(unused_variables)]
    fn maintain(&mut self, storage: &mut World) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}

impl<T: AssetFetch> AssetFetch for Arc<RwLock<T>> {
    fn load_bytes(
        &mut self,
        path: AssetPath,
        storage: &mut World,
    ) -> Result<AssetRef, Box<dyn Error>> {
        self.write()
            .map_err(|error| format!("{}", error))?
            .load_bytes(path, storage)
    }
}

impl<T: AssetFetch> AssetFetch for Arc<Mutex<T>> {
    fn load_bytes(
        &mut self,
        path: AssetPath,
        storage: &mut World,
    ) -> Result<AssetRef, Box<dyn Error>> {
        self.lock()
            .map_err(|error| format!("{}", error))?
            .load_bytes(path, storage)
    }
}
