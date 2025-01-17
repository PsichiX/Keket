use keket::{
    database::path::AssetPath,
    fetch::{AssetBytesAreReadyToProcess, AssetFetch},
    third_party::anput::bundle::DynamicBundle,
};
use reqwest::Url;
use std::{error::Error, net::SocketAddr};

pub mod third_party {
    pub use reqwest;
}

/// A marker struct indicating an asset originates from asset server client.
pub struct AssetFromClient;

/// Client asset fetch from asset server.
pub struct ClientAssetFetch {
    root: Url,
}

impl ClientAssetFetch {
    /// Creates a new instance of `ClientAssetFetch` with the given address.
    ///
    /// # Arguments
    /// - `address`: A string slice representing the server IP address.
    ///
    /// # Returns
    /// - `Ok(ClientAssetFetch)` if the initialization is successful.
    /// - `Err(Box<dyn Error>)` if any parsing errors occur.
    pub fn new(address: &str) -> Result<Self, Box<dyn Error>> {
        address.parse::<SocketAddr>()?;
        let root = format!("http://{}/assets/", address).parse::<Url>()?;
        Ok(Self { root })
    }
}

impl AssetFetch for ClientAssetFetch {
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
        let _ = bundle.add_component(AssetFromClient);
        let _ = bundle.add_component(url);
        Ok(bundle)
    }
}
