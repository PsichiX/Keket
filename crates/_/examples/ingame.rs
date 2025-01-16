use anput::{
    prefab::Prefab,
    prelude::*,
    processor::WorldProcessor,
    third_party::{
        intuicio_core::prelude::{NativeStructBuilder, Registry},
        intuicio_framework_serde::SerializationRegistry,
    },
};
use keket::{
    database::{handle::AssetHandle, path::AssetPathStatic, reference::AssetRef, AssetDatabase},
    fetch::file::FileAssetFetch,
    protocol::{
        bundle::{BundleAssetProtocol, BundleWithDependencies, BundleWithDependenciesProcessor},
        group::GroupAssetProtocol,
        text::TextAssetProtocol,
    },
};
use serde::{Deserialize, Serialize};
use std::{error::Error, io::stdin};

fn main() -> Result<(), Box<dyn Error>> {
    // Define game plugin.
    struct GamePlugin;
    let game = GraphSchedulerQuickPlugin::<true, GamePlugin>::default()
        .with_resource::<Registry>(|registry| {
            // Setup reflection registry for prefab deserialization.
            registry.add_type(NativeStructBuilder::new::<ImageRenderable>().build());
        })
        .with_resource::<SerializationRegistry>(|registry| {
            // Setup serialization registry for prefab deserialization.
            registry.register_serde::<ImageRenderable>();
        })
        .resource(
            // Setup database.
            AssetDatabase::default()
                .with_protocol(TextAssetProtocol)
                .with_protocol(GroupAssetProtocol)
                .with_protocol(BundleAssetProtocol::new("prefab", PrefabAssetProcessor))
                .with_fetch(FileAssetFetch::default().with_root("./resources/")),
        )
        .system(asset_database_maintain, "asset_database_maintain", ())
        .system(
            override_world_from_prefab_asset,
            "override_world_from_prefab_asset",
            (),
        )
        .system(render_images, "render_images", ())
        .commit();

    // Setup universe.
    let mut universe = Universe::default()
        .with_basics(10240, 10240)
        .with_plugin(game);

    // Schedule prefab to load.
    universe
        .resources
        .get_mut::<true, AssetDatabase>()?
        .schedule("prefab://prefab.json")?;

    // Run game with graph scheduler.
    let mut scheduler = GraphScheduler::<true>::default();
    loop {
        scheduler.run(&mut universe)?;

        // Pause before next frame to ask player what to do.
        // Normally game don't need this, but here we let player
        // see how asset state changes every frame.
        println!("Type command:");
        let mut input = String::new();
        stdin().read_line(&mut input)?;
        input = input.trim().to_owned();
        if input == "exit" {
            println!("Exiting game");
            break;
        }
    }

    Ok(())
}

fn asset_database_maintain(context: SystemContext) -> Result<(), Box<dyn Error>> {
    let mut assets = context.fetch::<Res<true, &mut AssetDatabase>>()?;

    if assets.is_busy() {
        println!("Asset database is processing assets!");
    }

    // This system is used to maintain asset database internal processes.
    // No real asset progress is being made without it.
    assets.maintain()?;

    Ok(())
}

fn override_world_from_prefab_asset(context: SystemContext) -> Result<(), Box<dyn Error>> {
    let (mut commands, assets, processor, serialization, registry) = context.fetch::<(
        Res<true, &mut CommandBuffer>,
        Res<true, &AssetDatabase>,
        Res<true, &WorldProcessor>,
        Res<true, &SerializationRegistry>,
        Res<true, &Registry>,
    )>()?;

    // Search for added prefab asset entities.
    for entity in assets.storage.added().iter_of::<Prefab>() {
        // Grab prefab data from asset
        let prefab = AssetHandle::new(entity).access::<&Prefab>(&assets);
        // Build world out of it.
        let world = prefab
            .to_world::<true>(&processor, &serialization, &registry, ())?
            .0;
        // Schedule to override existing one with it.
        commands.schedule(|simulation| {
            println!("Override existing world from prefab!");
            *simulation = world;
        });
    }

    Ok(())
}

fn render_images(context: SystemContext) -> Result<(), Box<dyn Error>> {
    let (world, mut assets, query) = context.fetch::<(
        &World,
        Res<true, &mut AssetDatabase>,
        Query<true, &ImageRenderable>,
    )>()?;

    for image in query.query(world) {
        // Try to render entities with renderable component using asset content
        // behind asset reference if asset is loaded.
        // Asset references resolve only once by their path and cache asset handle.
        if let Ok(asset) = image.reference.resolve(&assets) {
            if let Some(content) = asset.access_checked::<&String>() {
                println!("{}", content);
            }
        } else {
            // If image asset is not present in database, schedule it for loading.
            image.reference.path().schedule(&mut assets)?;
        }
    }

    Ok(())
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ImageRenderable {
    pub reference: AssetRef,
}

impl ImageRenderable {
    pub fn new(path: impl Into<AssetPathStatic>) -> Self {
        Self {
            reference: AssetRef::new(path),
        }
    }
}

struct PrefabAssetProcessor;

impl BundleWithDependenciesProcessor for PrefabAssetProcessor {
    type Bundle = (Prefab,);

    fn process_bytes(
        &mut self,
        bytes: Vec<u8>,
    ) -> Result<BundleWithDependencies<Self::Bundle>, Box<dyn Error>> {
        println!("Processing prefab");
        let prefab = serde_json::from_slice::<Prefab>(&bytes)?;
        Ok(BundleWithDependencies::new((prefab,)))
    }
}
