pub mod bundle;
pub mod bytes;
pub mod group;
pub mod text;

use crate::{database::handle::AssetHandle, fetch::AssetBytesAreReadyToProcess};
use anput::world::World;
use std::error::Error;

/// Trait defining the protocol for processing and handling assets.
///
/// Implementers of this trait can define custom behavior for handling assets
/// at various stages of the asset lifecycle.
pub trait AssetProtocol: Send + Sync {
    /// Returns the name of the asset protocol.
    ///
    /// This name can be used for identification or debugging purposes.
    fn name(&self) -> &str;

    /// Processes the raw byte data of an asset.
    ///
    /// This function is optional to override. It is called with the raw byte
    /// data of an asset, allowing implementers to transform or prepare the data
    /// for usage.
    ///
    /// # Arguments
    /// - `handle`: The handle of the asset being processed.
    /// - `storage`: The world storage containing all asset-related data.
    /// - `bytes`: The raw bytes representing the asset's data.
    ///
    /// # Returns
    /// - `Ok(())` on success.
    /// - An error wrapped in `Box<dyn Error>` if processing fails.
    ///
    /// # Default Implementation
    /// Does nothing and returns `Ok(())`.
    #[allow(unused_variables)]
    fn process_bytes(
        &mut self,
        handle: AssetHandle,
        storage: &mut World,
        bytes: Vec<u8>,
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    /// Processes an asset by first retrieving its raw byte data and then
    /// delegating to `process_bytes`.
    ///
    /// This function performs the following:
    /// 1. Retrieves the `AssetBytesAreReadyToProcess` component associated
    ///    with the asset's entity.
    /// 2. Extracts the raw byte data from the component.
    /// 3. Removes the `AssetBytesAreReadyToProcess` component from the entity.
    /// 4. Passes the byte data to `process_bytes` for further processing.
    ///
    /// # Arguments
    /// - `handle`: The handle of the asset being processed.
    /// - `storage`: The world storage containing all asset-related data.
    ///
    /// # Returns
    /// - `Ok(())` on success.
    /// - An error wrapped in `Box<dyn Error>` if processing fails.
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

    /// Maintains protocol state.
    ///
    /// Can be used for handling periodic or deferred operations.
    ///
    /// # Arguments
    /// - `storage`: The world storage where asset state is maintained.
    ///
    /// # Returns
    /// - `Ok(())` if maintenance succeeds, otherwise an error.
    #[allow(unused_variables)]
    fn maintain(&mut self, storage: &mut World) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}
