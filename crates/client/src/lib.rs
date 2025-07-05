use keket::{
    database::path::AssetPath,
    fetch::{AssetAwaitsResolution, AssetBytesAreReadyToProcess, AssetFetch},
    third_party::anput::{
        bundle::DynamicBundle, entity::Entity, query::Update,
        third_party::intuicio_data::type_hash::TypeHash, world::World,
    },
};
use reqwest::Url;
use std::{
    error::Error,
    net::{SocketAddr, TcpStream},
};
use tungstenite::{WebSocket, connect, stream::MaybeTlsStream};

pub mod third_party {
    pub use reqwest;
    pub use tungstenite;
}

/// A marker struct indicating an asset originates from asset server client.
pub struct AssetFromClient;

/// Client asset fetch from asset server.
pub struct ClientAssetFetch {
    root: Url,
    socket: WebSocket<MaybeTlsStream<TcpStream>>,
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
        let root = format!("http://{address}/assets/").parse::<Url>()?;
        let (socket, _) = connect(format!("ws://{address}/changes"))?;
        if let MaybeTlsStream::Plain(tcp) = socket.get_ref() {
            tcp.set_nonblocking(true)?;
        }
        Ok(Self { root, socket })
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
        let mut response = reqwest::blocking::get(url.clone())
            .map_err(|error| format!("Failed to get HTTP content from: `{url}`. Error: {error}"))?;
        let mut bytes = vec![];
        response.copy_to(&mut bytes).map_err(|error| {
            format!("Failed to read bytes response from: `{url}`. Error: {error}")
        })?;
        let mut bundle = DynamicBundle::default();
        let _ = bundle.add_component(AssetBytesAreReadyToProcess(bytes));
        let _ = bundle.add_component(AssetFromClient);
        let _ = bundle.add_component(url);
        Ok(bundle)
    }

    fn maintain(&mut self, storage: &mut World) -> Result<(), Box<dyn Error>> {
        if self.socket.can_read() {
            let paths = std::iter::from_fn(|| self.socket.read().ok())
                .filter(|message| message.is_text())
                .filter_map(|message| message.to_text().ok().map(|path| path.to_owned()))
                .collect::<Vec<_>>();
            if !paths.is_empty() {
                let to_refresh = storage
                    .query::<true, (Entity, Update<AssetPath>)>()
                    .filter(|(_, path)| paths.iter().any(|p| p == path.read().path()))
                    .inspect(|(_, path)| path.notify(storage))
                    .map(|(entity, _)| entity)
                    .collect::<Vec<_>>();
                for entity in to_refresh {
                    let columns = storage
                        .row::<true>(entity)?
                        .columns()
                        .filter(|info| info.type_hash() != TypeHash::of::<AssetPath>())
                        .cloned()
                        .collect::<Vec<_>>();
                    storage.remove_raw(entity, columns)?;
                    storage.insert(entity, (AssetAwaitsResolution,))?;
                }
            }
        }
        Ok(())
    }
}
