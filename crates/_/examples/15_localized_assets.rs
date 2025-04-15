use keket::{
    database::{AssetDatabase, path::AssetPath},
    fetch::{file::FileAssetFetch, rewrite::RewriteAssetFetch},
    protocol::text::TextAssetProtocol,
};
use std::{
    error::Error,
    sync::{Arc, RwLock},
};

fn main() -> Result<(), Box<dyn Error>> {
    /* ANCHOR: main */
    let language = Arc::new(RwLock::new("en"));
    let language2 = language.clone();

    let mut database = AssetDatabase::default()
        .with_protocol(TextAssetProtocol)
        .with_fetch(RewriteAssetFetch::new(
            FileAssetFetch::default().with_root("resources"),
            move |path| {
                // Rewrite input path to localized one.
                Ok(AssetPath::from_parts(
                    path.protocol(),
                    &format!(
                        "{}.{}{}",
                        path.path_without_extension(),
                        *language2.read().unwrap(),
                        path.path_dot_extension().unwrap_or_default()
                    ),
                    path.meta(),
                ))
            },
        ));

    // Gets `text://localized.en.txt`.
    let asset = database.ensure("text://localized.txt")?;
    println!("English: {}", asset.access::<&String>(&database));

    // Change language.
    *language.write().unwrap() = "de";
    database.storage.clear();

    // Gets `text://localized.de.txt`.
    let asset = database.ensure("text://localized.txt")?;
    println!("German: {}", asset.access::<&String>(&database));
    /* ANCHOR_END: main */

    Ok(())
}
