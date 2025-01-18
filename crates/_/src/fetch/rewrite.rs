use crate::{database::path::AssetPath, fetch::AssetFetch};
use anput::{bundle::DynamicBundle, world::World};
use std::error::Error;

/// A wrapper for an [`AssetFetch`] implementation that rewrites the asset path
/// before delegating to the inner fetcher.
///
/// This struct is useful for dynamically rewriting asset paths at runtime,
/// allowing redirection or transformation of the requested asset paths
/// without modifying the underlying fetcher implementation.
pub struct RewriteAssetFetch<Fetch: AssetFetch> {
    fetch: Fetch,
    #[allow(clippy::type_complexity)]
    callback: Box<dyn Fn(AssetPath) -> Result<AssetPath, Box<dyn Error>> + Send + Sync>,
}

impl<Fetch: AssetFetch> RewriteAssetFetch<Fetch> {
    /// Creates a new instance of [`RewriteAssetFetch`].
    ///
    /// # Arguments
    /// - `fetch`: The inner fetcher that handles asset fetching.
    /// - `callback`: A closure that rewrites the provided asset path. The closure
    ///    must accept an [`AssetPath`] and return a `Result` containing either
    ///    the rewritten path or an error.
    ///
    /// # Returns
    /// A new [`RewriteAssetFetch`] instance.
    pub fn new(
        fetch: Fetch,
        callback: impl Fn(AssetPath) -> Result<AssetPath, Box<dyn Error>> + Send + Sync + 'static,
    ) -> Self {
        Self {
            fetch,
            callback: Box::new(callback),
        }
    }
}

impl<Fetch: AssetFetch> AssetFetch for RewriteAssetFetch<Fetch> {
    fn load_bytes(&self, path: AssetPath) -> Result<DynamicBundle, Box<dyn Error>> {
        let path = (self.callback)(path)?;
        self.fetch.load_bytes(path)
    }

    fn maintain(&mut self, storage: &mut World) -> Result<(), Box<dyn Error>> {
        self.fetch.maintain(storage)
    }
}
