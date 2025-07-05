use crate::{database::path::AssetPath, fetch::AssetFetch};
use anput::{bundle::DynamicBundle, world::World};
use std::{error::Error, sync::RwLock};

/// A router-based asset fetcher that allows routing assets to different fetchers based on patterns.
#[derive(Default)]
pub struct RouterAssetFetch {
    #[allow(clippy::type_complexity)]
    table: RwLock<
        Vec<(
            // Route rule validator.
            Box<dyn Fn(&AssetPath) -> bool + Send + Sync>,
            // Asset fetcher.
            Box<dyn AssetFetch>,
            // Priority.
            usize,
        )>,
    >,
}

impl RouterAssetFetch {
    /// Adds a route rule with priority and asset fetcher.
    ///
    /// # Arguments
    /// - `rule`: The route validator.
    /// - `fetch`: The asset fetcher to handle matched paths.
    /// - `priority`: The priority of this rule.
    ///
    /// # Returns
    /// - The `RouterAssetFetch` instance with the new routing entry.
    pub fn route(
        mut self,
        rule: impl Fn(&AssetPath) -> bool + Send + Sync + 'static,
        fetch: impl AssetFetch + 'static,
        priority: usize,
    ) -> Self {
        self.add(rule, fetch, priority);
        self
    }

    /// Adds a route rule with priority and asset fetcher.
    ///
    /// # Arguments
    /// - `rule`: The route validator.
    /// - `fetch`: The asset fetcher to handle matched paths.
    /// - `priority`: The priority of this rule.
    pub fn add(
        &mut self,
        rule: impl Fn(&AssetPath) -> bool + Send + Sync + 'static,
        fetch: impl AssetFetch + 'static,
        priority: usize,
    ) {
        if let Ok(mut table) = self.table.write() {
            table.push((Box::new(rule), Box::new(fetch), priority));
            table.sort_by(|(_, _, a), (_, _, b)| a.cmp(b).reverse());
        }
    }
}

impl AssetFetch for RouterAssetFetch {
    fn load_bytes(&self, path: AssetPath) -> Result<DynamicBundle, Box<dyn Error>> {
        for (rule, fetch, _) in self
            .table
            .read()
            .map_err(|error| format!("{error}"))?
            .iter()
        {
            if rule(&path) {
                return fetch.load_bytes(path);
            }
        }
        Err(format!("Could not find route for asset: `{path}`").into())
    }

    fn maintain(&mut self, storage: &mut World) -> Result<(), Box<dyn Error>> {
        for (_, fetch, _) in self
            .table
            .write()
            .map_err(|error| format!("{error}"))?
            .iter_mut()
        {
            fetch.maintain(storage)?;
        }
        Ok(())
    }
}
