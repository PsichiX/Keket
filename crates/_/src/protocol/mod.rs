pub mod bundle;
pub mod bytes;
pub mod closure;
pub mod text;

use crate::{
    database::{path::AssetPath, reference::AssetRef},
    fetch::AssetBytesAreReadyToProcess,
};
use anput::{entity::Entity, query::Include, world::World};
use std::error::Error;

pub trait AssetProtocol: Send + Sync {
    fn name(&self) -> &str;

    fn process_bytes(
        &mut self,
        reference: AssetRef,
        storage: &mut World,
        bytes: Vec<u8>,
    ) -> Result<(), Box<dyn Error>>;

    fn process_asset(
        &mut self,
        reference: AssetRef,
        storage: &mut World,
    ) -> Result<(), Box<dyn Error>> {
        let bytes = {
            let mut bytes =
                storage.component_mut::<true, AssetBytesAreReadyToProcess>(reference.entity())?;
            std::mem::take(&mut bytes.0)
        };
        self.process_bytes(reference, storage, bytes)
    }

    fn process_assets(&mut self, storage: &mut World) -> Result<(), Box<dyn Error>> {
        let to_process = storage
            .query::<true, (Entity, &AssetPath, Include<AssetBytesAreReadyToProcess>)>()
            .filter(|(_, path, _)| path.protocol() == self.name())
            .map(|(entity, _, _)| AssetRef::new(entity))
            .collect::<Vec<_>>();
        for reference in to_process {
            self.process_asset(reference, storage)?;
        }
        Ok(())
    }
}
