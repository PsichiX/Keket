use keket::{
    database::{path::AssetPath, reference::AssetRef},
    fetch::{deferred::DeferredAssetJob, AssetBytesAreReadyToProcess, AssetFetch},
    third_party::anput::world::World,
};
use reqwest::Url;
use std::error::Error;

pub mod third_party {
    pub use reqwest;
}

pub struct AssetFromHttp;

pub struct HttpAssetFetch {
    root: Url,
}

impl HttpAssetFetch {
    pub fn new(root: &str) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            root: root.parse()?,
        })
    }
}

impl AssetFetch for HttpAssetFetch {
    fn load_bytes(
        &mut self,
        reference: AssetRef,
        path: AssetPath,
        storage: &mut World,
    ) -> Result<(), Box<dyn Error>> {
        let url = self.root.join(path.path()).unwrap_or_else(|error| {
            panic!(
                "Failed to join root URL: `{}` with path: `{}`. Error: {}",
                self.root,
                path.path_with_meta(),
                error
            );
        });
        let mut response = reqwest::blocking::get(url.clone()).unwrap_or_else(|error| {
            panic!(
                "Failed to get HTTP content from: `{}`. Error: {}",
                url, error
            );
        });
        let mut bytes = vec![];
        response.copy_to(&mut bytes).unwrap_or_else(|error| {
            panic!(
                "Failed to read bytes response from: `{}`. Error: {}",
                url, error
            );
        });
        let bundle = (AssetBytesAreReadyToProcess(bytes), AssetFromHttp, url);
        storage.insert(reference.entity(), bundle)?;
        Ok(())
    }
}

impl DeferredAssetJob for HttpAssetFetch {
    type Result = (AssetBytesAreReadyToProcess, AssetFromHttp, Url);

    fn execute(&self, path: AssetPath) -> Result<Self::Result, String> {
        let url = self.root.join(path.path()).map_err(|error| {
            format!(
                "Failed to join root URL: `{}` with path: `{}`. Error: {}",
                self.root,
                path.path_with_meta(),
                error
            )
        })?;
        let mut response = reqwest::blocking::get(url.clone()).map_err(|error| {
            format!(
                "Failed to get HTTP content from: `{}`. Error: {}",
                url, error
            )
        })?;
        let mut bytes = vec![];
        response.copy_to(&mut bytes).map_err(|error| {
            format!(
                "Failed to read bytes response from: `{}`. Error: {}",
                url, error
            )
        })?;
        Ok((AssetBytesAreReadyToProcess(bytes), AssetFromHttp, url))
    }
}
