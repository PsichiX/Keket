use anput::prelude::*;
use keket::{
    database::{handle::AssetHandle, path::AssetPath, AssetDatabase},
    fetch::file::FileAssetFetch,
    protocol::text::TextAssetProtocol,
};
use std::{error::Error, io::stdin};

fn main() -> Result<(), Box<dyn Error>> {
    struct GamePlugin;
    let game = GraphSchedulerQuickPlugin::<true, GamePlugin>::default()
        .resource(
            AssetDatabase::default()
                .with_protocol(TextAssetProtocol)
                .with_fetch(FileAssetFetch::default().with_root("./resources/")),
        )
        .system(asset_database_maintain, "asset_database_maintain", ())
        .system(render_images, "render_images", ())
        .commit();

    let mut universe = Universe::default()
        .with_basics(10240, 10240)
        .with_plugin(game);

    universe.simulation.spawn(ImageRenderable::bundle(
        "text://cat.txt",
        &mut *universe.resources.get_mut::<true, AssetDatabase>()?,
    )?)?;

    universe.simulation.spawn(ImageRenderable::bundle(
        "text://logo.txt",
        &mut *universe.resources.get_mut::<true, AssetDatabase>()?,
    )?)?;

    let mut scheduler = GraphScheduler::<true>::default();
    loop {
        scheduler.run(&mut universe)?;

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

    assets.maintain()?;

    Ok(())
}

fn render_images(context: SystemContext) -> Result<(), Box<dyn Error>> {
    let (world, assets, query) = context.fetch::<(
        &World,
        Res<true, &AssetDatabase>,
        Query<true, &ImageRenderable>,
    )>()?;

    for image in query.query(world) {
        let content = image.0.access::<&String>(&assets);
        println!("{}", content);
    }

    Ok(())
}

pub struct ImageRenderable(pub AssetHandle);

impl ImageRenderable {
    pub fn bundle(
        path: impl Into<AssetPath<'static>>,
        database: &mut AssetDatabase,
    ) -> Result<(Self,), Box<dyn Error>> {
        let handle = path.into().resolve(database)?;
        Ok((Self(handle),))
    }
}
