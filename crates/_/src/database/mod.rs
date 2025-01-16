pub mod events;
pub mod handle;
pub mod path;
pub mod reference;
pub mod tags;

use crate::{
    database::{
        events::{AssetEvent, AssetEventBindings, AssetEventKind},
        handle::{AssetDependency, AssetHandle},
        path::{AssetPath, AssetPathStatic},
    },
    fetch::{
        deferred::AssetAwaitsDeferredJob, AssetAwaitsResolution, AssetBytesAreReadyToProcess,
        AssetFetch, AssetFetchEngine,
    },
    protocol::AssetProtocol,
};
use anput::{
    commands::Command, database::WorldDestroyIteratorExt, entity::Entity, query::Include,
    world::World,
};
use std::error::Error;

#[derive(Default)]
pub struct AssetDatabase {
    pub storage: World,
    pub events: AssetEventBindings,
    pub allow_asset_progression_failures: bool,
    fetch_stack: Vec<AssetFetchEngine>,
    protocols: Vec<Box<dyn AssetProtocol>>,
}

impl AssetDatabase {
    pub fn with_fetch(mut self, fetch: impl AssetFetch + 'static) -> Self {
        self.push_fetch(fetch);
        self
    }

    pub fn with_protocol(mut self, protocol: impl AssetProtocol + 'static) -> Self {
        self.add_protocol(protocol);
        self
    }

    pub fn with_asset_progression_failures(mut self) -> Self {
        self.allow_asset_progression_failures = true;
        self
    }

    pub fn push_fetch(&mut self, fetch: impl AssetFetch + 'static) {
        self.fetch_stack.push(AssetFetchEngine::new(fetch));
    }

    pub fn pop_fetch(&mut self) -> Option<Box<dyn AssetFetch>> {
        self.fetch_stack.pop().map(|fetch| fetch.into_inner())
    }

    pub fn swap_fetch(&mut self, fetch: impl AssetFetch + 'static) -> Option<Box<dyn AssetFetch>> {
        let result = self.pop_fetch();
        self.fetch_stack.push(AssetFetchEngine::new(fetch));
        result
    }

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

    pub fn add_protocol(&mut self, protocol: impl AssetProtocol + 'static) {
        self.protocols.push(Box::new(protocol));
    }

    pub fn remove_protocol(&mut self, name: &str) -> Option<Box<dyn AssetProtocol>> {
        self.protocols
            .iter()
            .position(|protocol| protocol.name() == name)
            .map(|index| self.protocols.remove(index))
    }

    pub fn find(&self, path: impl Into<AssetPathStatic>) -> Option<AssetHandle> {
        let path = path.into();
        self.storage.find_by::<true, _>(&path).map(AssetHandle::new)
    }

    pub fn schedule(
        &mut self,
        path: impl Into<AssetPathStatic>,
    ) -> Result<AssetHandle, Box<dyn Error>> {
        let path = path.into();
        Ok(AssetHandle::new(
            self.storage.spawn((path, AssetAwaitsResolution))?,
        ))
    }

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

    pub fn unload<'a>(&mut self, path: impl Into<AssetPath<'a>>) {
        let path = path.into();
        let to_remove = self
            .storage
            .query::<true, (Entity, &AssetPath)>()
            .filter(|(_, p)| *p == &path)
            .map(|(entity, _)| entity);
        self.storage
            .traverse_outgoing::<true, AssetDependency>(to_remove)
            .to_despawn_command()
            .execute(&mut self.storage)
    }

    pub fn reload(
        &mut self,
        path: impl Into<AssetPathStatic>,
    ) -> Result<AssetHandle, Box<dyn Error>> {
        let path = path.into();
        self.unload(path.clone());
        self.ensure(path)
    }

    pub fn assets_awaiting_resolution(&self) -> impl Iterator<Item = AssetHandle> + '_ {
        self.storage
            .query::<true, (Entity, Include<AssetAwaitsResolution>)>()
            .map(|(entity, _)| AssetHandle::new(entity))
    }

    pub fn assets_with_bytes_ready_to_process(&self) -> impl Iterator<Item = AssetHandle> + '_ {
        self.storage
            .query::<true, (Entity, Include<AssetBytesAreReadyToProcess>)>()
            .map(|(entity, _)| AssetHandle::new(entity))
    }

    pub fn assets_awaiting_deferred_job(&self) -> impl Iterator<Item = AssetHandle> + '_ {
        self.storage
            .query::<true, (Entity, Include<AssetAwaitsDeferredJob>)>()
            .map(|(entity, _)| AssetHandle::new(entity))
    }

    pub fn does_await_resolution(&self) -> bool {
        self.storage.has_component::<AssetAwaitsResolution>()
    }

    pub fn has_bytes_ready_to_process(&self) -> bool {
        self.storage.has_component::<AssetBytesAreReadyToProcess>()
    }

    pub fn does_await_deferred_job(&self) -> bool {
        self.storage.has_component::<AssetAwaitsDeferredJob>()
    }

    pub fn is_busy(&self) -> bool {
        self.storage.has_component::<AssetAwaitsResolution>()
            || self.storage.has_component::<AssetBytesAreReadyToProcess>()
            || self.storage.has_component::<AssetAwaitsDeferredJob>()
    }

    pub fn maintain(&mut self) -> Result<(), Box<dyn Error>> {
        {
            let mut lookup = self
                .storage
                .lookup_access::<true, &mut AssetEventBindings>();
            for entity in self.storage.added().iter_of::<AssetAwaitsResolution>() {
                let event = AssetEvent {
                    handle: AssetHandle::new(entity),
                    kind: AssetEventKind::AwaitsResolution,
                };
                self.events.dispatch(event)?;
                if let Some(bindings) = lookup.access(entity) {
                    bindings.dispatch(event)?;
                }
            }
            for entity in self.storage.added().iter_of::<AssetAwaitsDeferredJob>() {
                let event = AssetEvent {
                    handle: AssetHandle::new(entity),
                    kind: AssetEventKind::AwaitsDeferredJob,
                };
                self.events.dispatch(event)?;
                if let Some(bindings) = lookup.access(entity) {
                    bindings.dispatch(event)?;
                }
            }
            for entity in self
                .storage
                .added()
                .iter_of::<AssetBytesAreReadyToProcess>()
            {
                let event = AssetEvent {
                    handle: AssetHandle::new(entity),
                    kind: AssetEventKind::BytesReadyToProcess,
                };
                self.events.dispatch(event)?;
                if let Some(bindings) = lookup.access(entity) {
                    bindings.dispatch(event)?;
                }
            }
            for entity in self
                .storage
                .removed()
                .iter_of::<AssetBytesAreReadyToProcess>()
            {
                let handle = AssetHandle::new(entity);
                if handle.is_ready_to_use(self) {
                    continue;
                }
                let event = AssetEvent {
                    handle,
                    kind: AssetEventKind::BytesProcessed,
                };
                self.events.dispatch(event)?;
                if let Some(bindings) = lookup.access(entity) {
                    bindings.dispatch(event)?;
                }
            }
            for entity in self.storage.removed().iter_of::<AssetPathStatic>() {
                let event = AssetEvent {
                    handle: AssetHandle::new(entity),
                    kind: AssetEventKind::Unloaded,
                };
                self.events.dispatch(event)?;
                if let Some(bindings) = lookup.access(entity) {
                    bindings.dispatch(event)?;
                }
            }
        }
        self.storage.clear_changes();
        for fetch in &mut self.fetch_stack {
            fetch.maintain(&mut self.storage)?;
        }
        for protocol in &mut self.protocols {
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
