use crate::{
    database::path::{AssetPath, AssetPathStatic},
    fetch::{AssetAwaitsAsyncFetch, AssetFetch},
};
use anput::{
    bundle::DynamicBundle, third_party::intuicio_data::managed::ManagedValue, world::World,
};
use moirai::{JobHandle, JobLocation, JobPriority, Jobs};
use std::{
    collections::HashMap,
    error::Error,
    sync::{Arc, RwLock},
};

/// A deferred asset fetcher that queues tasks for loading asset bytes asynchronously
/// on separate jobs and defers processing until the tasks are completed.
///
/// The `DeferredAssetFetch` struct allows asset fetching to occur in the background
/// on jobs, with tasks being executed asynchronously and loaded asset bytes
/// being processed only when the task has finished.
pub struct DeferredAssetFetch<Fetch: AssetFetch> {
    fetch: Arc<RwLock<Fetch>>,
    jobs: ManagedValue<Jobs>,
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
            jobs: ManagedValue::Owned(Default::default()),
            job_handles: Default::default(),
        }
    }

    /// Sets the jobs for the deferred asset fetcher.
    ///
    /// # Arguments
    /// - `jobs`: The jobs runner to be managed by the deferred asset fetcher.
    ///
    /// # Returns
    /// - A new `DeferredAssetFetch` instance with the updated jobs.
    pub fn jobs(mut self, jobs: impl Into<ManagedValue<Jobs>>) -> Self {
        self.jobs = jobs.into();
        self
    }
}

impl<Fetch: AssetFetch> AssetFetch for DeferredAssetFetch<Fetch> {
    fn load_bytes(&self, path: AssetPath) -> Result<DynamicBundle, Box<dyn Error>> {
        let path = path.into_static();
        let path2 = path.clone();
        let fetch = self.fetch.clone();
        let job = async move {
            fetch.read().map_err(|error|{
                format!(
                    "Failed to get read access to inner fetch engine in async fetch for asset: `{path}`. Error: {error}"
                )
            })?.load_bytes(path.clone()).map_err(|error| {
                format!(
                    "Failed async fetch for asset: `{path}`. Error: {error}"
                )
            })
        };
        let jobs = self.jobs.read().ok_or_else(|| {
            format!("Failed to get read access to jobs runner in async fetch for asset: `{path2}`")
        })?;
        let handle = jobs.spawn_on(
            JobLocation::other_than_current_thread(),
            JobPriority::Normal,
            job,
        )?;
        self.job_handles
            .write()
            .map_err(|error| format!("{error}"))?
            .insert(path2, handle);
        let mut bundle = DynamicBundle::default();
        let _ = bundle.add_component(AssetAwaitsAsyncFetch);
        Ok(bundle)
    }

    fn maintain(&mut self, storage: &mut World) -> Result<(), Box<dyn Error>> {
        if let ManagedValue::Owned(jobs) = &self.jobs {
            jobs.read()
                .ok_or("Failed to get read access to jobs runner in deferred fetch maintainance.")?
                .run_local();
        }

        self.fetch
            .write()
            .map_err(|error| format!("Failed deferred fetch engine maintainance. Error: {error}"))?
            .maintain(storage)?;

        let complete = self
            .job_handles
            .read()
            .map_err(|error| format!("{error}"))?
            .iter()
            .filter(|(_, handle)| handle.is_done())
            .map(|(path, _)| path.clone())
            .collect::<Vec<_>>();
        for path in complete.into_iter().rev() {
            let handle = self
                .job_handles
                .write()
                .map_err(|error| format!("{error}"))?
                .remove(&path)
                .unwrap();
            match handle.try_take() {
                Some(Some(result)) => {
                    if let Some(entity) = storage.find_by::<true, _>(&path) {
                        storage.remove::<(AssetAwaitsAsyncFetch,)>(entity)?;
                    }
                    let result = result.map_err(|error| {
                        format!("Async fetch execution of `{path}` asset panicked! Error: {error}")
                    })?;
                    if let Some(entity) = storage.find_by::<true, _>(&path) {
                        storage.insert(entity, result)?;
                    }
                }
                Some(None) => {
                    if let Some(entity) = storage.find_by::<true, _>(&path) {
                        storage.remove::<(AssetAwaitsAsyncFetch,)>(entity)?;
                    }
                    return Err(format!(
                        "Async fetch execution of `{path}` asset failed with undefined error!"
                    )
                    .into());
                }
                None => {
                    self.job_handles
                        .write()
                        .map_err(|error| format!("{error}"))?
                        .insert(path.clone(), handle);
                }
            };
        }
        Ok(())
    }
}
