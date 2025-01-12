use crate::{database::reference::AssetRef, protocol::AssetProtocol};
use anput::world::World;
use std::error::Error;

pub struct TextAssetProtocol;

impl AssetProtocol for TextAssetProtocol {
    fn name(&self) -> &str {
        "text"
    }

    fn process_bytes(
        &mut self,
        reference: AssetRef,
        storage: &mut World,
        bytes: Vec<u8>,
    ) -> Result<(), Box<dyn Error>> {
        let text = std::str::from_utf8(&bytes)?.to_owned();
        println!("=== TEXT: {}", text.len());
        storage.insert(reference.entity(), (text,))?;
        Ok(())
    }
}
