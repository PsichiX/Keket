pub mod events;
pub mod handle;
pub mod inspector;
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
        AssetAwaitsAsyncFetch, AssetAwaitsResolution, AssetBytesAreReadyToProcess, AssetFetch,
        AssetFetchEngine,
    },
    protocol::AssetProtocol,
    store::{
        AssetAwaitsAsyncStore, AssetAwaitsStoring, AssetBytesAreReadyToStore, AssetStore,
        AssetStoreEngine,
    },
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
    collections::VecDeque,
    error::Error,
    sync::{Arc, Mutex},
};

/// Command type for asset database operations.
pub type AssetDatabaseCommand = Box<dyn FnOnce(&mut World) + Send + Sync>;

/// Sender for asset database commands.
///
/// This is used to send commands to the asset database from external places.
#[derive(Clone)]
pub struct AssetDatabaseCommandsSender {
    queue: Arc<Mutex<VecDeque<AssetDatabaseCommand>>>,
}

impl AssetDatabaseCommandsSender {
    /// Sends a command to the asset database.
    ///
    /// # Arguments
    /// - `command`: The command to send.
    pub fn send(&self, command: AssetDatabaseCommand) {
        if let Ok(mut queue) = self.queue.lock() {
            queue.push_back(command);
        }
    }
}

/// Asset database for managing assets and their states.
#[derive(Default)]
pub struct AssetDatabase {
    pub storage: World,
    pub events: AssetEventBindings,
    pub allow_asset_progression_failures: bool,
    fetch_stack: Vec<AssetFetchEngine>,
    store_stack: Vec<AssetStoreEngine>,
    protocols: Vec<Box<dyn AssetProtocol>>,
    commands: Arc<Mutex<VecDeque<AssetDatabaseCommand>>>,
}

impl AssetDatabase {
    /// Adds a fetcher to its fetch stack.
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

    /// Adds a store to its store stack.
    ///
    /// # Arguments
    /// - `store`: A concrete implementation of the `AssetStore` trait.
    ///
    /// # Returns
    /// The updated `AssetDatabase` with the store added.
    pub fn with_store(mut self, store: impl AssetStore + 'static) -> Self {
        self.push_store(store);
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
    ///
    /// # Arguments
    /// - `fetch`: A new fetch implementation to add to the stack.
    pub fn push_fetch(&mut self, fetch: impl AssetFetch + 'static) {
        self.fetch_stack.push(AssetFetchEngine::new(fetch));
    }

    /// Removes and returns the top fetch engine from the stack.
    ///
    /// # Returns
    /// The old fetch engine if present.
    /// Returns `None` if the stack is empty.
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

    /// Adds a store engine to the stack.
    ///
    /// # Arguments
    /// - `store`: A new store implementation to add to the stack.
    pub fn push_store(&mut self, store: impl AssetStore + 'static) {
        self.store_stack.push(AssetStoreEngine::new(store));
    }

    /// Removes and returns the top store engine from the stack.
    ///
    /// # Returns
    /// The old store engine if present.
    /// Returns `None` if the stack is empty.
    pub fn pop_store(&mut self) -> Option<Box<dyn AssetStore>> {
        self.store_stack.pop().map(|store| store.into_inner())
    }

    /// Replaces the top store engine and returns the old one.
    ///
    /// # Arguments
    /// - `store`: A new store implementation to replace the top one.
    ///
    /// # Returns
    /// The old store engine if present.
    /// Returns `None` if the stack is empty.
    pub fn swap_store(&mut self, store: impl AssetStore + 'static) -> Option<Box<dyn AssetStore>> {
        let result = self.pop_store();
        self.store_stack.push(AssetStoreEngine::new(store));
        result
    }

    /// Temporarily uses a store engine to perform a closure and removes it afterward.
    ///
    /// # Arguments
    /// - `store`: The store engine to add temporarily.
    /// - `f`: The closure to execute using the store engine.
    ///
    /// # Returns
    /// The result of the closure if successful, or an error otherwise.
    pub fn using_store<R>(
        &mut self,
        store: impl AssetStore + 'static,
        f: impl FnOnce(&mut Self) -> Result<R, Box<dyn Error>>,
    ) -> Result<R, Box<dyn Error>> {
        self.push_store(store);
        let result = f(self)?;
        self.pop_store();
        Ok(result)
    }

    /// Registers a new protocol for processing assets.
    ///
    /// # Arguments
    /// - `protocol`: An implementation of the `AssetProtocol` trait.
    pub fn add_protocol(&mut self, protocol: impl AssetProtocol + 'static) {
        self.protocols.push(Box::new(protocol));
    }

    /// Removes a protocol by its name.
    ///
    /// # Arguments
    /// - `name`: The name of the protocol to remove.
    ///
    /// # Returns
    /// The removed protocol if found, otherwise `None`.
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
                    return Err(format!("Missing protocol for asset: `{path}`").into());
                };
                let status = protocol.process_asset_bytes(handle, &mut self.storage);
                if status.is_err()
                    && let Ok(mut bindings) = self
                        .storage
                        .component_mut::<true, AssetEventBindings>(handle.entity())
                {
                    bindings.dispatch(AssetEvent {
                        handle,
                        kind: AssetEventKind::BytesProcessingFailed,
                        path: path.clone(),
                    })?;
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

    /// Schedules an asset to be stored.
    ///
    /// # Arguments
    /// - `path`: The path of the asset to store.
    ///
    /// # Returns
    /// Result indicating success or failure.
    pub fn store(&mut self, path: impl Into<AssetPathStatic>) -> Result<(), Box<dyn Error>> {
        let path = path.into();
        let entity = self
            .storage
            .find_by::<true, _>(&path)
            .ok_or_else(|| format!("Asset `{path}` not found"))?;
        self.storage.insert(entity, (AssetAwaitsStoring,))?;
        Ok(())
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

    /// Returns an iterator over all assets awaiting async fetch.
    ///
    /// # Returns
    /// An iterator that yields `AssetHandle` instances.
    pub fn assets_awaiting_async_fetch(&self) -> impl Iterator<Item = AssetHandle> + '_ {
        self.storage
            .query::<true, (Entity, Include<AssetAwaitsAsyncFetch>)>()
            .map(|(entity, _)| AssetHandle::new(entity))
    }

    /// Checks if there are any assets awaiting storing.
    ///
    /// # Returns
    /// `true` if assets are awaiting storing, otherwise `false`.
    pub fn does_await_storing(&self) -> bool {
        self.storage.has_component::<AssetAwaitsStoring>()
    }

    /// Checks if there are any assets with bytes ready to store.
    ///
    /// # Returns
    /// `true` if assets are ready to store, otherwise `false`.
    pub fn has_bytes_ready_to_store(&self) -> bool {
        self.storage.has_component::<AssetBytesAreReadyToStore>()
    }

    /// Checks if there are assets awaiting async store.
    ///
    /// # Returns
    /// `true` if async stores are awaiting completion, otherwise `false`.
    pub fn does_await_async_store(&self) -> bool {
        self.storage.has_component::<AssetAwaitsAsyncStore>()
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

    /// Checks if there are assets awaiting async fetch.
    ///
    /// # Returns
    /// `true` if async fetches are awaiting completion, otherwise `false`.
    pub fn does_await_async_fetch(&self) -> bool {
        self.storage.has_component::<AssetAwaitsAsyncFetch>()
    }

    /// Determines if the asset database is currently busy with tasks.
    ///
    /// # Returns
    /// `true` if busy, otherwise `false`.
    pub fn is_busy(&self) -> bool {
        self.storage.has_component::<AssetAwaitsResolution>()
            || self.storage.has_component::<AssetBytesAreReadyToProcess>()
            || self.storage.has_component::<AssetAwaitsAsyncFetch>()
            || self.storage.has_component::<AssetAwaitsStoring>()
            || self.storage.has_component::<AssetBytesAreReadyToStore>()
            || self.storage.has_component::<AssetAwaitsAsyncStore>()
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
            asset_awaits_async_fetch,
        ) in self.storage.query::<true, (
            AssetHandle,
            Option<&AssetAwaitsResolution>,
            Option<&AssetBytesAreReadyToProcess>,
            Option<&AssetAwaitsAsyncFetch>,
        )>() {
            if asset_awaits_resolution.is_some() {
                out_status.awaiting_resolution.add(handle);
            } else if asset_bytes_ready_to_process.is_some() {
                out_status.with_bytes_ready_to_process.add(handle);
            } else if asset_awaits_async_fetch.is_some() {
                out_status.awaiting_async_fetch.add(handle);
            } else {
                out_status.ready_to_use.add(handle);
            }
        }
    }

    /// Returns the sender for asset database commands.
    /// This can be used to send commands to the asset database from external places.
    pub fn commands_sender(&self) -> AssetDatabaseCommandsSender {
        AssetDatabaseCommandsSender {
            queue: self.commands.clone(),
        }
    }

    /// Performs maintenance on the asset database, processing events and managing states.
    ///
    /// - Processes changed assets and dispatches relevant events.
    /// - Maintains fetch and store engines and protocols.
    /// - Resolves assets and processes their data using protocols.
    /// - Stores requested assets.
    ///
    /// # Returns
    /// `Ok(())` if successful, or an error if any step fails.
    pub fn maintain(&mut self) -> Result<(), Box<dyn Error>> {
        if let Ok(mut queue) = self.commands.lock() {
            while let Some(command) = queue.pop_front() {
                command(&mut self.storage);
            }
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
            for entity in self.storage.added().iter_of::<AssetAwaitsAsyncFetch>() {
                if let Some((path, bindings)) = lookup.access(entity) {
                    let event = AssetEvent {
                        handle: AssetHandle::new(entity),
                        kind: AssetEventKind::AwaitsAsyncFetch,
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
            for entity in self.storage.added().iter_of::<AssetAwaitsStoring>() {
                if let Some((path, bindings)) = lookup.access(entity) {
                    let event = AssetEvent {
                        handle: AssetHandle::new(entity),
                        kind: AssetEventKind::AwaitsStoring,
                        path: path.clone(),
                    };
                    self.events.dispatch(event.clone())?;
                    bindings.dispatch(event)?;
                }
            }
            for entity in self.storage.added().iter_of::<AssetAwaitsAsyncStore>() {
                if let Some((path, bindings)) = lookup.access(entity) {
                    let event = AssetEvent {
                        handle: AssetHandle::new(entity),
                        kind: AssetEventKind::AwaitsAsyncStore,
                        path: path.clone(),
                    };
                    self.events.dispatch(event.clone())?;
                    bindings.dispatch(event)?;
                }
            }
            for entity in self.storage.added().iter_of::<AssetBytesAreReadyToStore>() {
                if let Some((path, bindings)) = lookup.access(entity) {
                    let event = AssetEvent {
                        handle: AssetHandle::new(entity),
                        kind: AssetEventKind::BytesReadyToStore,
                        path: path.clone(),
                    };
                    self.events.dispatch(event.clone())?;
                    bindings.dispatch(event)?;
                }
            }
            for entity in self
                .storage
                .removed()
                .iter_of::<AssetBytesAreReadyToStore>()
            {
                if let Some((path, bindings)) = lookup.access(entity) {
                    let handle = AssetHandle::new(entity);
                    let event = AssetEvent {
                        handle,
                        kind: AssetEventKind::BytesStored,
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
        for store in &mut self.store_stack {
            store.maintain(&mut self.storage)?;
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
                let status = protocol.process_asset_bytes(handle, &mut self.storage);
                if status.is_err()
                    && let Ok(mut bindings) = self
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
                if !self.allow_asset_progression_failures {
                    status?;
                }
            }
            let to_produce = self
                .storage
                .query::<true, (Entity, &AssetPath, Include<AssetAwaitsStoring>)>()
                .filter(|(_, path, _)| path.protocol() == protocol.name())
                .map(|(entity, _, _)| AssetHandle::new(entity))
                .collect::<Vec<_>>();
            for handle in to_produce {
                let status = protocol.produce_asset_bytes(handle, &mut self.storage);
                if status.is_err() {
                    if let Ok(mut bindings) = self
                        .storage
                        .component_mut::<true, AssetEventBindings>(handle.entity())
                    {
                        bindings.dispatch(AssetEvent {
                            handle,
                            kind: AssetEventKind::BytesStoringFailed,
                            path: self
                                .storage
                                .component::<true, AssetPathStatic>(handle.entity())?
                                .clone(),
                        })?;
                    }
                } else {
                    self.storage
                        .remove::<(AssetAwaitsStoring,)>(handle.entity())?;
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
        if !to_resolve.is_empty() {
            if let Some(fetch) = self.fetch_stack.last_mut() {
                for (handle, path) in to_resolve {
                    let status = fetch.load_bytes(handle, path.clone(), &mut self.storage);
                    if !self.allow_asset_progression_failures {
                        status?;
                    }
                    self.storage
                        .remove::<(AssetAwaitsResolution,)>(handle.entity())?;
                }
            } else {
                return Err("There is no asset fetch on stack!".into());
            }
        }
        let to_store = self
            .storage
            .query::<true, (AssetHandle, &AssetPath, &mut AssetBytesAreReadyToStore)>()
            .map(|(handle, path, bytes)| (handle, path.clone(), std::mem::take(&mut bytes.0)))
            .collect::<Vec<_>>();
        if !to_store.is_empty() {
            if let Some(store) = self.store_stack.last_mut() {
                for (handle, path, bytes) in to_store {
                    let status = store.save_bytes(handle, path.clone(), bytes, &mut self.storage);
                    if !self.allow_asset_progression_failures {
                        status?;
                    }
                    self.storage
                        .remove::<(AssetBytesAreReadyToStore,)>(handle.entity())?;
                }
            } else {
                return Err("There is no asset store on stack!".into());
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
