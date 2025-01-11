use crate::{
    database::AssetDatabase,
    fetch::{AssetBytesAreLoading, AssetBytesAreReadyToProcess},
};
use anput::{
    archetype::Archetype,
    bundle::Bundle,
    component::Component,
    entity::Entity,
    prelude::{Command, WorldDestroyIteratorExt},
    query::{Exclude, Include, QueryError, TypedLookupFetch, TypedQueryFetch},
};
use std::error::Error;

pub struct AssetDependency;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AssetRef {
    entity: Entity,
}

impl AssetRef {
    pub fn new(entity: Entity) -> Self {
        Self { entity }
    }

    pub fn delete(self, database: &mut AssetDatabase) {
        database
            .storage
            .traverse_outgoing::<true, AssetDependency>([self.entity])
            .to_despawn_command()
            .execute(&mut database.storage);
    }

    pub fn entity(self) -> Entity {
        self.entity
    }

    pub fn bytes_are_loading(self, database: &AssetDatabase) -> bool {
        self.access_checked::<(Entity, Include<AssetBytesAreLoading>)>(database)
            .is_some()
    }

    pub fn bytes_are_ready_to_process(self, database: &AssetDatabase) -> bool {
        self.access_checked::<(Entity, Include<AssetBytesAreReadyToProcess>)>(database)
            .is_some()
    }

    pub fn is_ready_to_use(self, database: &AssetDatabase) -> bool {
        self.access_checked::<(
            Entity,
            Exclude<AssetBytesAreLoading>,
            Exclude<AssetBytesAreReadyToProcess>,
        )>(database)
            .is_some()
    }

    pub fn give(
        self,
        database: &mut AssetDatabase,
        bundle: impl Bundle,
    ) -> Result<(), Box<dyn Error>> {
        Ok(database.storage.insert(self.entity, bundle)?)
    }

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

    pub fn access_checked<'a, Fetch: TypedLookupFetch<'a, true>>(
        self,
        database: &'a AssetDatabase,
    ) -> Option<Fetch::Value> {
        database
            .storage
            .lookup_access::<'a, true, Fetch>()
            .access(self.entity)
    }

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

    pub fn dependencies(self, database: &AssetDatabase) -> impl Iterator<Item = AssetRef> + '_ {
        database
            .storage
            .relations_outgoing::<true, AssetDependency>(self.entity)
            .map(|(_, _, entity)| Self { entity })
    }

    pub fn dependent(self, database: &AssetDatabase) -> impl Iterator<Item = AssetRef> + '_ {
        database
            .storage
            .relations_incomming::<true, AssetDependency>(self.entity)
            .map(|(entity, _, _)| Self { entity })
    }
}

impl<'a, const LOCKING: bool> TypedQueryFetch<'a, LOCKING> for AssetRef {
    type Value = AssetRef;
    type Access = <Entity as TypedQueryFetch<'a, LOCKING>>::Access;

    fn does_accept_archetype(_: &Archetype) -> bool {
        true
    }

    fn access(archetype: &'a Archetype) -> Result<Self::Access, QueryError> {
        <Entity as TypedQueryFetch<'a, LOCKING>>::access(archetype)
    }

    fn fetch(access: &mut Self::Access) -> Option<Self::Value> {
        <Entity as TypedQueryFetch<'a, LOCKING>>::fetch(access).map(AssetRef::new)
    }
}

impl<'a, const LOCKING: bool> TypedLookupFetch<'a, LOCKING> for AssetRef {
    type Value = AssetRef;
    type Access = <Entity as TypedLookupFetch<'a, LOCKING>>::Access;

    fn try_access(archetype: &'a Archetype) -> Option<Self::Access> {
        <Entity as TypedLookupFetch<'a, LOCKING>>::try_access(archetype)
    }

    fn fetch(access: &mut Self::Access, entity: Entity) -> Option<Self::Value> {
        <Entity as TypedLookupFetch<'a, LOCKING>>::fetch(access, entity).map(AssetRef::new)
    }
}
