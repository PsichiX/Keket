use crate::{
    database::{path::AssetPath, reference::AssetRef},
    fetch::{AssetBytesAreReadyToProcess, AssetFetch},
};
use anput::world::World;
use std::error::Error;

pub trait ContainerPartialFetch: Send + Sync {
    fn part(&mut self, path: AssetPath) -> Result<Vec<u8>, Box<dyn Error>>;
}

pub struct AssetFromContainer;

pub struct ContainerAssetFetch<Partial: ContainerPartialFetch> {
    partial: Partial,
}

impl<Partial: ContainerPartialFetch> ContainerAssetFetch<Partial> {
    pub fn new(partial: Partial) -> Self {
        Self { partial }
    }
}

impl<Partial: ContainerPartialFetch> AssetFetch for ContainerAssetFetch<Partial> {
    fn load_bytes(
        &mut self,
        path: AssetPath,
        storage: &mut World,
    ) -> Result<AssetRef, Box<dyn Error>> {
        let bytes = self.partial.part(path.clone())?;
        let bundle = (
            path.into_static(),
            AssetBytesAreReadyToProcess(bytes),
            AssetFromContainer,
        );
        let entity = storage.spawn(bundle)?;
        Ok(AssetRef::new(entity))
    }
}
