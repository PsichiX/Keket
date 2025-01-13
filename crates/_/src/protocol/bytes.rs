use crate::{database::handle::AssetHandle, protocol::AssetProtocol};
use anput::world::World;
use std::error::Error;

pub struct BytesAssetProtocol;

impl AssetProtocol for BytesAssetProtocol {
    fn name(&self) -> &str {
        "bytes"
    }

    fn process_bytes(
        &mut self,
        handle: AssetHandle,
        storage: &mut World,
        bytes: Vec<u8>,
    ) -> Result<(), Box<dyn Error>> {
        storage.insert(handle.entity(), (bytes,))?;
        Ok(())
    }
}
