use keket::{
    database::AssetDatabase,
    fetch::{
        file::FileAssetFetch,
        router::{RouterAssetFetch, RouterEntryPattern, RouterPattern},
    },
    protocol::{bytes::BytesAssetProtocol, text::TextAssetProtocol},
};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut database = AssetDatabase::default()
        .with_protocol(TextAssetProtocol)
        .with_protocol(BytesAssetProtocol)
        .with_fetch(
            // Router allows to combine multiple asset sources, so that proper one to use
            // for given asset is selected by pattern in asset path.
            RouterAssetFetch::default()
                // Every asset that has `router=file` meta, will load asset from file.
                .route(
                    RouterPattern::default().entry(RouterEntryPattern::key_value("router", "file")),
                    FileAssetFetch::default().with_root("resources"),
                )
                // Every asset that has `memory/` path prefix, will load from in-memory collection.
                .route(
                    RouterPattern::new("memory/").priority(1),
                    vec![(
                        "trash.bin".to_owned(),
                        std::fs::read("./resources/trash.bin")?,
                    )],
                ),
        );

    // This asset will select file router.
    let lorem = database.ensure("text://lorem.txt?router=file")?;
    println!("Lorem Ipsum: {}", lorem.access::<&String>(&database));

    // This asset will select memory router.
    let trash = database.ensure("bytes://memory/trash.bin")?;
    println!("Bytes: {:?}", trash.access::<&Vec<u8>>(&database));

    Ok(())
}
