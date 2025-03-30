pub mod events;
pub mod handle;
pub mod loading;
pub mod path;
pub mod reference;
pub mod tags;

use crate::{
    database::{
        events::{AssetEvent, AssetEventBindings, AssetEventKind, AssetEventListener},
        handle::{AssetDependency, AssetHandle},
        loading::AssetsLoadingStatus,
        path::{AssetPath, AssetPathStatic},
    },
    fetch::{
        deferred::AssetAwaitsDeferredJob, AssetAwaitsResolution, AssetBytesAreReadyToProcess,
        AssetFetch, AssetFetchEngine,
    },
    protocol::AssetProtocol,
};
use anput::{
    bundle::{Bundle, BundleChain},
    commands::Command,
    database::WorldDestroyIteratorExt,
    entity::Entity,
    query::Include,
    world::World,
};
use std::{
    error::Error,
    sync::mpsc::{channel, Receiver, Sender},
};

pub type AssetDatabaseCommand = Box<dyn FnOnce(&mut World) + Send + Sync>;

/// Asset database for managing assets and their states.
pub struct AssetDatabase {
    pub storage: World,
    pub events: AssetEventBindings,
    pub allow_asset_progression_failures: bool,
    fetch_stack: Vec<AssetFetchEngine>,
    protocols: Vec<Box<dyn AssetProtocol>>,
    #[allow(clippy::type_complexity)]
    sender: Sender<AssetDatabaseCommand>,
    #[allow(clippy::type_complexity)]
    receiver: Receiver<AssetDatabaseCommand>,
}

impl Default for AssetDatabase {
    fn default() -> Self {
        let (sender, receiver) = channel();
        Self {
            storage: Default::default(),
            events: Default::default(),
            allow_asset_progression_failures: false,
            fetch_stack: Default::default(),
            protocols: Default::default(),
            sender,
            receiver,
        }
    }
}

impl AssetDatabase {
    /// Creates a new `AssetDatabase` and adds a fetcher to its fetch stack.
    ///
    /// # Arguments
    /// - `fetch`: A concrete implementation of the `AssetFetch` trait.
    ///
    /// # Returns
    /// The updated `AssetDatabase` with the fetcher added.
    pub fn with_fetch(mut self, fetch: impl AssetFetch + 'static) -> Self {
        self.push_fetch(fetch);
        self
    }

    /// Registers a new asset protocol with the database.
    ///
    /// # Arguments
    /// - `protocol`: An implementation of the `AssetProtocol` trait.
    ///
    /// # Returns
    /// The updated `AssetDatabase` with the protocol added.
    pub fn with_protocol(mut self, protocol: impl AssetProtocol + 'static) -> Self {
        self.add_protocol(protocol);
        self
    }

    /// Enables allowing asset progression failures.
    ///
    /// # Returns
    /// The updated `AssetDatabase` with the option enabled.
    pub fn with_asset_progression_failures(mut self) -> Self {
        self.allow_asset_progression_failures = true;
        self
    }

    /// Binds event listener.
    ///
    /// # Returns
    /// The updated `AssetDatabase` with the option enabled.
    pub fn with_event(mut self, listener: impl AssetEventListener + 'static) -> Self {
        self.events.bind(listener);
        self
    }

    /// Adds a fetch engine to the stack.
    pub fn push_fetch(&mut self, fetch: impl AssetFetch + 'static) {
        self.fetch_stack.push(AssetFetchEngine::new(fetch));
    }

    /// Removes and returns the top fetch engine from the stack.
    pub fn pop_fetch(&mut self) -> Option<Box<dyn AssetFetch>> {
        self.fetch_stack.pop().map(|fetch| fetch.into_inner())
    }

    /// Replaces the top fetch engine and returns the old one.
    ///
    /// # Arguments
    /// - `fetch`: A new fetch implementation to replace the top one.
    ///
    /// # Returns
    /// The old fetch engine if present.
    pub fn swap_fetch(&mut self, fetch: impl AssetFetch + 'static) -> Option<Box<dyn AssetFetch>> {
        let result = self.pop_fetch();
        self.fetch_stack.push(AssetFetchEngine::new(fetch));
        result
    }

    /// Temporarily uses a fetch engine to perform a closure and removes it afterward.
    ///
    /// # Arguments
    /// - `fetch`: The fetch engine to add temporarily.
    /// - `f`: The closure to execute using the fetch engine.
    ///
    /// # Returns
    /// The result of the closure if successful, or an error otherwise.
    pub fn using_fetch<R>(
        &mut self,
        fetch: impl AssetFetch + 'static,
        f: impl FnOnce(&mut Self) -> Result<R, Box<dyn Error>>,
    ) -> Result<R, Box<dyn Error>> {
        self.push_fetch(fetch);
        let result = f(self)?;
        self.pop_fetch();
        Ok(result)
    }

    /// Registers a new protocol for processing assets.
    pub fn add_protocol(&mut self, protocol: impl AssetProtocol + 'static) {
        self.protocols.push(Box::new(protocol));
    }

    /// Removes a protocol by its name.
    pub fn remove_protocol(&mut self, name: &str) -> Option<Box<dyn AssetProtocol>> {
        self.protocols
            .iter()
            .position(|protocol| protocol.name() == name)
            .map(|index| self.protocols.remove(index))
    }

    /// Finds an asset by its path and returns a handle.
    ///
    /// # Arguments
    /// - `path`: The path of the asset to find.
    ///
    /// # Returns
    /// An `AssetHandle` if the asset is found, otherwise `None`.
    pub fn find(&self, path: impl Into<AssetPathStatic>) -> Option<AssetHandle> {
        let path = path.into();
        self.storage.find_by::<true, _>(&path).map(AssetHandle::new)
    }

    /// Schedules an asset to be resolved later.
    ///
    /// # Arguments
    /// - `path`: The path of the asset to schedule.
    ///
    /// # Returns
    /// An `AssetHandle` for the scheduled asset.
    pub fn schedule(
        &mut self,
        path: impl Into<AssetPathStatic>,
    ) -> Result<AssetHandle, Box<dyn Error>> {
        let path = path.into();
        Ok(AssetHandle::new(
            self.storage.spawn((path, AssetAwaitsResolution))?,
        ))
    }

    /// Adds an asset to database, already resolved.
    /// Great for runtime generated assets.
    ///
    /// # Arguments
    /// - `path`: The path of the asset to schedule.
    /// - `bundle`: Components bundle to put into spawned asset.
    ///
    /// # Returns
    /// An `AssetHandle` for the scheduled asset.
    pub fn spawn(
        &mut self,
        path: impl Into<AssetPathStatic>,
        bundle: impl Bundle,
    ) -> Result<AssetHandle, Box<dyn Error>> {
        let path = path.into();
        Ok(AssetHandle::new(
            self.storage.spawn(BundleChain((path,), bundle))?,
        ))
    }

    /// Ensures an asset exists or is scheduled for resolution.
    ///
    /// # Arguments
    /// - `path`: The path of the asset to ensure.
    ///
    /// # Returns
    /// An `AssetHandle` for the asset.
    pub fn ensure(
        &mut self,
        path: impl Into<AssetPathStatic>,
    ) -> Result<AssetHandle, Box<dyn Error>> {
        let path = path.into();
        if let Some(entity) = self.storage.find_by::<true, _>(&path) {
            return Ok(AssetHandle::new(entity));
        }
        if let Some(fetch) = self.fetch_stack.last_mut() {
            let entity = self.storage.spawn((path.clone(),))?;
            let handle = AssetHandle::new(entity);
            let status = fetch.load_bytes(handle, path.clone(), &mut self.storage);
            if !self.allow_asset_progression_failures {
                status?;
            }
            if handle.bytes_are_ready_to_process(self) {
                let Some(protocol) = self
                    .protocols
                    .iter_mut()
                    .find(|protocol| protocol.name() == path.protocol())
                else {
                    handle.delete(self);
                    return Err(format!("Missing protocol for asset: `{}`", path).into());
                };
                let status = protocol.process_asset(handle, &mut self.storage);
                if status.is_err() {
                    if let Ok(mut bindings) = self
                        .storage
                        .component_mut::<true, AssetEventBindings>(handle.entity())
                    {
                        bindings.dispatch(AssetEvent {
                            handle,
                            kind: AssetEventKind::BytesProcessingFailed,
                            path: path.clone(),
                        })?;
                    }
                }
                if !self.allow_asset_progression_failures {
                    status?;
                }
            }
            Ok(handle)
        } else {
            Err("There is no asset fetch on stack!".into())
        }
    }

    /// Unloads an asset by its path, removing it from the storage.
    ///
    /// # Arguments
    /// - `path`: The path of the asset to unload.
    pub fn unload<'a>(&mut self, path: impl Into<AssetPath<'a>>) {
        let path = path.into();
        let to_remove = self
            .storage
            .query::<true, (Entity, &AssetPath)>()
            .filter(|(_, p)| *p == &path)
            .map(|(entity, _)| entity);
        self.storage
            .traverse_outgoing::<true, AssetDependency>(to_remove)
            .map(|(_, entity)| entity)
            .to_despawn_command()
            .execute(&mut self.storage)
    }

    /// Tries to dereference an asset by its path. If asset has no references
    /// left, it gets removed it from the storage.
    ///
    /// # Arguments
    /// - `path`: The path of the asset to unload.
    pub fn dereference_or_unload<'a>(&mut self, path: impl Into<AssetPath<'a>>) {
        let path = path.into();
        let to_remove = self
            .storage
            .query::<true, (Entity, &AssetPath)>()
            .filter(|(_, p)| *p == &path)
            .filter_map(|(entity, _)| {
                if let Ok(mut counter) = self
                    .storage
                    .component_mut::<true, AssetReferenceCounter>(entity)
                {
                    counter.decrement();
                    if counter.counter() == 0 {
                        Some(entity)
                    } else {
                        None
                    }
                } else {
                    Some(entity)
                }
            });
        self.storage
            .traverse_outgoing::<true, AssetDependency>(to_remove)
            .map(|(_, entity)| entity)
            .to_despawn_command()
            .execute(&mut self.storage);
    }

    /// Reloads an asset by unloading and ensuring it is reloaded.
    ///
    /// # Arguments
    /// - `path`: The path of the asset to reload.
    ///
    /// # Returns
    /// An `AssetHandle` for the reloaded asset.
    pub fn reload(
        &mut self,
        path: impl Into<AssetPathStatic>,
    ) -> Result<AssetHandle, Box<dyn Error>> {
        let path = path.into();
        self.unload(path.clone());
        self.ensure(path)
    }

    /// Returns an iterator over all assets waiting for resolution.
    ///
    /// # Returns
    /// An iterator that yields `AssetHandle` instances.
    pub fn assets_awaiting_resolution(&self) -> impl Iterator<Item = AssetHandle> + '_ {
        self.storage
            .query::<true, (Entity, Include<AssetAwaitsResolution>)>()
            .map(|(entity, _)| AssetHandle::new(entity))
    }

    /// Returns an iterator over all assets whose bytes are ready to process.
    ///
    /// # Returns
    /// An iterator that yields `AssetHandle` instances.
    pub fn assets_with_bytes_ready_to_process(&self) -> impl Iterator<Item = AssetHandle> + '_ {
        self.storage
            .query::<true, (Entity, Include<AssetBytesAreReadyToProcess>)>()
            .map(|(entity, _)| AssetHandle::new(entity))
    }

    /// Returns an iterator over all assets awaiting deferred jobs.
    ///
    /// # Returns
    /// An iterator that yields `AssetHandle` instances.
    pub fn assets_awaiting_deferred_job(&self) -> impl Iterator<Item = AssetHandle> + '_ {
        self.storage
            .query::<true, (Entity, Include<AssetAwaitsDeferredJob>)>()
            .map(|(entity, _)| AssetHandle::new(entity))
    }

    /// Checks if there are any assets awaiting resolution.
    ///
    /// # Returns
    /// `true` if assets are awaiting resolution, otherwise `false`.
    pub fn does_await_resolution(&self) -> bool {
        self.storage.has_component::<AssetAwaitsResolution>()
    }

    /// Checks if there are assets with bytes ready to process.
    ///
    /// # Returns
    /// `true` if assets are ready to process, otherwise `false`.
    pub fn has_bytes_ready_to_process(&self) -> bool {
        self.storage.has_component::<AssetBytesAreReadyToProcess>()
    }

    /// Checks if there are assets awaiting deferred jobs.
    ///
    /// # Returns
    /// `true` if deferred jobs are awaiting, otherwise `false`.
    pub fn does_await_deferred_job(&self) -> bool {
        self.storage.has_component::<AssetAwaitsDeferredJob>()
    }

    /// Determines if the asset database is currently busy with tasks.
    ///
    /// # Returns
    /// `true` if busy, otherwise `false`.
    pub fn is_busy(&self) -> bool {
        self.storage.has_component::<AssetAwaitsResolution>()
            || self.storage.has_component::<AssetBytesAreReadyToProcess>()
            || self.storage.has_component::<AssetAwaitsDeferredJob>()
    }

    /// Reports the status of assets in the database.
    ///
    /// # Arguments
    /// - `out_status`: A mutable reference to output `AssetsLoadingStatus`.
    pub fn report_loading_status(&self, out_status: &mut AssetsLoadingStatus) {
        out_status.clear();
        for (
            handle,
            asset_awaits_resolution,
            asset_bytes_ready_to_process,
            asset_awaits_deferred_job,
        ) in self.storage.query::<true, (
            AssetHandle,
            Option<&AssetAwaitsResolution>,
            Option<&AssetBytesAreReadyToProcess>,
            Option<&AssetAwaitsDeferredJob>,
        )>() {
            if asset_awaits_resolution.is_some() {
                out_status.awaiting_resolution.add(handle);
            } else if asset_bytes_ready_to_process.is_some() {
                out_status.with_bytes_ready_to_process.add(handle);
            } else if asset_awaits_deferred_job.is_some() {
                out_status.awaiting_deferred_job.add(handle);
            } else {
                out_status.ready_to_use.add(handle);
            }
        }
    }

    /// Returns the sender for asset database commands.
    /// This can be used to send commands to the asset database from external places.
    #[allow(clippy::type_complexity)]
    pub fn commands_sender(&self) -> Sender<AssetDatabaseCommand> {
        self.sender.clone()
    }

    /// Performs maintenance on the asset database, processing events and managing states.
    ///
    /// - Processes newly added assets and dispatches relevant events.
    /// - Maintains fetch engines and protocols.
    /// - Resolves assets and processes their data using protocols.
    ///
    /// # Returns
    /// `Ok(())` if successful, or an error if any step fails.
    pub fn maintain(&mut self) -> Result<(), Box<dyn Error>> {
        while let Ok(command) = self.receiver.try_recv() {
            command(&mut self.storage);
        }
        let despawn = if let Some(changes) = self.storage.updated() {
            if changes.has_component::<AssetReferenceCounter>() {
                Some(
                    changes
                        .iter_of::<AssetReferenceCounter>()
                        .filter_map(|entity| {
                            let counter = self
                                .storage
                                .component::<true, AssetReferenceCounter>(entity)
                                .ok()?;
                            if counter.counter() == 0 {
                                Some(entity)
                            } else {
                                None
                            }
                        })
                        .to_despawn_command(),
                )
            } else {
                None
            }
        } else {
            None
        };
        if let Some(despawn) = despawn {
            despawn.execute(&mut self.storage);
        }
        {
            let mut lookup = self
                .storage
                .lookup_access::<true, (&AssetPathStatic, &mut AssetEventBindings)>();
            for entity in self.storage.added().iter_of::<AssetAwaitsResolution>() {
                if let Some((path, bindings)) = lookup.access(entity) {
                    let event = AssetEvent {
                        handle: AssetHandle::new(entity),
                        kind: AssetEventKind::AwaitsResolution,
                        path: path.clone(),
                    };
                    self.events.dispatch(event.clone())?;
                    bindings.dispatch(event)?;
                }
            }
            for entity in self.storage.added().iter_of::<AssetAwaitsDeferredJob>() {
                if let Some((path, bindings)) = lookup.access(entity) {
                    let event = AssetEvent {
                        handle: AssetHandle::new(entity),
                        kind: AssetEventKind::AwaitsDeferredJob,
                        path: path.clone(),
                    };
                    self.events.dispatch(event.clone())?;
                    bindings.dispatch(event)?;
                }
            }
            for entity in self
                .storage
                .added()
                .iter_of::<AssetBytesAreReadyToProcess>()
            {
                if let Some((path, bindings)) = lookup.access(entity) {
                    let event = AssetEvent {
                        handle: AssetHandle::new(entity),
                        kind: AssetEventKind::BytesReadyToProcess,
                        path: path.clone(),
                    };
                    self.events.dispatch(event.clone())?;
                    bindings.dispatch(event)?;
                }
            }
            for entity in self
                .storage
                .removed()
                .iter_of::<AssetBytesAreReadyToProcess>()
            {
                if let Some((path, bindings)) = lookup.access(entity) {
                    let handle = AssetHandle::new(entity);
                    if handle.is_ready_to_use(self) {
                        continue;
                    }
                    let event = AssetEvent {
                        handle,
                        kind: AssetEventKind::BytesProcessed,
                        path: path.clone(),
                    };
                    self.events.dispatch(event.clone())?;
                    bindings.dispatch(event)?;
                }
            }
            for entity in self.storage.removed().iter_of::<AssetPathStatic>() {
                if let Some((path, bindings)) = lookup.access(entity) {
                    let event = AssetEvent {
                        handle: AssetHandle::new(entity),
                        kind: AssetEventKind::Unloaded,
                        path: path.clone(),
                    };
                    self.events.dispatch(event.clone())?;
                    bindings.dispatch(event)?;
                }
            }
        }
        self.storage.clear_changes();
        for fetch in &mut self.fetch_stack {
            fetch.maintain(&mut self.storage)?;
        }
        for protocol in &mut self.protocols {
            protocol.maintain(&mut self.storage)?;
            let to_process = self
                .storage
                .query::<true, (Entity, &AssetPath, Include<AssetBytesAreReadyToProcess>)>()
                .filter(|(_, path, _)| path.protocol() == protocol.name())
                .map(|(entity, _, _)| AssetHandle::new(entity))
                .collect::<Vec<_>>();
            for handle in to_process {
                let status = protocol.process_asset(handle, &mut self.storage);
                if status.is_err() {
                    if let Ok(mut bindings) = self
                        .storage
                        .component_mut::<true, AssetEventBindings>(handle.entity())
                    {
                        bindings.dispatch(AssetEvent {
                            handle,
                            kind: AssetEventKind::BytesProcessingFailed,
                            path: self
                                .storage
                                .component::<true, AssetPathStatic>(handle.entity())?
                                .clone(),
                        })?;
                    }
                }
                if !self.allow_asset_progression_failures {
                    status?;
                }
            }
        }
        let to_resolve = self
            .storage
            .query::<true, (AssetHandle, &AssetPath, Include<AssetAwaitsResolution>)>()
            .map(|(handle, path, _)| (handle, path.clone()))
            .collect::<Vec<_>>();
        for (handle, path) in to_resolve {
            if let Some(fetch) = self.fetch_stack.last_mut() {
                let status = fetch.load_bytes(handle, path.clone(), &mut self.storage);
                if !self.allow_asset_progression_failures {
                    status?;
                }
                if handle.bytes_are_ready_to_process(self) {
                    let Some(protocol) = self
                        .protocols
                        .iter_mut()
                        .find(|protocol| protocol.name() == path.protocol())
                    else {
                        return Err(format!("Missing protocol for asset: `{}`", path).into());
                    };
                    let status = protocol.process_asset(handle, &mut self.storage);
                    if status.is_err() {
                        if let Ok(mut bindings) = self
                            .storage
                            .component_mut::<true, AssetEventBindings>(handle.entity())
                        {
                            bindings.dispatch(AssetEvent {
                                handle,
                                kind: AssetEventKind::BytesProcessingFailed,
                                path: self
                                    .storage
                                    .component::<true, AssetPathStatic>(handle.entity())?
                                    .clone(),
                            })?;
                        }
                    }
                    if !self.allow_asset_progression_failures {
                        status?;
                    }
                }
                self.storage
                    .remove::<(AssetAwaitsResolution,)>(handle.entity())?;
            } else {
                return Err("There is no asset fetch on stack!".into());
            }
        }
        Ok(())
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct AssetReferenceCounter(usize);

impl AssetReferenceCounter {
    pub fn counter(&self) -> usize {
        self.0
    }

    pub fn increment(&mut self) {
        self.0 = self.0.saturating_add(1);
    }

    pub fn decrement(&mut self) {
        self.0 = self.0.saturating_sub(1);
    }
}
