use keket::{
    database::{path::AssetPath, AssetDatabase},
    fetch::file::FileAssetFetch,
    protocol::text::TextAssetProtocol,
};
use std::{error::Error, fs::Metadata, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    let mut database = AssetDatabase::default()
        .with_protocol(TextAssetProtocol)
        .with_fetch(FileAssetFetch::default().with_root("resources"));

    let lorem = database.ensure("text://lorem.txt".try_into()?)?;
    println!("{}", lorem.access::<&String>(&database));

    let ipsum = database.ensure("text://ipsum.txt".try_into()?)?;
    println!("{}", ipsum.access::<&String>(&database));

    for (asset_path, file_path, metadata) in database
        .storage
        .query::<true, (&AssetPath, &PathBuf, &Metadata)>()
    {
        println!(
            "Asset: `{}` at location: {:?} has metadata: {:#?}",
            asset_path, file_path, metadata
        );
    }

    Ok(())
}
