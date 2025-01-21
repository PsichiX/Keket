use keket::{
    database::AssetDatabase,
    fetch::{
        extract::{from_asset_extractor, ExtractAssetFetch},
        file::FileAssetFetch,
    },
    protocol::{bytes::BytesAssetProtocol, text::TextAssetProtocol},
};
use std::{error::Error, io::Cursor, io::Read};
use zip::ZipArchive;

fn main() -> Result<(), Box<dyn Error>> {
    /* ANCHOR: main */
    let mut database = AssetDatabase::default()
        .with_protocol(TextAssetProtocol)
        .with_protocol(BytesAssetProtocol)
        // We start with regular fetch engine.
        .with_fetch(FileAssetFetch::default().with_root("resources"));

    // Start loading package ZIP bytes/
    database.ensure("bytes://package.zip")?;

    // Maintain database while busy.
    while database.is_busy() {
        database.maintain()?;
    }

    // Then we push extraction asset fetch to fetch engine stack. From now on
    // any future asset request will be extracted from loaded ZIP archive.
    database.push_fetch(ExtractAssetFetch::new(from_asset_extractor(
        "bytes://package.zip",
        |bytes: &Vec<u8>, path| {
            let mut archive = ZipArchive::new(Cursor::new(bytes))?;
            let mut file = archive.by_name(path.path())?;
            let mut result = vec![];
            file.read_to_end(&mut result)?;
            Ok(result)
        },
    )));

    // Extract some assets from ZIP asset.
    let lorem = database.ensure("text://lorem.txt")?;
    let trash = database.ensure("bytes://trash.bin")?;

    // Run maintenance to process extracted asset bytes.
    database.maintain()?;

    println!("Lorem Ipsum: {}", lorem.access::<&String>(&database));
    println!("Bytes: {:?}", trash.access::<&Vec<u8>>(&database));
    /* ANCHOR_END: main */

    Ok(())
}
