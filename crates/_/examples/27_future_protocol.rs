use anput::bundle::DynamicBundle;
use keket::{
    database::{AssetDatabase, handle::AssetHandle, path::AssetPathStatic},
    fetch::file::FileAssetFetch,
    protocol::future::{FutureAssetProtocol, FutureStorageAccess},
};
use moirai::coroutine::yield_now;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    /* ANCHOR: main */
    let mut database = AssetDatabase::default()
        .with_protocol(FutureAssetProtocol::new("lines").process(process_lines))
        .with_fetch(FileAssetFetch::default().with_root("resources"));

    let lines = database.schedule("lines://lorem.txt")?;

    while database.is_busy() {
        database.maintain()?;
    }

    println!(
        "Lines count: {}",
        lines.access::<&Vec<String>>(&database).len()
    );
    /* ANCHOR_END: main */

    Ok(())
}

/* ANCHOR: async_asset_protocol */
async fn process_lines(
    handle: AssetHandle,
    access: FutureStorageAccess,
    bytes: Vec<u8>,
) -> Result<DynamicBundle, Box<dyn Error>> {
    let access = access.access()?;
    let path = access.read().unwrap();
    let path = path.component::<true, AssetPathStatic>(handle.entity())?;
    println!("Processing {} asset lines asynchronously...", path.path());
    let content = String::from_utf8(bytes)?;
    let mut items = Vec::default();
    for line in content.lines() {
        println!("Processing line: {}", line);
        items.push(line.to_owned());
        yield_now().await;
    }
    println!("Lines processing done");
    Ok(DynamicBundle::new(items).ok().unwrap())
}
/* ANCHOR_END: async_asset_protocol */
