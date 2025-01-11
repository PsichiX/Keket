use crate::{database::reference::AssetRef, protocol::AssetProtocol};
use anput::world::World;
use std::error::Error;

pub struct BytesAssetProtocol;

impl AssetProtocol for BytesAssetProtocol {
    fn name(&self) -> &str {
        "bytes"
    }

    fn process_bytes(
        &mut self,
        reference: AssetRef,
        storage: &mut World,
        bytes: Vec<u8>,
    ) -> Result<(), Box<dyn Error>> {
        storage.insert(reference.entity(), (bytes,))?;
        Ok(())
    }
}
