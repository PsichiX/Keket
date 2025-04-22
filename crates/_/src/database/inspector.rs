use crate::database::{
    AssetDatabase,
    handle::{AssetDependency, AssetHandle},
    path::AssetPathStatic,
};
use anput::{entity::Entity, query::TypedLookupFetch, world::World};
use std::error::Error;

/// The `AssetInspector` struct provides a way to access asset path, its
/// components, as well as dependencies.
pub struct AssetInspector<'a> {
    storage: &'a World,
    entity: Entity,
}

impl<'a> AssetInspector<'a> {
    /// Creates a new `AssetInspector` instance for the given asset handle.
    ///
    /// # Arguments
    /// - `database` - A reference to the `AssetDatabase` containing the asset.
    /// - `handle` - The `AssetHandle` representing the asset to inspect.
    ///
    /// # Returns
    /// A new `AssetInspector` instance.
    pub fn new(database: &'a AssetDatabase, handle: AssetHandle) -> Self {
        Self {
            storage: &database.storage,
            entity: handle.entity(),
        }
    }

    /// Creates a new `AssetInspector` instance for the given asset entity.
    ///
    /// # Arguments
    /// - `storage` - A reference to the `World` containing the asset entity.
    /// - `entity` - The `Entity` representing the asset to inspect.
    ///
    /// # Returns
    /// A new `AssetInspector` instance.
    pub fn new_raw(storage: &'a World, entity: Entity) -> Self {
        Self { storage, entity }
    }

    /// Returns the assigned asset handle.
    pub fn handle(&self) -> AssetHandle {
        AssetHandle::new(self.entity)
    }

    /// Returns the asset path associated with the asset.
    pub fn path(&self) -> Result<AssetPathStatic, Box<dyn Error>> {
        Ok(self
            .storage
            .component::<true, AssetPathStatic>(self.entity)
            .map(|path| (*path).to_owned())?)
    }

    /// Tries to access typed data for this asset.
    pub fn access_checked<Fetch: TypedLookupFetch<'a, true>>(&self) -> Option<Fetch::Value> {
        self.storage
            .lookup_access::<'a, true, Fetch>()
            .access(self.entity)
    }

    /// Accesses typed data for this asset or panics if it cannot.
    pub fn access<Fetch: TypedLookupFetch<'a, true>>(&self) -> Fetch::Value {
        self.access_checked::<Fetch>().unwrap_or_else(|| {
            panic!(
                "Failed to get access to requested {:?} asset components!",
                self.entity
            )
        })
    }

    /// Returns an iterator over asset dependencies.
    pub fn dependencies(&'a self) -> impl Iterator<Item = Self> + 'a {
        self.storage
            .relations_outgoing::<true, AssetDependency>(self.entity)
            .map(|(_, _, entity)| Self {
                storage: self.storage,
                entity,
            })
    }

    /// Recursively iterates through all assigned asset dependencies.
    pub fn traverse_dependencies(&'a self) -> impl Iterator<Item = Self> + 'a {
        self.storage
            .traverse_outgoing::<true, AssetDependency>([self.entity])
            .map(|(_, entity)| Self {
                storage: self.storage,
                entity,
            })
    }
}
