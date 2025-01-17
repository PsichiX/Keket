use crate::{
    database::path::AssetPath,
    fetch::{file::FileAssetFetch, AssetAwaitsResolution, AssetFetch},
};
use anput::{
    bundle::DynamicBundle, entity::Entity, query::Update,
    third_party::intuicio_data::prelude::TypeHash, world::World,
};
use notify::{Config, Event, PollWatcher, RecursiveMode, Result as NotifyResult, Watcher};
use std::{
    error::Error,
    path::PathBuf,
    sync::{
        mpsc::{channel, Receiver},
        Mutex,
    },
    time::Duration,
};

/// A file asset fetcher with hot reload capabilities.
/// This fetcher watches a specified directory for file changes and reloads affected assets on modification.
pub struct HotReloadFileAssetFetch {
    fetch: FileAssetFetch,
    rx: Mutex<Receiver<NotifyResult<Event>>>,
    _watcher: PollWatcher,
}

impl HotReloadFileAssetFetch {
    /// Creates a new `HotReloadFileAssetFetch` with the specified file fetcher.
    ///
    /// # Arguments
    /// - `fetch`: A `FileAssetFetch` that defines the root directory to watch and the logic for loading asset bytes.
    ///
    /// # Returns
    /// - A new `HotReloadFileAssetFetch` instance if initialization succeeds.
    /// - An error if the watcher fails to initialize.
    pub fn new(fetch: FileAssetFetch, poll_interval: Duration) -> Result<Self, Box<dyn Error>> {
        let (tx, rx) = channel::<NotifyResult<Event>>();
        let mut watcher =
            PollWatcher::new(tx, Config::default().with_poll_interval(poll_interval))?;
        watcher.watch(&fetch.root, RecursiveMode::Recursive)?;
        Ok(Self {
            fetch,
            rx: Mutex::new(rx),
            _watcher: watcher,
        })
    }
}

impl AssetFetch for HotReloadFileAssetFetch {
    fn load_bytes(&self, path: AssetPath) -> Result<DynamicBundle, Box<dyn Error>> {
        self.fetch.load_bytes(path)
    }

    fn maintain(&mut self, storage: &mut World) -> Result<(), Box<dyn Error>> {
        let rx = self.rx.lock().map_err(|error| format!("{}", error))?;
        while let Ok(Ok(event)) = rx.try_recv() {
            if event.kind.is_modify() {
                let to_refresh = storage
                    .query::<true, (Entity, &PathBuf, Update<AssetPath>)>()
                    .filter(|(_, path, _)| event.paths.contains(path))
                    .inspect(|(_, _, path)| path.notify(storage))
                    .map(|(entity, _, _)| entity)
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
        self.fetch.maintain(storage)
    }
}
