use keket::{
    database::path::AssetPath,
    fetch::{AssetBytesAreReadyToProcess, AssetFetch},
    third_party::anput::bundle::DynamicBundle,
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
    fn load_bytes(&self, path: AssetPath) -> Result<DynamicBundle, Box<dyn Error>> {
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
        let mut bundle = DynamicBundle::default();
        let _ = bundle.add_component(AssetBytesAreReadyToProcess(bytes));
        let _ = bundle.add_component(AssetFromHttp);
        let _ = bundle.add_component(url);
        Ok(bundle)
    }
}
