use crate::{database::reference::AssetRef, protocol::AssetProtocol};
use anput::{component::Component, world::World};
use serde::de::DeserializeOwned;
use std::error::Error;

pub struct SerdeAssetProtocol<T: Component + DeserializeOwned> {
    name: String,
    #[allow(clippy::type_complexity)]
    deserializer: Box<dyn Fn(Vec<u8>) -> Result<T, Box<dyn Error>> + Send + Sync>,
}

impl<T: Component + DeserializeOwned> SerdeAssetProtocol<T> {
    pub fn new(
        name: impl ToString,
        deserializer: impl Fn(Vec<u8>) -> Result<T, Box<dyn Error>> + Send + Sync + 'static,
    ) -> Self {
        Self {
            name: name.to_string(),
            deserializer: Box::new(deserializer),
        }
    }
}

impl<T: Component + DeserializeOwned> AssetProtocol for SerdeAssetProtocol<T> {
    fn name(&self) -> &str {
        &self.name
    }

    fn process_bytes(
        &mut self,
        reference: AssetRef,
        storage: &mut World,
        bytes: Vec<u8>,
    ) -> Result<(), Box<dyn Error>> {
        let content = (self.deserializer)(bytes)?;
        storage.insert(reference.entity(), (content,))?;
        Ok(())
    }
}
