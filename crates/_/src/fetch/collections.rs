use crate::{
    database::path::AssetPath,
    fetch::{AssetBytesAreReadyToProcess, AssetFetch},
};
use anput::bundle::DynamicBundle;
use std::{
    collections::{BTreeMap, HashMap},
    error::Error,
};

pub struct AssetFromCollection;

impl AssetFetch for Vec<(String, Vec<u8>)> {
    fn load_bytes(&self, path: AssetPath) -> Result<DynamicBundle, Box<dyn Error>> {
        let bytes = self
            .iter()
            .find(|(p, _)| p == path.path())
            .ok_or_else(|| -> Box<dyn Error> {
                format!("Missing map key: `{}`", path.path()).into()
            })?
            .1
            .clone();
        let mut bundle = DynamicBundle::default();
        let _ = bundle.add_component(AssetBytesAreReadyToProcess(bytes));
        let _ = bundle.add_component(AssetFromCollection);
        Ok(bundle)
    }
}

impl AssetFetch for HashMap<String, Vec<u8>> {
    fn load_bytes(&self, path: AssetPath) -> Result<DynamicBundle, Box<dyn Error>> {
        let bytes = self
            .get(path.path())
            .ok_or_else(|| -> Box<dyn Error> {
                format!("Missing map key: `{}`", path.path()).into()
            })
            .cloned()?;
        let mut bundle = DynamicBundle::default();
        let _ = bundle.add_component(AssetBytesAreReadyToProcess(bytes));
        let _ = bundle.add_component(AssetFromCollection);
        Ok(bundle)
    }
}

impl AssetFetch for BTreeMap<String, Vec<u8>> {
    fn load_bytes(&self, path: AssetPath) -> Result<DynamicBundle, Box<dyn Error>> {
        let bytes = self
            .get(path.path())
            .ok_or_else(|| -> Box<dyn Error> {
                format!("Missing map key: `{}`", path.path()).into()
            })
            .cloned()?;
        let mut bundle = DynamicBundle::default();
        let _ = bundle.add_component(AssetBytesAreReadyToProcess(bytes));
        let _ = bundle.add_component(AssetFromCollection);
        Ok(bundle)
    }
}
