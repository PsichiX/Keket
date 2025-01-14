use crate::{
    database::path::AssetPath,
    fetch::{AssetBytesAreReadyToProcess, AssetFetch},
};
use anput::bundle::DynamicBundle;
use std::{error::Error, sync::RwLock};

pub trait ContainerPartialFetch: Send + Sync + 'static {
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

pub struct AssetFromContainer;

pub struct ContainerAssetFetch<Partial: ContainerPartialFetch> {
    partial: RwLock<Partial>,
}

impl<Partial: ContainerPartialFetch> ContainerAssetFetch<Partial> {
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
