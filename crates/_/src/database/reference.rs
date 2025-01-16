use crate::database::{handle::AssetHandle, path::AssetPathStatic, AssetDatabase};
use anput::{entity::Entity, query::TypedLookupFetch};
use serde::{Deserialize, Serialize};
use std::{error::Error, sync::RwLock};

#[derive(Debug, Serialize, Deserialize)]
#[serde(from = "AssetPathStatic", into = "AssetPathStatic")]
pub struct AssetRef {
    path: AssetPathStatic,
    #[serde(skip)]
    handle: RwLock<Option<AssetHandle>>,
}

impl Default for AssetRef {
    fn default() -> Self {
        Self::new("")
    }
}

impl AssetRef {
    pub fn new(path: impl Into<AssetPathStatic>) -> Self {
        Self {
            path: path.into(),
            handle: RwLock::new(None),
        }
    }

    pub fn new_resolved(path: impl Into<AssetPathStatic>, handle: AssetHandle) -> Self {
        Self {
            path: path.into(),
            handle: RwLock::new(Some(handle)),
        }
    }

    pub fn invalidate(&self) -> Result<(), Box<dyn Error>> {
        *self.handle.write().map_err(|error| format!("{}", error))? = None;
        Ok(())
    }

    pub fn path(&self) -> &AssetPathStatic {
        &self.path
    }

    pub fn handle(&self) -> Result<AssetHandle, Box<dyn Error>> {
        self.handle
            .read()
            .map_err(|error| format!("{}", error))?
            .ok_or_else(|| format!("Asset with `{}` path is not yet resolved!", self.path).into())
    }

    pub fn resolve<'a>(
        &'a self,
        database: &'a AssetDatabase,
    ) -> Result<AssetResolved<'a>, Box<dyn Error>> {
        let mut handle = self.handle.write().map_err(|error| format!("{}", error))?;
        if let Some(result) = handle.as_ref() {
            Ok(AssetResolved::new(*result, database))
        } else {
            let result = database
                .find(self.path.clone())
                .ok_or_else(|| format!("Asset with `{}` path not found in database!", self.path))?;
            *handle = Some(result);
            Ok(AssetResolved::new(result, database))
        }
    }
}

impl Clone for AssetRef {
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            handle: RwLock::new(
                self.handle
                    .try_read()
                    .ok()
                    .map(|handle| *handle)
                    .unwrap_or_default(),
            ),
        }
    }
}

impl PartialEq for AssetRef {
    fn eq(&self, other: &Self) -> bool {
        self.path.eq(&other.path)
    }
}

impl Eq for AssetRef {}

impl From<AssetPathStatic> for AssetRef {
    fn from(path: AssetPathStatic) -> Self {
        Self::new(path)
    }
}

impl From<AssetRef> for AssetPathStatic {
    fn from(value: AssetRef) -> Self {
        value.path
    }
}

pub struct AssetResolved<'a> {
    handle: AssetHandle,
    database: &'a AssetDatabase,
}

impl<'a> AssetResolved<'a> {
    pub fn new(handle: AssetHandle, database: &'a AssetDatabase) -> Self {
        Self { handle, database }
    }

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

    pub fn dependencies(&self) -> impl Iterator<Item = AssetRef> + '_ {
        self.handle
            .dependencies(self.database)
            .filter_map(|handle| {
                Some(AssetRef::new_resolved(
                    handle
                        .access_checked::<&AssetPathStatic>(self.database)?
                        .clone(),
                    handle,
                ))
            })
    }

    pub fn dependent(&self) -> impl Iterator<Item = AssetRef> + '_ {
        self.handle.dependent(self.database).filter_map(|handle| {
            Some(AssetRef::new_resolved(
                handle
                    .access_checked::<&AssetPathStatic>(self.database)?
                    .clone(),
                handle,
            ))
        })
    }

    pub fn traverse_dependencies(&self) -> impl Iterator<Item = AssetRef> + '_ {
        self.handle
            .traverse_dependencies(self.database)
            .filter_map(|handle| {
                Some(AssetRef::new_resolved(
                    handle
                        .access_checked::<&AssetPathStatic>(self.database)?
                        .clone(),
                    handle,
                ))
            })
    }
}
