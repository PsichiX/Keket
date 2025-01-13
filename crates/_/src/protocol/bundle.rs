use crate::{database::handle::AssetHandle, protocol::AssetProtocol};
use anput::{bundle::Bundle, world::World};
use std::error::Error;

pub struct BundleAssetProtocol<B: Bundle> {
    name: String,
    #[allow(clippy::type_complexity)]
    processor: Box<dyn Fn(Vec<u8>) -> Result<B, Box<dyn Error>> + Send + Sync>,
}

impl<B: Bundle> BundleAssetProtocol<B> {
    pub fn new(
        name: impl ToString,
        processor: impl Fn(Vec<u8>) -> Result<B, Box<dyn Error>> + Send + Sync + 'static,
    ) -> Self {
        Self {
            name: name.to_string(),
            processor: Box::new(processor),
        }
    }
}

impl<B: Bundle> AssetProtocol for BundleAssetProtocol<B> {
    fn name(&self) -> &str {
        &self.name
    }

    fn process_bytes(
        &mut self,
        handle: AssetHandle,
        storage: &mut World,
        bytes: Vec<u8>,
    ) -> Result<(), Box<dyn Error>> {
        let bundle = (self.processor)(bytes)?;
        storage.insert(handle.entity(), bundle)?;
        Ok(())
    }
}
