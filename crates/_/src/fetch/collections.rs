use crate::{
    database::{path::AssetPath, reference::AssetRef},
    fetch::{AssetBytesAreReadyToProcess, AssetFetch},
};
use anput::world::World;
use std::{
    collections::{BTreeMap, HashMap},
    error::Error,
};

pub struct AssetFromCollection;

impl AssetFetch for Vec<(String, Vec<u8>)> {
    fn load_bytes(
        &mut self,
        path: AssetPath,
        storage: &mut World,
    ) -> Result<AssetRef, Box<dyn Error>> {
        let bytes = self
            .iter()
            .find(|(p, _)| p == path.path())
            .ok_or_else(|| -> Box<dyn Error> {
                format!("Missing map key: `{}`", path.path()).into()
            })?
            .1
            .clone();
        let bundle = (
            path.into_static(),
            AssetBytesAreReadyToProcess(bytes),
            AssetFromCollection,
        );
        let entity = storage.spawn(bundle)?;
        Ok(AssetRef::new(entity))
    }
}

impl AssetFetch for HashMap<String, Vec<u8>> {
    fn load_bytes(
        &mut self,
        path: AssetPath,
        storage: &mut World,
    ) -> Result<AssetRef, Box<dyn Error>> {
        let bytes = self
            .get(path.path())
            .ok_or_else(|| -> Box<dyn Error> {
                format!("Missing map key: `{}`", path.path()).into()
            })
            .cloned()?;
        let bundle = (
            path.into_static(),
            AssetBytesAreReadyToProcess(bytes),
            AssetFromCollection,
        );
        let entity = storage.spawn(bundle)?;
        Ok(AssetRef::new(entity))
    }
}

impl AssetFetch for BTreeMap<String, Vec<u8>> {
    fn load_bytes(
        &mut self,
        path: AssetPath,
        storage: &mut World,
    ) -> Result<AssetRef, Box<dyn Error>> {
        let bytes = self
            .get(path.path())
            .ok_or_else(|| -> Box<dyn Error> {
                format!("Missing map key: `{}`", path.path()).into()
            })
            .cloned()?;
        let bundle = (
            path.into_static(),
            AssetBytesAreReadyToProcess(bytes),
            AssetFromCollection,
        );
        let entity = storage.spawn(bundle)?;
        Ok(AssetRef::new(entity))
    }
}
