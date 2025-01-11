use crate::{database::reference::AssetRef, protocol::AssetProtocol};
use anput::world::World;
use std::error::Error;

pub struct ClosureAssetProtocol {
    name: String,
    #[allow(clippy::type_complexity)]
    processor:
        Box<dyn Fn(AssetRef, &mut World, Vec<u8>) -> Result<(), Box<dyn Error>> + Send + Sync>,
}

impl ClosureAssetProtocol {
    pub fn new(
        name: impl ToString,
        processor: impl Fn(AssetRef, &mut World, Vec<u8>) -> Result<(), Box<dyn Error>>
            + Send
            + Sync
            + 'static,
    ) -> Self {
        Self {
            name: name.to_string(),
            processor: Box::new(processor),
        }
    }
}

impl AssetProtocol for ClosureAssetProtocol {
    fn name(&self) -> &str {
        &self.name
    }

    fn process_bytes(
        &mut self,
        reference: AssetRef,
        storage: &mut World,
        bytes: Vec<u8>,
    ) -> Result<(), Box<dyn Error>> {
        (self.processor)(reference, storage, bytes)
    }
}
