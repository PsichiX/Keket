use crate::{
    database::path::AssetPath,
    fetch::{file::FileAssetFetch, AssetAwaitsResolution, AssetFetch},
};
use anput::{bundle::DynamicBundle, entity::Entity, query::Update, world::World, TypeHash};
use notify::{Config, Event, PollWatcher, RecursiveMode, Result as NotifyResult, Watcher};
use std::{
    error::Error,
    path::PathBuf,
    sync::{
        mpsc::{channel, Receiver},
        Mutex,
    },
};

pub struct HotReloadFileAssetFetch {
    fetch: FileAssetFetch,
    watcher: PollWatcher,
    rx: Mutex<Receiver<NotifyResult<Event>>>,
}

impl HotReloadFileAssetFetch {
    pub fn new(fetch: FileAssetFetch) -> Result<Self, Box<dyn Error>> {
        let (tx, rx) = channel::<NotifyResult<Event>>();
        let mut watcher = PollWatcher::new(tx, Config::default().with_manual_polling())?;
        watcher.watch(&fetch.root, RecursiveMode::Recursive)?;
        Ok(Self {
            fetch,
            watcher,
            rx: Mutex::new(rx),
        })
    }
}

impl AssetFetch for HotReloadFileAssetFetch {
    fn load_bytes(&self, path: AssetPath) -> Result<DynamicBundle, Box<dyn Error>> {
        self.fetch.load_bytes(path)
    }

    fn maintain(&mut self, storage: &mut World) -> Result<(), Box<dyn Error>> {
        self.watcher.poll()?;
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
