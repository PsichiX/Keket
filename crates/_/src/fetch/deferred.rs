use crate::{
    database::path::{AssetPath, AssetPathStatic},
    fetch::AssetFetch,
};
use anput::{
    bundle::DynamicBundle,
    jobs::{JobHandle, Jobs},
    world::World,
};
use std::{
    collections::HashMap,
    error::Error,
    sync::{Arc, RwLock},
};

/// Marker component used to signify that the asset fetch job for an asset
/// is deferred and is pending execution.
pub struct AssetAwaitsDeferredJob;

/// A deferred asset fetcher that queues tasks for loading asset bytes asynchronously
/// on separate jobs and defers processing until the tasks are completed.
///
/// The `DeferredAssetFetch` struct allows asset fetching to occur in the background
/// on jobs, with tasks being executed asynchronously and loaded asset bytes
/// being processed only when the task has finished.
pub struct DeferredAssetFetch<Fetch: AssetFetch> {
    fetch: Arc<RwLock<Fetch>>,
    jobs: Jobs,
    #[allow(clippy::type_complexity)]
    job_handles: RwLock<HashMap<AssetPathStatic, JobHandle<Result<DynamicBundle, String>>>>,
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
            jobs: Default::default(),
            job_handles: Default::default(),
        }
    }
}

impl<Fetch: AssetFetch> AssetFetch for DeferredAssetFetch<Fetch> {
    fn load_bytes(&self, path: AssetPath) -> Result<DynamicBundle, Box<dyn Error>> {
        let path = path.into_static();
        let path2 = path.clone();
        let fetch = self.fetch.clone();
        let handle = self.jobs.queue(move |_| {
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
        })?;
        self.job_handles
            .write()
            .map_err(|error| format!("{}", error))?
            .insert(path2, handle);
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
            .job_handles
            .read()
            .map_err(|error| format!("{}", error))?
            .iter()
            .filter(|(_, handle)| handle.is_done())
            .map(|(path, _)| path.clone())
            .collect::<Vec<_>>();
        for path in complete.into_iter().rev() {
            let handle = self
                .job_handles
                .write()
                .map_err(|error| format!("{}", error))?
                .remove(&path)
                .unwrap();
            match handle.try_take() {
                Some(Some(result)) => {
                    let result = result.map_err(|_| {
                        format!("Deferred job execution of `{}` asset panicked!", path)
                    })?;
                    if let Some(entity) = storage.find_by::<true, _>(&path) {
                        storage.remove::<(AssetAwaitsDeferredJob,)>(entity)?;
                        storage.insert(entity, result)?;
                    }
                }
                Some(None) => {
                    if let Some(entity) = storage.find_by::<true, _>(&path) {
                        storage.remove::<(AssetAwaitsDeferredJob,)>(entity)?;
                    }
                    return Err(format!(
                        "Deferred job execution of `{}` asset failed with undefined error!",
                        path
                    )
                    .into());
                }
                None => {
                    self.job_handles
                        .write()
                        .map_err(|error| format!("{}", error))?
                        .insert(path.clone(), handle);
                }
            };
        }
        Ok(())
    }
}
