use crate::{
    database::AssetDatabase,
    fetch::{AssetAwaitsResolution, AssetBytesAreReadyToProcess, deferred::AssetAwaitsDeferredJob},
    store::{AssetAwaitsStoring, AssetBytesAreReadyToStore},
};
use anput::{
    archetype::Archetype,
    bundle::Bundle,
    component::Component,
    entity::Entity,
    prelude::{Command, ComponentRefMut, WorldDestroyIteratorExt},
    query::{Exclude, Include, QueryError, TypedLookupFetch, TypedQueryFetch},
    world::World,
};
use std::error::Error;

/// A marker struct to represent an asset dependency relationship.
pub struct AssetDependency;

/// Represents a handle to a specific asset in the asset database.
///
/// This handle can perform operations like deleting, refreshing, resolving dependencies,
/// and accessing asset data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AssetHandle {
    entity: Entity,
}

impl AssetHandle {
    /// Creates a new asset handle from an entity.
    ///
    /// # Arguments
    /// - `entity`: The entity representing the asset.
    pub fn new(entity: Entity) -> Self {
        Self { entity }
    }

    /// Deletes the asset and its dependencies from the database.
    ///
    /// # Arguments
    /// - `database`: A mutable reference to the asset database.
    pub fn delete(self, database: &mut AssetDatabase) {
        database
            .storage
            .traverse_outgoing::<true, AssetDependency>([self.entity])
            .map(|(_, entity)| entity)
            .to_despawn_command()
            .execute(&mut database.storage);
    }

    /// Refreshes the asset, marking it for resolution.
    ///
    /// # Arguments
    /// - `database`: A mutable reference to the asset database.
    ///
    /// # Returns
    /// A `Result` indicating success or failure.
    pub fn refresh(self, database: &mut AssetDatabase) -> Result<(), Box<dyn Error>> {
        database
            .storage
            .insert(self.entity, (AssetAwaitsResolution,))?;
        Ok(())
    }

    /// Schedules the asset to store its bytes.
    ///
    /// # Arguments
    /// - `database`: A mutable reference to the asset database.
    ///
    /// # Returns
    /// A `Result` indicating success or failure.
    pub fn store(self, database: &mut AssetDatabase) -> Result<(), Box<dyn Error>> {
        database
            .storage
            .insert(self.entity, (AssetAwaitsStoring,))?;
        Ok(())
    }

    /// Returns the entity associated with this handle.
    pub fn entity(self) -> Entity {
        self.entity
    }

    /// Checks if the asset exists in the database.
    pub fn does_exists(self, database: &AssetDatabase) -> bool {
        database.storage.has_entity(self.entity)
    }

    /// Checks if the asset is marked for storing its bytes.
    pub fn awaits_storing(self, database: &AssetDatabase) -> bool {
        self.access_checked::<(Entity, Include<AssetAwaitsStoring>)>(database)
            .is_some()
    }

    /// Checks if the asset is marked for storing its bytes and is not already stored.
    pub fn bytes_are_ready_to_store(self, database: &AssetDatabase) -> bool {
        self.access_checked::<(Entity, Include<AssetBytesAreReadyToStore>)>(database)
            .is_some()
    }

    /// Checks if the asset is awaiting resolution.
    pub fn awaits_resolution(self, database: &AssetDatabase) -> bool {
        self.access_checked::<(Entity, Include<AssetAwaitsResolution>)>(database)
            .is_some()
    }

    /// Checks if the asset bytes are ready for processing.
    pub fn bytes_are_ready_to_process(self, database: &AssetDatabase) -> bool {
        self.access_checked::<(Entity, Include<AssetBytesAreReadyToProcess>)>(database)
            .is_some()
    }

    /// Checks if the asset is awaiting a deferred job.
    pub fn awaits_deferred_job(self, database: &AssetDatabase) -> bool {
        self.access_checked::<(Entity, Include<AssetAwaitsDeferredJob>)>(database)
            .is_some()
    }

    /// Checks if the asset is ready for use (all dependencies are resolved).
    pub fn is_ready_to_use(self, database: &AssetDatabase) -> bool {
        let mut lookup = database.storage.lookup_access::<true, (
            Entity,
            Exclude<AssetAwaitsResolution>,
            Exclude<AssetBytesAreReadyToProcess>,
            Exclude<AssetAwaitsDeferredJob>,
        )>();
        database
            .storage
            .traverse_outgoing::<true, AssetDependency>([self.entity])
            .all(|(_, entity)| lookup.access(entity).is_some())
    }

    /// Adds a bundle of components to the asset.
    ///
    /// # Arguments
    /// - `bundle`: The components to be added to the asset.
    ///
    /// # Returns
    /// A `Result` indicating success or failure.
    pub fn give(
        self,
        database: &mut AssetDatabase,
        bundle: impl Bundle,
    ) -> Result<(), Box<dyn Error>> {
        Ok(database.storage.insert(self.entity, bundle)?)
    }

    /// Takes and removes a specific component from the asset.
    ///
    /// # Arguments
    /// - `database`: A mutable reference to the asset database.
    ///
    /// # Returns
    /// The removed component or an error.
    pub fn take<T: Component + Default>(
        self,
        database: &mut AssetDatabase,
    ) -> Result<T, Box<dyn Error>> {
        let mut component = database.storage.component_mut::<true, T>(self.entity)?;
        let result = std::mem::take(&mut *component);
        drop(component);
        database.storage.remove::<(T,)>(self.entity)?;
        Ok(result)
    }

    /// Ensures that a component exists on the asset, adding a default instance if missing.
    ///
    /// # Arguments
    /// - `database`: A mutable reference to the asset database.
    ///
    /// # Returns
    /// A mutable reference to the component.
    pub fn ensure<T: Component + Default>(
        self,
        database: &mut AssetDatabase,
    ) -> Result<ComponentRefMut<true, T>, Box<dyn Error>> {
        if !database.storage.has_entity_component::<T>(self.entity) {
            database.storage.insert(self.entity, (T::default(),))?;
        }
        Ok(database.storage.component_mut::<true, T>(self.entity)?)
    }

    /// Tries to access typed data for this asset.
    pub fn access_checked<'a, Fetch: TypedLookupFetch<'a, true>>(
        self,
        database: &'a AssetDatabase,
    ) -> Option<Fetch::Value> {
        database
            .storage
            .lookup_access::<'a, true, Fetch>()
            .access(self.entity)
    }

    /// Accesses typed data for this asset or panics if it cannot.
    pub fn access<'a, Fetch: TypedLookupFetch<'a, true>>(
        self,
        database: &'a AssetDatabase,
    ) -> Fetch::Value {
        self.access_checked::<'a, Fetch>(database)
            .unwrap_or_else(|| {
                panic!(
                    "Failed to get access to requested {:?} asset components!",
                    self.entity
                )
            })
    }

    /// Returns an iterator over asset dependencies.
    pub fn dependencies(self, database: &AssetDatabase) -> impl Iterator<Item = AssetHandle> + '_ {
        database
            .storage
            .relations_outgoing::<true, AssetDependency>(self.entity)
            .map(|(_, _, entity)| Self { entity })
    }

    /// Returns an iterator over assets dependent on this one.
    pub fn dependent(self, database: &AssetDatabase) -> impl Iterator<Item = AssetHandle> + '_ {
        database
            .storage
            .relations_incomming::<true, AssetDependency>(self.entity)
            .map(|(entity, _, _)| Self { entity })
    }

    /// Recursively iterates through all dependencies.
    pub fn traverse_dependencies(
        self,
        database: &AssetDatabase,
    ) -> impl Iterator<Item = AssetHandle> + '_ {
        database
            .storage
            .traverse_outgoing::<true, AssetDependency>([self.entity])
            .map(|(_, entity)| Self { entity })
    }
}

impl<'a, const LOCKING: bool> TypedQueryFetch<'a, LOCKING> for AssetHandle {
    type Value = AssetHandle;
    type Access = <Entity as TypedQueryFetch<'a, LOCKING>>::Access;

    fn does_accept_archetype(_: &Archetype) -> bool {
        true
    }

    fn access(archetype: &'a Archetype) -> Result<Self::Access, QueryError> {
        <Entity as TypedQueryFetch<'a, LOCKING>>::access(archetype)
    }

    fn fetch(access: &mut Self::Access) -> Option<Self::Value> {
        <Entity as TypedQueryFetch<'a, LOCKING>>::fetch(access).map(AssetHandle::new)
    }
}

impl<'a, const LOCKING: bool> TypedLookupFetch<'a, LOCKING> for AssetHandle {
    type Value = AssetHandle;
    type ValueOne = AssetHandle;
    type Access = <Entity as TypedLookupFetch<'a, LOCKING>>::Access;

    fn try_access(archetype: &'a Archetype) -> Option<Self::Access> {
        <Entity as TypedLookupFetch<'a, LOCKING>>::try_access(archetype)
    }

    fn fetch(access: &mut Self::Access, entity: Entity) -> Option<Self::Value> {
        <Entity as TypedLookupFetch<'a, LOCKING>>::fetch(access, entity).map(AssetHandle::new)
    }

    fn fetch_one(world: &'a World, entity: Entity) -> Option<Self::ValueOne> {
        <Entity as TypedLookupFetch<'a, LOCKING>>::fetch_one(world, entity).map(AssetHandle::new)
    }
}
