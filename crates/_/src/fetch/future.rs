use crate::{
    database::path::{AssetPath, AssetPathStatic},
    fetch::{AssetFetch, deferred::AssetAwaitsDeferredJob},
};
use anput::{bundle::DynamicBundle, world::World};
use std::{
    collections::HashMap,
    error::Error,
    pin::Pin,
    sync::RwLock,
    task::{Context, Poll, Waker},
};

type AssetFetchFuture =
    Pin<Box<dyn Future<Output = Result<DynamicBundle, Box<dyn Error>>> + Send + Sync>>;

/// A future-based asset fetcher that allows fetching asset bytes asynchronously.
/// It uses an user-defined future spawner to create futures for loading asset
/// bytes and manages their completion in a non-blocking manner.
///
/// The main reason for this fetcher to exist is to allow the use of async/await
/// syntax for asset loading, and especially to allow the use of third party
/// async/await libraries such as `tokio` or `async-std` for asset loading.
pub struct FutureAssetFetch {
    future_spawner: Box<dyn Fn(AssetPathStatic) -> AssetFetchFuture + Send + Sync>,
    futures: RwLock<HashMap<AssetPathStatic, Option<AssetFetchFuture>>>,
}

impl FutureAssetFetch {
    /// Creates a new `FutureAssetFetch` with a specified future spawner function.
    ///
    /// # Arguments
    /// - `future_spawner`: A function that takes an `AssetPathStatic` and returns
    ///   a future that resolves to a `DynamicBundle` or an error.
    ///
    /// # Returns
    /// - A new `FutureAssetFetch` instance.
    pub fn new<Fut>(future_spawner: impl Fn(AssetPathStatic) -> Fut + Send + Sync + 'static) -> Self
    where
        Fut: Future<Output = Result<DynamicBundle, Box<dyn Error>>> + Send + Sync + 'static,
    {
        Self {
            future_spawner: Box::new(move |path| Box::pin(future_spawner(path))),
            futures: Default::default(),
        }
    }
}

impl AssetFetch for FutureAssetFetch {
    fn load_bytes(&self, path: AssetPath) -> Result<DynamicBundle, Box<dyn Error>> {
        let path: AssetPathStatic = path.into_static();
        self.futures
            .write()
            .map_err(|error| format!("{}", error))?
            .insert(path.clone(), Some((self.future_spawner)(path)));
        let mut bundle = DynamicBundle::default();
        let _ = bundle.add_component(AssetAwaitsDeferredJob);
        Ok(bundle)
    }

    fn maintain(&mut self, storage: &mut World) -> Result<(), Box<dyn Error>> {
        let mut cx = Context::from_waker(Waker::noop());
        for (path, future) in self
            .futures
            .write()
            .map_err(|error| format!("{}", error))?
            .iter_mut()
        {
            if let Some(mut f) = future.take() {
                match f.as_mut().poll(&mut cx) {
                    Poll::Ready(Ok(result)) => {
                        if let Some(entity) = storage.find_by::<true, _>(path) {
                            storage.remove::<(AssetAwaitsDeferredJob,)>(entity)?;
                            storage.insert(entity, result)?;
                        }
                    }
                    Poll::Ready(Err(e)) => {
                        return Err(e);
                    }
                    Poll::Pending => {
                        *future = Some(f);
                    }
                }
            }
        }
        Ok(())
    }
}
