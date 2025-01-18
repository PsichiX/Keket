use crate::{database::path::AssetPath, fetch::AssetFetch};
use anput::{bundle::DynamicBundle, world::World};
use std::{borrow::Cow, error::Error, sync::RwLock};

/// Defines a routing pattern for asset fetching with path matching rules.
///
/// The `RouterPattern` object allows for specifying a prefix path and associated entry patterns
/// which are used to match and route requests to the appropriate asset fetchers.
#[derive(Default)]
pub struct RouterPattern {
    path_prefix: Cow<'static, str>,
    meta_entry_patterns: Vec<RouterEntryPattern>,
    priority: usize,
}

impl RouterPattern {
    /// Creates a new routing pattern with a given path prefix.
    ///
    /// # Arguments
    /// - `path_prefix`: The prefix that will match the start of the asset paths.
    ///
    /// # Returns
    /// - A new `RouterPattern` initialized with the specified path prefix.
    pub fn new(path_prefix: impl Into<Cow<'static, str>>) -> Self {
        Self {
            path_prefix: path_prefix.into(),
            meta_entry_patterns: vec![],
            priority: 0,
        }
    }

    /// Adds a new entry pattern to the routing pattern for matching assets.
    ///
    /// # Arguments
    /// - `pattern`: The `RouterEntryPattern` to add.
    ///
    /// # Returns
    /// - The updated `RouterPattern`.
    pub fn entry(mut self, pattern: RouterEntryPattern) -> Self {
        self.meta_entry_patterns.push(pattern);
        self
    }

    /// Sets priority of this pattern.
    ///
    /// # Arguments
    /// - `priotity`: Higher priority means this rule will be tested first.
    ///
    /// # Returns
    /// - The updated `RouterPattern`.
    pub fn priority(mut self, value: usize) -> Self {
        self.priority = value;
        self
    }

    /// Validates the routing pattern against the given asset path.
    ///
    /// # Arguments
    /// - `path`: The asset path to check against the routing pattern.
    ///
    /// # Returns
    /// - An `Option<AssetPath>`: A validated `AssetPath` if the asset matches the pattern, otherwise `None`.
    fn validate(&self, path: &AssetPath) -> Option<AssetPath> {
        if path.path().starts_with(self.path_prefix.as_ref())
            && self
                .meta_entry_patterns
                .iter()
                .all(|pattern| pattern.validate(path))
        {
            Some(AssetPath::from_parts(
                path.protocol(),
                path.path()
                    .strip_prefix(self.path_prefix.as_ref())
                    .unwrap_or(path.path()),
                path.meta(),
            ))
        } else {
            None
        }
    }
}

/// Represents a pattern for validating individual entries in an asset's metadata.
pub struct RouterEntryPattern {
    key: Option<Cow<'static, str>>,
    value: Option<Cow<'static, str>>,
}

impl RouterEntryPattern {
    /// Creates a pattern matching a given metadata key.
    ///
    /// # Arguments
    /// - `key`: The metadata key to match.
    ///
    /// # Returns
    /// - A new `RouterEntryPattern` for matching the specified key.
    pub fn key(key: impl Into<Cow<'static, str>>) -> Self {
        Self {
            key: Some(key.into()),
            value: None,
        }
    }

    /// Creates a pattern matching a given metadata value.
    ///
    /// # Arguments
    /// - `value`: The metadata value to match.
    ///
    /// # Returns
    /// - A new `RouterEntryPattern` for matching the specified value.
    pub fn value(value: impl Into<Cow<'static, str>>) -> Self {
        Self {
            key: None,
            value: Some(value.into()),
        }
    }

    /// Creates a pattern that matches a specific key-value pair in the asset's metadata.
    ///
    /// # Arguments
    /// - `key`: The metadata key.
    /// - `value`: The metadata value.
    ///
    /// # Returns
    /// - A new `RouterEntryPattern` for the specified key-value pair.
    pub fn key_value(
        key: impl Into<Cow<'static, str>>,
        value: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self {
            key: Some(key.into()),
            value: Some(value.into()),
        }
    }

    /// Validates the router entry pattern against the metadata of an asset path.
    ///
    /// # Arguments
    /// - `path`: The asset path whose metadata will be validated.
    ///
    /// # Returns
    /// - `true` if the metadata of the asset path matches the pattern.
    /// - `false` if the asset path does not match.
    fn validate(&self, path: &AssetPath) -> bool {
        path.meta_items().any(|(key, value)| {
            self.key.as_deref().map(|k| k == key).unwrap_or(true)
                && self.value.as_deref().map(|v| v == value).unwrap_or(true)
        })
    }
}

/// A router-based asset fetcher that allows routing assets to different fetchers based on patterns.
#[derive(Default)]
pub struct RouterAssetFetch {
    table: RwLock<Vec<(RouterPattern, Box<dyn AssetFetch>)>>,
}

impl RouterAssetFetch {
    /// Adds a routing pattern and its associated asset fetcher to the router.
    ///
    /// # Arguments
    /// - `pattern`: The routing pattern to match asset paths.
    /// - `fetch`: The asset fetcher to handle matched paths.
    ///
    /// # Returns
    /// - The `RouterAssetFetch` instance with the new routing entry.
    pub fn route(mut self, pattern: RouterPattern, fetch: impl AssetFetch + 'static) -> Self {
        self.add(pattern, fetch);
        self
    }

    /// Adds a routing pattern and asset fetcher to the internal table of the router.
    ///
    /// # Arguments
    /// - `pattern`: The pattern used to match asset paths.
    /// - `fetch`: The fetcher to route matched assets to.
    pub fn add(&mut self, pattern: RouterPattern, fetch: impl AssetFetch + 'static) {
        if let Ok(mut table) = self.table.write() {
            table.push((pattern, Box::new(fetch)));
            table.sort_by(|(a, _), (b, _)| a.priority.cmp(&b.priority).reverse());
        }
    }
}

impl AssetFetch for RouterAssetFetch {
    fn load_bytes(&self, path: AssetPath) -> Result<DynamicBundle, Box<dyn Error>> {
        for (pattern, fetch) in self
            .table
            .write()
            .map_err(|error| format!("{}", error))?
            .iter_mut()
        {
            if let Some(path) = pattern.validate(&path) {
                return fetch.load_bytes(path);
            }
        }
        Err(format!("Could not find route for asset: `{}`", path).into())
    }

    fn maintain(&mut self, storage: &mut World) -> Result<(), Box<dyn Error>> {
        for (_, fetch) in self
            .table
            .write()
            .map_err(|error| format!("{}", error))?
            .iter_mut()
        {
            fetch.maintain(storage)?;
        }
        Ok(())
    }
}
