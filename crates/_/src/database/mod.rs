pub mod path;
pub mod reference;

use crate::{
    database::{
        path::AssetPath,
        reference::{AssetDependency, AssetRef},
    },
    fetch::AssetFetch,
    protocol::AssetProtocol,
};
use anput::{commands::Command, database::WorldDestroyIteratorExt, entity::Entity, world::World};
use std::{borrow::Cow, collections::HashSet, error::Error};

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

    pub fn ensure(
        &mut self,
        path: impl Into<AssetPath<'static>>,
    ) -> Result<AssetRef, Box<dyn Error>> {
        let path = path.into();
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
        path: impl Into<AssetPath<'static>>,
    ) -> Result<AssetRef, Box<dyn Error>> {
        let path = path.into();
        self.unload(path.clone());
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

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct AssetTags {
    tags: HashSet<Cow<'static, str>>,
}

impl AssetTags {
    pub fn new(tag: impl Into<Cow<'static, str>>) -> Self {
        let mut tags = HashSet::with_capacity(1);
        tags.insert(tag.into());
        Self { tags }
    }

    pub fn with(mut self, tag: impl Into<Cow<'static, str>>) -> Self {
        self.add(tag);
        self
    }

    pub fn len(&self) -> usize {
        self.tags.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tags.is_empty()
    }

    pub fn has(&mut self, tag: &str) {
        self.tags.contains(tag);
    }

    pub fn add(&mut self, tag: impl Into<Cow<'static, str>>) {
        self.tags.insert(tag.into());
    }

    pub fn remove(&mut self, tag: &str) {
        self.tags.remove(tag);
    }

    pub fn iter(&self) -> impl Iterator<Item = &str> + '_ {
        self.tags.iter().map(|tag| tag.as_ref())
    }

    pub fn is_subset_of(&self, other: &Self) -> bool {
        self.tags.is_subset(&other.tags)
    }

    pub fn is_superset_of(&self, other: &Self) -> bool {
        self.tags.is_superset(&other.tags)
    }

    pub fn intersection(&self, other: &Self) -> Self {
        self.tags.intersection(&other.tags).cloned().collect()
    }

    pub fn union(&self, other: &Self) -> Self {
        self.tags.union(&other.tags).cloned().collect()
    }
}

impl FromIterator<Cow<'static, str>> for AssetTags {
    fn from_iter<T: IntoIterator<Item = Cow<'static, str>>>(iter: T) -> Self {
        Self {
            tags: iter.into_iter().collect(),
        }
    }
}