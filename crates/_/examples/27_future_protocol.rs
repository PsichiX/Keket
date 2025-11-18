use anput::bundle::DynamicBundle;
use keket::{
    database::AssetDatabase, fetch::file::FileAssetFetch, protocol::future::FutureAssetProtocol,
};
use moirai::coroutine::yield_now;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    /* ANCHOR: main */
    let mut database = AssetDatabase::default()
        .with_protocol(
            FutureAssetProtocol::new("lines").process(|_, _, bytes| process_lines(bytes)),
        )
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
async fn process_lines(bytes: Vec<u8>) -> Result<DynamicBundle, Box<dyn Error>> {
    println!("Processing lines asynchronously...");
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
