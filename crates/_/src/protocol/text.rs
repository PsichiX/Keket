use crate::{database::handle::AssetHandle, protocol::AssetProtocol};
use anput::world::World;
use std::error::Error;

pub struct TextAssetProtocol;

impl AssetProtocol for TextAssetProtocol {
    fn name(&self) -> &str {
        "text"
    }

    fn process_bytes(
        &mut self,
        handle: AssetHandle,
        storage: &mut World,
        bytes: Vec<u8>,
    ) -> Result<(), Box<dyn Error>> {
        let text = std::str::from_utf8(&bytes)?.to_owned();
        storage.insert(handle.entity(), (text,))?;
        Ok(())
    }
}
