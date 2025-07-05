use crate::{
    database::path::{AssetPath, AssetPathStatic},
    fetch::{AssetBytesAreReadyToProcess, AssetFetch},
};
use anput::{
    bundle::DynamicBundle,
    commands::{CommandBuffer, InsertCommand, RemoveCommand},
    component::Component,
    entity::Entity,
    query::Include,
    world::World,
};
use std::error::Error;

/// Marker component representing that an asset is awaiting extraction from storage.
pub struct AssetAwaitsExtractionFromStorage;

/// A handler for asset extraction that implements `AssetFetch`.
///
/// This struct is used to define the extraction logic for assets
/// awaiting extraction from storage.
pub struct ExtractAssetFetch {
    #[allow(clippy::type_complexity)]
    extract: Box<
        dyn FnMut(Entity, AssetPath, &World, &mut CommandBuffer) -> Result<(), Box<dyn Error>>
            + Send
            + Sync,
    >,
}

impl ExtractAssetFetch {
    /// Creates a new `ExtractAssetFetch` instance.
    ///
    /// # Arguments
    /// - `extract`: A function that performs asset extraction.
    ///
    /// # Returns
    /// A new `ExtractAssetFetch` instance.
    pub fn new(
        extract: impl FnMut(Entity, AssetPath, &World, &mut CommandBuffer) -> Result<(), Box<dyn Error>>
        + Send
        + Sync
        + 'static,
    ) -> Self {
        Self {
            extract: Box::new(extract),
        }
    }
}

impl AssetFetch for ExtractAssetFetch {
    fn load_bytes(&self, _: AssetPath) -> Result<DynamicBundle, Box<dyn Error>> {
        Ok(DynamicBundle::new(AssetAwaitsExtractionFromStorage)
            .ok()
            .unwrap())
    }

    fn maintain(&mut self, storage: &mut World) -> Result<(), Box<dyn Error>> {
        if !storage.has_component::<AssetAwaitsExtractionFromStorage>() {
            return Ok(());
        }
        let to_extract = storage
            .query::<true, (
                Entity,
                &AssetPath,
                Include<AssetAwaitsExtractionFromStorage>,
            )>()
            .map(|(entity, path, _)| (entity, path.clone()))
            .collect::<Vec<_>>();
        let mut commands = CommandBuffer::default();
        for (entity, path) in to_extract {
            (self.extract)(entity, path, storage, &mut commands)?;
        }
        commands.execute(storage);
        Ok(())
    }
}

/// Creates a function to extract asset bytes from a specific asset component.
///
/// This function is designed to handle components associated with a specific asset source.
///
/// # Arguments
/// - `source`: The asset path identifying the source asset.
/// - `extract`: A function that extracts bytes from the component with asset path.
///
/// # Returns
/// A closure suitable for use with `ExtractAssetFetch`.
#[allow(clippy::type_complexity)]
pub fn from_asset_extractor<T: Component>(
    source: impl Into<AssetPathStatic>,
    extract: impl Fn(&T, AssetPath) -> Result<Vec<u8>, Box<dyn Error>> + Send + Sync,
) -> impl FnMut(Entity, AssetPath, &World, &mut CommandBuffer) -> Result<(), Box<dyn Error>> + Send + Sync
{
    let source = source.into();
    let mut source_entity = None;
    move |entity: Entity,
          path: AssetPath,
          storage: &World,
          commands: &mut CommandBuffer|
          -> Result<(), Box<dyn Error>> {
        if source_entity.is_none() {
            source_entity = storage.find_by::<true, _>(&source);
        }
        let Some(source_entity) = source_entity else {
            return Err(format!("Source asset: `{source}` does not exists!").into());
        };
        let Ok(component) = storage.component::<true, T>(source_entity) else {
            return Err(format!(
                "Source asset: `{}` does not have component: `{}`",
                source,
                std::any::type_name::<T>()
            )
            .into());
        };
        let bytes = extract(&*component, path)?;
        commands.command(InsertCommand::new(
            entity,
            (AssetBytesAreReadyToProcess(bytes),),
        ));
        commands.command(RemoveCommand::<(AssetAwaitsExtractionFromStorage,)>::new(
            entity,
        ));
        Ok(())
    }
}
