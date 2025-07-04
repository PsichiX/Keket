use crate::{
    database::path::{AssetPath, AssetPathStatic},
    store::{AssetAwaitsAsyncStore, AssetStore},
};
use anput::{bundle::DynamicBundle, world::World};
use std::{
    collections::HashMap,
    error::Error,
    pin::Pin,
    sync::RwLock,
    task::{Context, Poll, Waker},
};

type AssetStoreFuture = Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + Sync>>;

pub struct FutureAssetStore {
    future_spawner: Box<dyn Fn(AssetPathStatic, Vec<u8>) -> AssetStoreFuture + Send + Sync>,
    futures: RwLock<HashMap<AssetPathStatic, Option<AssetStoreFuture>>>,
}

impl FutureAssetStore {
    pub fn new<Fut>(
        future_spawner: impl Fn(AssetPathStatic, Vec<u8>) -> Fut + Send + Sync + 'static,
    ) -> Self
    where
        Fut: Future<Output = Result<(), Box<dyn Error>>> + Send + Sync + 'static,
    {
        Self {
            future_spawner: Box::new(move |path, bytes| Box::pin(future_spawner(path, bytes))),
            futures: Default::default(),
        }
    }
}

impl AssetStore for FutureAssetStore {
    fn save_bytes(&self, path: AssetPath, bytes: Vec<u8>) -> Result<DynamicBundle, Box<dyn Error>> {
        let path: AssetPathStatic = path.into_static();
        self.futures
            .write()
            .map_err(|error| format!("{}", error))?
            .insert(path.clone(), Some((self.future_spawner)(path, bytes)));
        let mut bundle = DynamicBundle::default();
        let _ = bundle.add_component(AssetAwaitsAsyncStore);
        Ok(bundle)
    }

    fn maintain(&mut self, storage: &mut World) -> Result<(), Box<dyn Error>> {
        let mut cx = Context::from_waker(Waker::noop());
        let mut futures = self.futures.write().map_err(|error| format!("{}", error))?;
        for (path, future) in futures.iter_mut() {
            if let Some(mut f) = future.take() {
                match f.as_mut().poll(&mut cx) {
                    Poll::Ready(Ok(_)) => {
                        if let Some(entity) = storage.find_by::<true, _>(path) {
                            storage.remove::<(AssetAwaitsAsyncStore,)>(entity)?;
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
        futures.retain(|_, v| v.is_some());
        Ok(())
    }
}
