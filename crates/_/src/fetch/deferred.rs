use crate::{
    database::path::{AssetPath, AssetPathStatic},
    fetch::AssetFetch,
};
use anput::{bundle::DynamicBundle, world::World};
use std::{
    error::Error,
    sync::{Arc, RwLock},
    thread::{JoinHandle, spawn},
};

/// Marker component used to signify that the asset fetch job for an asset
/// is deferred and is pending execution.
pub struct AssetAwaitsDeferredJob;

/// A deferred asset fetcher that queues tasks for loading asset bytes asynchronously
/// on separate threads and defers processing until the tasks are completed.
///
/// The `DeferredAssetFetch` struct allows asset fetching to occur in the background
/// on worker threads, with tasks being executed asynchronously and loaded asset bytes
/// being processed only when the task has finished.
pub struct DeferredAssetFetch<Fetch: AssetFetch> {
    fetch: Arc<RwLock<Fetch>>,
    // TODO: refactor to use worker threads with tasks queued to execution on worker threads.
    #[allow(clippy::type_complexity)]
    tasks: RwLock<Vec<(AssetPathStatic, JoinHandle<Result<DynamicBundle, String>>)>>,
}

impl<Fetch: AssetFetch> DeferredAssetFetch<Fetch> {
    /// Creates a new `DeferredAssetFetch` with a specified asset fetcher.
    ///
    /// # Arguments
    /// - `fetch`: The asset fetcher to handle fetching the bytes of an asset in the background.
    ///
    /// # Returns
    /// - A new `DeferredAssetFetch` instance.
    pub fn new(fetch: Fetch) -> Self {
        Self {
            fetch: Arc::new(RwLock::new(fetch)),
            tasks: Default::default(),
        }
    }
}

impl<Fetch: AssetFetch> AssetFetch for DeferredAssetFetch<Fetch> {
    fn load_bytes(&self, path: AssetPath) -> Result<DynamicBundle, Box<dyn Error>> {
        let path = path.into_static();
        let fetch = self.fetch.clone();
        self.tasks
            .write()
            .map_err(|error| format!("{}", error))?
            .push((
                path.clone(),
                spawn(move || {
                    fetch.read().map_err(|error|{
                        format!(
                            "Failed to get read access to inner fetch engine in deferred job for asset: `{}`. Error: {}",
                            path, error
                        )
                    })?.load_bytes(path.clone()).map_err(|error| {
                        format!(
                            "Failed deferred job for asset: `{}`. Error: {}",
                            path, error
                        )
                    })
                }),
            ));
        let mut bundle = DynamicBundle::default();
        let _ = bundle.add_component(AssetAwaitsDeferredJob);
        Ok(bundle)
    }

    fn maintain(&mut self, storage: &mut World) -> Result<(), Box<dyn Error>> {
        self.fetch
            .write()
            .map_err(|error| {
                format!(
                    "Failed deferred fetch engine maintainance. Error: {}",
                    error
                )
            })?
            .maintain(storage)?;
        let complete = self
            .tasks
            .read()
            .map_err(|error| format!("{}", error))?
            .iter()
            .enumerate()
            .filter(|(_, (_, join))| join.is_finished())
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        for index in complete.into_iter().rev() {
            let (path, join) = self
                .tasks
                .write()
                .map_err(|error| format!("{}", error))?
                .swap_remove(index);
            let result = join
                .join()
                .map_err(|_| format!("Deferred job execution of `{}` asset panicked!", path))??;
            if let Some(entity) = storage.find_by::<true, _>(&path) {
                storage.remove::<(AssetAwaitsDeferredJob,)>(entity)?;
                storage.insert(entity, result)?;
            }
        }
        Ok(())
    }
}
