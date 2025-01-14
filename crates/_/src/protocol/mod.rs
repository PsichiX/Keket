pub mod bundle;
pub mod bytes;
pub mod group;
pub mod text;

use crate::{
    database::{handle::AssetHandle, path::AssetPath},
    fetch::AssetBytesAreReadyToProcess,
};
use anput::{entity::Entity, query::Include, world::World};
use std::error::Error;

pub trait AssetProtocol: Send + Sync {
    fn name(&self) -> &str;

    #[allow(unused_variables)]
    fn process_bytes(
        &mut self,
        handle: AssetHandle,
        storage: &mut World,
        bytes: Vec<u8>,
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn process_asset(
        &mut self,
        handle: AssetHandle,
        storage: &mut World,
    ) -> Result<(), Box<dyn Error>> {
        let bytes = {
            let mut bytes =
                storage.component_mut::<true, AssetBytesAreReadyToProcess>(handle.entity())?;
            std::mem::take(&mut bytes.0)
        };
        storage.remove::<(AssetBytesAreReadyToProcess,)>(handle.entity())?;
        self.process_bytes(handle, storage, bytes)
    }

    fn process_assets(&mut self, storage: &mut World) -> Result<(), Box<dyn Error>> {
        let to_process = storage
            .query::<true, (Entity, &AssetPath, Include<AssetBytesAreReadyToProcess>)>()
            .filter(|(_, path, _)| path.protocol() == self.name())
            .map(|(entity, _, _)| AssetHandle::new(entity))
            .collect::<Vec<_>>();
        for handle in to_process {
            self.process_asset(handle, storage)?;
        }
        Ok(())
    }
}
