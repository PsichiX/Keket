use crate::{
    database::path::{AssetPath, AssetPathStatic},
    fetch::{AssetAwaitsAsyncFetch, AssetFetch},
};
use anput::{
    bundle::DynamicBundle,
    third_party::time::{Duration, Instant},
    world::World,
};
use std::{collections::BTreeSet, error::Error, sync::RwLock};

/// Strategy for throttling asset fetches during maintenance ticks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThrottledAssetFetchStrategy {
    /// Limits fetches to specified maximum number per maintenance tick.
    Number(usize),
    /// Limits fetches to specified maximum duration per maintenance tick.
    Duration(Duration),
}

pub struct ThrottledAssetFetch<Fetch: AssetFetch> {
    fetch: RwLock<Fetch>,
    strategy: ThrottledAssetFetchStrategy,
    awaiting: RwLock<BTreeSet<AssetPathStatic>>,
}

impl<Fetch: AssetFetch> ThrottledAssetFetch<Fetch> {
    pub fn new(fetch: Fetch, strategy: ThrottledAssetFetchStrategy) -> Self {
        Self {
            fetch: RwLock::new(fetch),
            strategy,
            awaiting: Default::default(),
        }
    }
}

impl<Fetch: AssetFetch> AssetFetch for ThrottledAssetFetch<Fetch> {
    fn load_bytes(&self, path: AssetPath) -> Result<DynamicBundle, Box<dyn Error>> {
        let path: AssetPathStatic = path.into_static();
        self.awaiting
            .write()
            .map_err(|error| {
                format!(
                    "Failed to get write access to inner fetch engine in throttled fetch for asset: `{path}`. Error: {error}"
                )
            })?
            .insert(path);
        let mut bundle = DynamicBundle::default();
        let _ = bundle.add_component(AssetAwaitsAsyncFetch);
        Ok(bundle)
    }

    fn maintain(&mut self, storage: &mut World) -> Result<(), Box<dyn Error>> {
        self.fetch
            .write()
            .map_err(|error| format!("Failed throttled fetch engine maintainance. Error: {error}"))?
            .maintain(storage)?;

        let mut number = 0;
        // TODO: make it work for web wasm!
        let timer = Instant::now();
        while let Some(path) = self.awaiting.write().map_err(|error| {
            format!(
                "Failed to get write access to awaiting fetches during throttled fetch maintainance. Error: {error}")
            }
        )?.pop_last() {
            let bundle = self.fetch
                .write()
                .map_err(|error| {
                    format!(
                        "Failed to get write access to inner fetch engine in throttled fetch for asset: `{path}`. Error: {error}"
                    )
                })?
                .load_bytes(path.clone());
            match bundle {
                Ok(bundle) => {
                    if let Some(entity) = storage.find_by::<true, _>(&path) {
                        storage.remove::<(AssetAwaitsAsyncFetch,)>(entity)?;
                        storage.insert(entity, bundle)?;
                    }
                }
                Err(e) => {
                    if let Some(entity) = storage.find_by::<true, _>(&path) {
                        storage.remove::<(AssetAwaitsAsyncFetch,)>(entity)?;
                    }
                    return Err(format!(
                        "Throttled fetch execution of `{path}` asset failed with error: {e}"
                    ).into());
                }
            }
            number += 1;
            match self.strategy {
                ThrottledAssetFetchStrategy::Number(max_per_tick) => {
                    if number >= max_per_tick {
                        break;
                    }
                }
                ThrottledAssetFetchStrategy::Duration(max_duration) => {
                    if timer.elapsed() >= max_duration {
                        break;
                    }
                }
            }
        }

        Ok(())
    }
}
