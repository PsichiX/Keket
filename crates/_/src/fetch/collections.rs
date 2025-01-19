use crate::{
    database::path::AssetPath,
    fetch::{AssetBytesAreReadyToProcess, AssetFetch},
};
use anput::bundle::DynamicBundle;
use std::{
    collections::{BTreeMap, HashMap},
    error::Error,
};

/// A marker component added to bundles that represent assets loaded from a collection.
/// The `AssetFromCollection` component serves as an indicator, though it doesn't hold data
/// about the asset itself.
pub struct AssetFromCollection;

impl<const N: usize> AssetFetch for [(&'static str, &'static [u8]); N] {
    fn load_bytes(&self, path: AssetPath) -> Result<DynamicBundle, Box<dyn Error>> {
        let bytes = self
            .iter()
            .find(|(p, _)| *p == path.path())
            .ok_or_else(|| -> Box<dyn Error> { format!("Missing key: `{}`", path.path()).into() })?
            .1
            .to_vec();
        let mut bundle = DynamicBundle::default();
        let _ = bundle.add_component(AssetBytesAreReadyToProcess(bytes));
        let _ = bundle.add_component(AssetFromCollection);
        Ok(bundle)
    }
}

impl AssetFetch for Vec<(String, Vec<u8>)> {
    fn load_bytes(&self, path: AssetPath) -> Result<DynamicBundle, Box<dyn Error>> {
        let bytes = self
            .iter()
            .find(|(p, _)| p == path.path())
            .ok_or_else(|| -> Box<dyn Error> { format!("Missing key: `{}`", path.path()).into() })?
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
            .ok_or_else(|| -> Box<dyn Error> { format!("Missing key: `{}`", path.path()).into() })
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
            .ok_or_else(|| -> Box<dyn Error> { format!("Missing key: `{}`", path.path()).into() })
            .cloned()?;
        let mut bundle = DynamicBundle::default();
        let _ = bundle.add_component(AssetBytesAreReadyToProcess(bytes));
        let _ = bundle.add_component(AssetFromCollection);
        Ok(bundle)
    }
}
