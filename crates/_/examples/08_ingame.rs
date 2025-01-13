use anput::prelude::*;
use keket::{
    database::{handle::AssetHandle, path::AssetPath, AssetDatabase},
    fetch::file::FileAssetFetch,
    protocol::bundle::BundleAssetProtocol,
};
use std::{error::Error, io::stdin};

fn main() -> Result<(), Box<dyn Error>> {
    struct GamePlugin;
    let game = GraphSchedulerQuickPlugin::<true, GamePlugin>::default()
        .resource(
            AssetDatabase::default()
                .with_protocol(BundleAssetProtocol::new("image", TextImageAsset::decode))
                .with_fetch(FileAssetFetch::default().with_root("./resources/")),
        )
        .system(asset_database_maintain, "asset_database_maintain", ())
        .system(render_images, "render_images", ())
        .commit();

    let mut universe = Universe::default()
        .with_basics(10240, 10240)
        .with_plugin(game);

    universe.simulation.spawn(ImageRenderable::bundle(
        "image://cat.txt",
        &mut *universe.resources.get_mut::<true, AssetDatabase>()?,
    )?)?;

    universe.simulation.spawn(ImageRenderable::bundle(
        "image://logo.txt",
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
        let content = &image.0.access::<&TextImageAsset>(&assets).0;
        println!("{}", content)
    }

    Ok(())
}

pub struct ImageRenderable(pub AssetHandle);

impl ImageRenderable {
    pub fn bundle(
        path: impl Into<AssetPath<'static>>,
        database: &mut AssetDatabase,
    ) -> Result<(Self,), Box<dyn Error>> {
        Ok((Self(database.ensure(path)?),))
    }
}

pub struct TextImageAsset(pub String);

impl TextImageAsset {
    fn decode(bytes: Vec<u8>) -> Result<(Self,), Box<dyn Error>> {
        println!("Decoding text image");
        let text = std::str::from_utf8(&bytes)?.to_owned();
        Ok((Self(text),))
    }
}
