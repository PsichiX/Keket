use crate::database::{
    AssetDatabase, AssetDatabaseCommandsSender, AssetReferenceCounter, handle::AssetHandle,
    path::AssetPathStatic,
};
use anput::{entity::Entity, query::TypedLookupFetch};
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    ops::{Deref, DerefMut},
    sync::RwLock,
};

/// A reference to an asset in the asset database.
///
/// `AssetRef` encapsulates a reference to an asset, identified by its path,
/// with an optional handle that can be lazily resolved or explicitly set.
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
    /// Creates a new `AssetRef` with the given asset path.
    ///
    /// # Arguments
    /// - `path`: The path to the asset.
    ///
    /// # Returns
    /// An instance of `AssetRef`.
    pub fn new(path: impl Into<AssetPathStatic>) -> Self {
        Self {
            path: path.into(),
            handle: RwLock::new(None),
        }
    }

    /// Creates a new resolved `AssetRef` with the given asset path and handle.
    ///
    /// # Arguments
    /// - `path`: The path to the asset.
    /// - `handle`: The resolved handle for the asset.
    ///
    /// # Returns
    /// A resolved instance of `AssetRef`.
    pub fn new_resolved(path: impl Into<AssetPathStatic>, handle: AssetHandle) -> Self {
        Self {
            path: path.into(),
            handle: RwLock::new(Some(handle)),
        }
    }

    /// Invalidates the asset handle, making the `AssetRef` unresolved.
    ///
    /// # Returns
    /// An error if the handle could not be invalidated.
    pub fn invalidate(&self) -> Result<(), Box<dyn Error>> {
        *self.handle.write().map_err(|error| format!("{error}"))? = None;
        Ok(())
    }

    /// Gets the asset path associated with this reference.
    ///
    /// # Returns
    /// A reference to the asset path.
    pub fn path(&self) -> &AssetPathStatic {
        &self.path
    }

    /// Gets the resolved handle for the asset.
    ///
    /// # Returns
    /// The resolved `AssetHandle`, or an error if it has not been resolved.
    pub fn handle(&self) -> Result<AssetHandle, Box<dyn Error>> {
        self.handle
            .read()
            .map_err(|error| format!("{error}"))?
            .ok_or_else(|| format!("Asset with `{}` path is not yet resolved!", self.path).into())
    }

    /// Resolves the asset handle using the asset database, if not already resolved.
    ///
    /// # Arguments
    /// - `database`: Reference to the `AssetDatabase` to resolve the asset.
    ///
    /// # Returns
    /// A resolved `AssetResolved` object, or an error if resolution fails.
    pub fn resolve<'a>(
        &'a self,
        database: &'a AssetDatabase,
    ) -> Result<AssetResolved<'a>, Box<dyn Error>> {
        let mut handle = self.handle.write().map_err(|error| format!("{error}"))?;
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

    /// Ensures existence of the asset with handle using the asset database.
    ///
    /// # Arguments
    /// - `database`: Reference to the `AssetDatabase` to ensure the asset.
    ///
    /// # Returns
    /// An ensured `AssetResolved` object, or an error if resolution fails.
    pub fn ensure<'a>(
        &'a self,
        database: &'a mut AssetDatabase,
    ) -> Result<AssetResolved<'a>, Box<dyn Error>> {
        let mut handle = self.handle.write().map_err(|error| format!("{error}"))?;
        if let Some(result) = handle.as_ref() {
            Ok(AssetResolved::new(*result, database))
        } else {
            let result = database.ensure(self.path.clone())?;
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

/// A wrapper for a resolved asset with direct access to its associated data.
///
/// `AssetResolved` provides additional utilities for checking asset state and accessing its data.
pub struct AssetResolved<'a> {
    handle: AssetHandle,
    database: &'a AssetDatabase,
}

impl<'a> AssetResolved<'a> {
    /// Creates a new resolved asset.
    ///
    /// # Arguments
    /// - `handle`: The resolved asset handle.
    /// - `database`: Reference to the `AssetDatabase`.
    ///
    /// # Returns
    /// An instance of `AssetResolved`.
    pub fn new(handle: AssetHandle, database: &'a AssetDatabase) -> Self {
        Self { handle, database }
    }

    /// Gets the entity associated with this asset.
    pub fn entity(&self) -> Entity {
        self.handle.entity()
    }

    /// Checks if the asset exists in the database.
    pub fn does_exists(&self) -> bool {
        self.handle.does_exists(self.database)
    }

    /// Checks if the asset is marked for storing its bytes.
    pub fn awaits_storing(self) -> bool {
        self.handle.awaits_storing(self.database)
    }

    /// Checks if the asset is marked for storing its bytes and is not already stored.
    pub fn bytes_are_ready_to_store(self) -> bool {
        self.handle.bytes_are_ready_to_store(self.database)
    }

    /// Checks if the asset is awaiting an async store.
    pub fn awaits_async_store(self) -> bool {
        self.handle.awaits_async_store(self.database)
    }

    /// Checks if the asset is awaiting resolution.
    pub fn awaits_resolution(&self) -> bool {
        self.handle.awaits_resolution(self.database)
    }

    /// Checks if the asset bytes are ready to be processed.
    pub fn bytes_are_ready_to_process(&self) -> bool {
        self.handle.bytes_are_ready_to_process(self.database)
    }

    /// Checks if the asset is awaiting an async fetch.
    pub fn awaits_async_fetch(&self) -> bool {
        self.handle.awaits_async_fetch(self.database)
    }

    /// Checks if the asset is ready to use.
    pub fn is_ready_to_use(&self) -> bool {
        self.handle.is_ready_to_use(self.database)
    }

    /// Waits asynchronously for the asset to be ready to use.
    pub async fn wait_for_ready_to_use(&self) {
        self.handle.wait_for_ready_to_use(self.database).await
    }

    /// Accesses the asset's typed data, if available.
    ///
    /// # Returns
    /// The asset's data or `None` if access fails.
    pub fn access_checked<'b, Fetch: TypedLookupFetch<'b, true>>(&'b self) -> Option<Fetch::Value> {
        self.handle.access_checked::<Fetch>(self.database)
    }

    /// Accesses the asset's typed data without additional checks.
    ///
    /// # Returns
    /// The asset's data.
    pub fn access<'b, Fetch: TypedLookupFetch<'b, true>>(&'b self) -> Fetch::Value {
        self.handle.access::<Fetch>(self.database)
    }

    /// Iterates over the asset's dependencies as `AssetRef` objects.
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

    /// Iterates over the assets dependent on this one as `AssetRef` objects.
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

    /// Iterates recursively over all dependencies as `AssetRef` objects.
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

/// A smart reference to an asset in the asset database.
/// Uses asset reference counting to ensure asset lifetime.
pub struct SmartAssetRef {
    inner: AssetRef,
    sender: AssetDatabaseCommandsSender,
}

impl Drop for SmartAssetRef {
    fn drop(&mut self) {
        if let Ok(handle) = self.inner.handle() {
            self.sender.send(Box::new(move |storage| {
                if let Ok(mut counter) =
                    storage.component_mut::<true, AssetReferenceCounter>(handle.entity())
                {
                    counter.decrement();
                    storage.update::<AssetReferenceCounter>(handle.entity());
                }
            }));
        }
    }
}

impl SmartAssetRef {
    /// Creates a new `SmartAssetRef` with the given asset path, ensuring it's
    /// existance and incrementing asset reference counter.
    ///
    /// # Arguments
    /// - `path`: The path to the asset.
    /// - `database`: Reference to the `AssetDatabase`.
    ///
    /// # Returns
    /// An instance of `SmartAssetRef`.
    pub fn new(
        path: impl Into<AssetPathStatic>,
        database: &mut AssetDatabase,
    ) -> Result<Self, Box<dyn Error>> {
        Self::from_ref(AssetRef::new(path), database)
    }

    /// Creates a new `SmartAssetRef` from an existing `AssetRef`, ensuring it's
    /// asset existance and incrementing asset reference counter.
    ///
    /// # Arguments
    /// - `inner`: The `AssetRef` to create the `SmartAssetRef` from.
    /// - `database`: Reference to the `AssetDatabase`.
    ///
    /// # Returns
    /// An instance of `SmartAssetRef`.
    pub fn from_ref(inner: AssetRef, database: &mut AssetDatabase) -> Result<Self, Box<dyn Error>> {
        let sender = database.commands_sender();
        let handle = inner.ensure(database)?.handle;
        handle
            .ensure::<AssetReferenceCounter>(database)?
            .increment();
        database
            .storage
            .update::<AssetReferenceCounter>(handle.entity());
        Ok(Self { inner, sender })
    }

    /// Converts the `SmartAssetRef` into a regular `AssetRef`, decrementing
    /// asset reference counter in the process.
    pub fn into_ref(self) -> AssetRef {
        self.inner.clone()
    }
}

impl Clone for SmartAssetRef {
    fn clone(&self) -> Self {
        if let Ok(handle) = self.inner.handle() {
            self.sender.send(Box::new(move |storage| {
                if let Ok(mut counter) =
                    storage.component_mut::<true, AssetReferenceCounter>(handle.entity())
                {
                    counter.increment();
                    storage.update::<AssetReferenceCounter>(handle.entity());
                }
            }));
        }
        Self {
            inner: self.inner.clone(),
            sender: self.sender.clone(),
        }
    }
}

impl Deref for SmartAssetRef {
    type Target = AssetRef;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for SmartAssetRef {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
