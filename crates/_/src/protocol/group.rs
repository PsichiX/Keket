use crate::{
    database::{
        handle::{AssetDependency, AssetHandle},
        path::{AssetPath, AssetPathStatic},
    },
    fetch::{AssetAwaitsResolution, AssetBytesAreReadyToProcess},
    protocol::AssetProtocol,
};
use anput::world::World;
use std::error::Error;

/// Marker component for assets of the "group" type.
///
/// Group assets represent collections of other assets defined by a list of asset paths.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct GroupAsset;

/// Protocol implementation for handling "group" assets.
///
/// A "group" asset is a collection of paths to other assets, usually defined in text form.
pub struct GroupAssetProtocol;

impl AssetProtocol for GroupAssetProtocol {
    fn name(&self) -> &str {
        "group"
    }

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
        for line in std::str::from_utf8(&bytes)?
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty() || !line.starts_with('#') || !line.starts_with(';'))
        {
            let path = AssetPath::new(line.to_owned()).into_static();
            let entity = if let Some(entity) = storage.find_by::<true, _>(&path) {
                entity
            } else {
                storage.spawn((path.clone(), AssetAwaitsResolution))?
            };
            storage.relate::<true, _>(AssetDependency, handle.entity(), entity)?;
        }
        storage.insert(handle.entity(), (GroupAsset,))?;
        Ok(())
    }

    fn produce_bytes(
        &mut self,
        handle: AssetHandle,
        storage: &mut World,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut lines = String::default();
        for (_, _, entity) in storage.relations_outgoing::<true, AssetDependency>(handle.entity()) {
            let path = storage.component::<true, AssetPathStatic>(entity)?;
            lines.push_str(path.content());
            lines.push('\n');
        }
        Ok(lines.trim().as_bytes().to_owned())
    }
}
