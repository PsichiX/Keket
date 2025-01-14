use crate::database::{handle::AssetHandle, path::AssetPath, AssetDatabase};
use anput::{bundle::Bundle, component::Component, entity::Entity, query::TypedLookupFetch};
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "AssetPath", into = "AssetPath")]
pub struct AssetRef {
    path: AssetPath<'static>,
    #[serde(skip)]
    handle: Option<AssetHandle>,
}

impl AssetRef {
    pub fn new(path: impl Into<AssetPath<'static>>) -> Self {
        Self {
            path: path.into(),
            handle: None,
        }
    }

    pub fn path(&self) -> &AssetPath {
        &self.path
    }

    pub fn handle(&self) -> Option<AssetHandle> {
        self.handle
    }

    pub fn is_resolved(&self) -> bool {
        self.handle.is_some()
    }

    pub fn resolve<'a>(
        &mut self,
        database: &'a mut AssetDatabase,
    ) -> Result<AssetResolved<'a>, Box<dyn Error>> {
        if let Some(handle) = self.handle {
            Ok(AssetResolved { handle, database })
        } else {
            let handle = database.ensure(self.path.clone())?;
            self.handle = Some(handle);
            Ok(AssetResolved { handle, database })
        }
    }

    pub fn try_resolve<'a>(
        &self,
        database: &'a mut AssetDatabase,
    ) -> Result<AssetResolved<'a>, Box<dyn Error>> {
        if let Some(handle) = self.handle {
            Ok(AssetResolved { handle, database })
        } else {
            let handle = database.ensure(self.path.clone())?;
            Ok(AssetResolved { handle, database })
        }
    }
}

impl From<AssetPath<'static>> for AssetRef {
    fn from(path: AssetPath<'static>) -> Self {
        Self { path, handle: None }
    }
}

impl From<AssetRef> for AssetPath<'static> {
    fn from(value: AssetRef) -> Self {
        value.path
    }
}

pub struct AssetResolved<'a> {
    handle: AssetHandle,
    database: &'a mut AssetDatabase,
}

impl AssetResolved<'_> {
    pub fn entity(&self) -> Entity {
        self.handle.entity()
    }

    pub fn does_exists(&self) -> bool {
        self.handle.does_exists(self.database)
    }

    pub fn awaits_resolution(&self) -> bool {
        self.handle.awaits_resolution(self.database)
    }

    pub fn bytes_are_ready_to_process(&self) -> bool {
        self.handle.bytes_are_ready_to_process(self.database)
    }

    pub fn awaits_deferred_job(&self) -> bool {
        self.handle.awaits_deferred_job(self.database)
    }

    pub fn is_ready_to_use(&self) -> bool {
        self.handle.is_ready_to_use(self.database)
    }

    pub fn access_checked<'b, Fetch: TypedLookupFetch<'b, true>>(&'b self) -> Option<Fetch::Value> {
        self.handle.access_checked::<Fetch>(self.database)
    }

    pub fn access<'b, Fetch: TypedLookupFetch<'b, true>>(&'b self) -> Fetch::Value {
        self.handle.access::<Fetch>(self.database)
    }

    pub fn delete(&mut self) {
        self.handle.delete(self.database);
    }

    pub fn refresh(&mut self) -> Result<(), Box<dyn Error>> {
        self.handle.refresh(self.database)
    }

    pub fn give(&mut self, bundle: impl Bundle) -> Result<(), Box<dyn Error>> {
        self.handle.give(self.database, bundle)
    }

    pub fn take<T: Component + Default>(&mut self) -> Result<T, Box<dyn Error>> {
        self.handle.take(self.database)
    }

    pub fn dependencies(&mut self) -> impl Iterator<Item = AssetRef> + '_ {
        self.handle
            .dependencies(self.database)
            .map(|handle| AssetRef {
                path: handle
                    .access::<&AssetPath>(self.database)
                    .clone()
                    .into_static(),
                handle: Some(handle),
            })
    }

    pub fn dependent(&mut self) -> impl Iterator<Item = AssetRef> + '_ {
        self.handle.dependent(self.database).map(|handle| AssetRef {
            path: handle
                .access::<&AssetPath>(self.database)
                .clone()
                .into_static(),
            handle: Some(handle),
        })
    }

    pub fn traverse_dependencies(&mut self) -> impl Iterator<Item = AssetRef> + '_ {
        self.handle
            .traverse_dependencies(self.database)
            .map(|handle| AssetRef {
                path: handle
                    .access::<&AssetPath>(self.database)
                    .clone()
                    .into_static(),
                handle: Some(handle),
            })
    }
}
