use keket::{
    database::{AssetDatabase, tracker::ConsumedSingleAssetLoader},
    fetch::file::FileAssetFetch,
    protocol::text::TextAssetProtocol,
};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    /* ANCHOR: main */
    let mut database = AssetDatabase::default()
        .with_protocol(TextAssetProtocol)
        .with_fetch(FileAssetFetch::default().with_root("resources"));

    // Load single asset from database and maintain it until it's done.
    let mut loader = ConsumedSingleAssetLoader::<String>::path("text://lorem.txt");
    while loader.is_in_progress() {
        // Database must be maintained anyway.
        database.maintain()?;
        // Loader is maintained to update its internal state.
        loader.maintain(&mut database);
    }

    // Consume asset data or handle error.
    match loader {
        ConsumedSingleAssetLoader::Data(data) => {
            println!("Lorem Ipsum: {data}");
        }
        ConsumedSingleAssetLoader::Error(error) => {
            eprintln!("Asset loading error: {error}");
        }
        _ => {}
    }
    /* ANCHOR_END: main */

    Ok(())
}
