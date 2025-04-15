use keket::{
    database::{
        AssetDatabase,
        loading::{AssetsLoadingStatus, AssetsLoadingTracker},
    },
    fetch::{deferred::DeferredAssetFetch, file::FileAssetFetch},
    protocol::bytes::BytesAssetProtocol,
};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    /* ANCHOR: main */
    let mut database = AssetDatabase::default()
        .with_protocol(BytesAssetProtocol)
        .with_fetch(DeferredAssetFetch::new(
            FileAssetFetch::default().with_root("resources"),
        ));

    // Create tracker to track specific assets loading status.
    // We schedule them to load later at first database maintainance
    // to not load them too quickly.
    let tracker = AssetsLoadingTracker::default().with_many([
        database.schedule("bytes://dlc.zip")?,
        database.schedule("bytes://ferris.png")?,
        database.schedule("bytes://main.zip")?,
        database.schedule("bytes://package.zip")?,
    ]);

    // Prepare loading status to fill in.
    // Its structure tells level of detail for particular categories. Each
    // category can be either amount or list of assets. Here we use amount
    // for every category, because we track just numeric progress.
    let mut status = AssetsLoadingStatus::amount();

    // Track progress as long as database is busy.
    while database.is_busy() {
        database.maintain()?;

        // Report current loading status (progress).
        tracker.report(&database, &mut status);
        let progress = status.progress();

        println!(
            "Loading {}% ({}/{})",
            progress.factor() * 100.0,
            progress.ready_to_use,
            progress.total()
        );
    }
    /* ANCHOR_END: main */

    Ok(())
}
