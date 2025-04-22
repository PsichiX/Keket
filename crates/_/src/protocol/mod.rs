pub mod bundle;
pub mod bytes;
pub mod group;
pub mod text;

use crate::{
    database::handle::AssetHandle, fetch::AssetBytesAreReadyToProcess,
    store::AssetBytesAreReadyToStore,
};
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
    fn process_asset_bytes(
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

    /// Produces raw byte data for an asset.
    ///
    /// This function is optional to override. It is called when the asset
    /// needs to be converted into raw byte data for storage or transmission.
    ///
    /// # Arguments
    /// - `handle`: The handle of the asset being processed.
    /// - `storage`: The world storage containing all asset-related data.
    ///
    /// # Returns
    /// - `Ok(Vec<u8>)` containing the raw byte data of the asset.
    /// - An error if producing bytes fails.
    ///
    /// # Default Implementation
    /// Returns an error indicating that the protocol does not support
    /// producing bytes.
    #[allow(unused_variables)]
    fn produce_bytes(
        &mut self,
        handle: AssetHandle,
        storage: &mut World,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        Err(format!(
            "Asset protocol `{}` does not support producing bytes.",
            self.name()
        )
        .into())
    }

    /// Produces an asset by first retrieving its raw byte data and then
    /// delegating to `produce_bytes`.
    ///
    /// This function performs the following:
    /// 1. calls `produce_bytes` to get the raw byte data for the asset.
    /// 2. Wraps the byte data in `AssetBytesAreReadyToStore` and stores it in the
    ///    entity associated with the asset handle.
    ///
    /// # Arguments
    /// - `handle`: The handle of the asset being processed.
    /// - `storage`: The world storage containing all asset-related data.
    ///
    /// # Returns
    /// - `Ok(())` on success.
    /// - An error wrapped in `Box<dyn Error>` if producing bytes fails.
    fn produce_asset_bytes(
        &mut self,
        handle: AssetHandle,
        storage: &mut World,
    ) -> Result<(), Box<dyn Error>> {
        let bytes = self.produce_bytes(handle, storage)?;
        storage.insert(handle.entity(), (AssetBytesAreReadyToStore(bytes),))?;
        Ok(())
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
