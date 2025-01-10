pub mod path;
pub mod reference;

use crate::{database::path::AssetPath, fetch::AssetFetch, protocol::AssetProtocol};
use anput::{commands::Command, database::WorldDestroyIteratorExt, entity::Entity, world::World};
use reference::AssetRef;
use std::error::Error;

#[derive(Default)]
pub struct AssetDatabase {
    pub storage: World,
    fetch_stack: Vec<Box<dyn AssetFetch>>,
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

    pub fn push_fetch(&mut self, fetch: impl AssetFetch + 'static) {
        self.fetch_stack.push(Box::new(fetch));
    }

    pub fn pop_fetch(&mut self) -> Option<Box<dyn AssetFetch>> {
        self.fetch_stack.pop()
    }

    pub fn swap_fetch(&mut self, fetch: impl AssetFetch + 'static) -> Option<Box<dyn AssetFetch>> {
        let result = self.pop_fetch();
        self.fetch_stack.push(Box::new(fetch));
        result
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

    pub fn ensure(&mut self, path: AssetPath<'static>) -> Result<AssetRef, Box<dyn Error>> {
        if let Some(entity) = self.storage.find_by::<true, _>(&path) {
            return Ok(AssetRef::new(entity));
        }
        if let Some(fetch) = self.fetch_stack.last_mut() {
            let result = fetch.load_bytes(path.clone(), &mut self.storage)?;
            if result.bytes_are_ready_to_process(self) {
                let Some(protocol) = self
                    .protocols
                    .iter_mut()
                    .find(|protocol| protocol.name() == path.protocol())
                else {
                    return Err(format!("Missing protocol for asset: `{}`", path).into());
                };
                protocol.process_asset(result, &mut self.storage)?;
            }
            Ok(result)
        } else {
            Err("There is no asset fetch on stack!".into())
        }
    }

    pub fn unload(&mut self, path: &AssetPath) {
        self.storage
            .query::<true, (Entity, &AssetPath)>()
            .filter(|(_, p)| *p == path)
            .map(|(entity, _)| entity)
            .to_despawn_command()
            .execute(&mut self.storage);
    }

    pub fn reload(&mut self, path: AssetPath<'static>) -> Result<AssetRef, Box<dyn Error>> {
        self.unload(&path);
        self.ensure(path)
    }

    pub fn maintain(&mut self) -> Result<(), Box<dyn Error>> {
        for fetch in &mut self.fetch_stack {
            fetch.maintain(&mut self.storage)?;
        }
        for protocol in &mut self.protocols {
            protocol.process_assets(&mut self.storage)?;
        }
        Ok(())
    }
}
