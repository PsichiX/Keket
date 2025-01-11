use crate::{
    database::{
        path::AssetPath,
        reference::{AssetDependency, AssetRef},
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
        reference: AssetRef,
        storage: &mut World,
    ) -> Result<(), Box<dyn Error>> {
        let bytes = {
            let mut bytes =
                storage.component_mut::<true, AssetBytesAreReadyToProcess>(reference.entity())?;
            std::mem::take(&mut bytes.0)
        };
        let paths = std::str::from_utf8(&bytes)?
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| AssetPath::new(line.trim().to_owned()))
            .collect::<Vec<_>>();
        for path in &paths {
            let entity = storage.spawn((path.clone(), AssetAwaitsResolution))?;
            storage.relate::<true, _>(AssetDependency, reference.entity(), entity)?;
        }
        storage.insert(reference.entity(), (GroupAsset(paths),))?;
        Ok(())
    }
}
