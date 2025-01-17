use crate::{
    database::path::{AssetPath, AssetPathStatic},
    fetch::AssetFetch,
};
use anput::{bundle::DynamicBundle, world::World};
use std::error::Error;

pub struct FallbackAssetFetch<Fetch: AssetFetch> {
    fetch: Fetch,
    pub assets: Vec<AssetPathStatic>,
}

impl<Fetch: AssetFetch> FallbackAssetFetch<Fetch> {
    pub fn new(fetch: Fetch) -> Self {
        Self {
            fetch,
            assets: Default::default(),
        }
    }

    pub fn path(mut self, path: impl Into<AssetPathStatic>) -> Self {
        self.assets.push(path.into());
        self
    }
}

impl<Fetch: AssetFetch> AssetFetch for FallbackAssetFetch<Fetch> {
    fn load_bytes(&self, path: AssetPath) -> Result<DynamicBundle, Box<dyn Error>> {
        let mut status = self.fetch.load_bytes(path.clone());
        if status.is_err() {
            for asset in &self.assets {
                if asset.protocol() == path.protocol() {
                    status = self.fetch.load_bytes(asset.clone());
                    if status.is_ok() {
                        break;
                    }
                }
            }
        }
        status
    }

    fn maintain(&mut self, storage: &mut World) -> Result<(), Box<dyn Error>> {
        self.fetch.maintain(storage)
    }
}
