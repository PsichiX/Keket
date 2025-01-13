use crate::{
    database::{
        handle::{AssetDependency, AssetHandle},
        path::AssetPath,
    },
    fetch::{AssetAwaitsResolution, AssetBytesAreReadyToProcess},
    protocol::AssetProtocol,
};
use anput::world::World;
use std::error::Error;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct GroupAsset(pub Vec<AssetPath<'static>>);

pub struct GroupAssetProtocol;

impl AssetProtocol for GroupAssetProtocol {
    fn name(&self) -> &str {
        "group"
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
        let paths = std::str::from_utf8(&bytes)?
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| AssetPath::new(line.trim().to_owned()))
            .collect::<Vec<_>>();
        for path in &paths {
            let entity = storage.spawn((path.clone(), AssetAwaitsResolution))?;
            storage.relate::<true, _>(AssetDependency, handle.entity(), entity)?;
        }
        storage.insert(handle.entity(), (GroupAsset(paths),))?;
        Ok(())
    }
}
