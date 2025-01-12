use crate::{
    database::{path::AssetPath, reference::AssetRef},
    fetch::AssetFetch,
};
use anput::world::World;
use std::{borrow::Cow, error::Error};

#[derive(Default)]
pub struct RouterPattern {
    path_prefix: Cow<'static, str>,
    meta_entry_patterns: Vec<RouterEntryPattern>,
}

impl RouterPattern {
    pub fn new(path_prefix: impl Into<Cow<'static, str>>) -> Self {
        Self {
            path_prefix: path_prefix.into(),
            meta_entry_patterns: vec![],
        }
    }

    pub fn entry(mut self, pattern: RouterEntryPattern) -> Self {
        self.meta_entry_patterns.push(pattern);
        self
    }

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

pub struct RouterEntryPattern {
    key: Option<Cow<'static, str>>,
    value: Option<Cow<'static, str>>,
}

impl RouterEntryPattern {
    pub fn key(key: impl Into<Cow<'static, str>>) -> Self {
        Self {
            key: Some(key.into()),
            value: None,
        }
    }

    pub fn value(value: impl Into<Cow<'static, str>>) -> Self {
        Self {
            key: None,
            value: Some(value.into()),
        }
    }

    pub fn key_value(
        key: impl Into<Cow<'static, str>>,
        value: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self {
            key: Some(key.into()),
            value: Some(value.into()),
        }
    }

    fn validate(&self, path: &AssetPath) -> bool {
        path.meta_items().any(|(key, value)| {
            self.key.as_deref().map(|k| k == key).unwrap_or(true)
                && self.value.as_deref().map(|v| v == value).unwrap_or(true)
        })
    }
}

#[derive(Default)]
pub struct RouterAssetFetch {
    table: Vec<(RouterPattern, Box<dyn AssetFetch>)>,
}

impl RouterAssetFetch {
    pub fn route(mut self, pattern: RouterPattern, fetch: impl AssetFetch + 'static) -> Self {
        self.add(pattern, fetch);
        self
    }

    pub fn add(&mut self, pattern: RouterPattern, fetch: impl AssetFetch + 'static) {
        self.table.push((pattern, Box::new(fetch)));
    }
}

impl AssetFetch for RouterAssetFetch {
    fn load_bytes(
        &mut self,
        reference: AssetRef,
        path: AssetPath,
        storage: &mut World,
    ) -> Result<(), Box<dyn Error>> {
        for (pattern, fetch) in self.table.iter_mut().rev() {
            if let Some(path) = pattern.validate(&path) {
                return fetch.load_bytes(reference, path, storage);
            }
        }
        Err(format!("Could not find route for asset: `{}`", path).into())
    }

    fn maintain(&mut self, storage: &mut World) -> Result<(), Box<dyn Error>> {
        for (_, fetch) in &mut self.table {
            fetch.maintain(storage)?;
        }
        Ok(())
    }
}
