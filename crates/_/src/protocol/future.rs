use crate::{
    database::{handle::AssetHandle, inspector::AssetInspector},
    protocol::AssetProtocol,
    store::AssetBytesAreReadyToStore,
};
use anput::{
    bundle::DynamicBundle, third_party::intuicio_data::managed::ManagedLazy, world::World,
};
use std::{
    collections::HashMap,
    error::Error,
    pin::Pin,
    sync::{Arc, RwLock},
    task::{Context, Poll, Waker},
};

pub struct AssetAwaitsAsyncProcessing;
pub struct AssetAwaitsAsyncProducing;

type AssetProtocolProcessFuture =
    Pin<Box<dyn Future<Output = Result<DynamicBundle, Box<dyn Error>>> + Send + Sync>>;
type AssetProtocolProduceFuture =
    Pin<Box<dyn Future<Output = Result<Vec<u8>, Box<dyn Error>>> + Send + Sync>>;

#[derive(Clone)]
pub struct FutureStorageAccess(Arc<RwLock<ManagedLazy<World>>>);

impl FutureStorageAccess {
    pub fn access(&self) -> Result<ManagedLazy<World>, Box<dyn Error>> {
        Ok(self.0.read().map_err(|error| format!("{error}"))?.clone())
    }
}

pub struct FutureAssetProtocol {
    name: String,
    #[allow(clippy::type_complexity)]
    process_future_spawner: Option<
        Box<
            dyn Fn(AssetHandle, FutureStorageAccess, Vec<u8>) -> AssetProtocolProcessFuture
                + Send
                + Sync,
        >,
    >,
    #[allow(clippy::type_complexity)]
    produce_future_spawner:
        Option<Box<dyn Fn(AssetInspector) -> AssetProtocolProduceFuture + Send + Sync>>,
    process_futures:
        RwLock<HashMap<AssetHandle, Option<(AssetProtocolProcessFuture, FutureStorageAccess)>>>,
    produce_futures: RwLock<HashMap<AssetHandle, Option<AssetProtocolProduceFuture>>>,
}

impl FutureAssetProtocol {
    pub fn new(name: impl ToString) -> Self {
        Self {
            name: name.to_string(),
            process_future_spawner: None,
            produce_future_spawner: None,
            process_futures: Default::default(),
            produce_futures: Default::default(),
        }
    }

    pub fn process<Fut>(
        mut self,
        future_spawner: impl Fn(AssetHandle, FutureStorageAccess, Vec<u8>) -> Fut
        + Send
        + Sync
        + 'static,
    ) -> Self
    where
        Fut: Future<Output = Result<DynamicBundle, Box<dyn Error>>> + Send + Sync + 'static,
    {
        self.process_future_spawner = Some(Box::new(move |handle, access, bytes| {
            Box::pin(future_spawner(handle, access, bytes))
        }));
        self
    }

    pub fn produce<Fut>(
        mut self,
        future_spawner: impl Fn(AssetInspector) -> Fut + Send + Sync + 'static,
    ) -> Self
    where
        Fut: Future<Output = Result<Vec<u8>, Box<dyn Error>>> + Send + Sync + 'static,
    {
        self.produce_future_spawner = Some(Box::new(move |inspector| {
            Box::pin(future_spawner(inspector))
        }));
        self
    }
}

impl AssetProtocol for FutureAssetProtocol {
    fn name(&self) -> &str {
        &self.name
    }

    fn process_bytes(
        &mut self,
        handle: AssetHandle,
        storage: &mut World,
        bytes: Vec<u8>,
    ) -> Result<(), Box<dyn Error>> {
        let Some(future_spawner) = self.process_future_spawner.as_ref() else {
            return Ok(());
        };
        let (lazy, _lifetime) = ManagedLazy::make(storage);
        let access = FutureStorageAccess(Arc::new(RwLock::new(lazy)));
        self.process_futures
            .write()
            .map_err(|error| format!("{error}"))?
            .insert(
                handle,
                Some(((future_spawner)(handle, access.clone(), bytes), access)),
            );
        storage.insert(handle.entity(), (AssetAwaitsAsyncProcessing,))?;
        Ok(())
    }

    fn produce_asset_bytes(
        &mut self,
        handle: AssetHandle,
        storage: &mut World,
    ) -> Result<(), Box<dyn Error>> {
        let Some(future_spawner) = self.produce_future_spawner.as_ref() else {
            return Err(format!(
                "Asset protocol `{}` does not support producing bytes.",
                self.name()
            )
            .into());
        };
        let inspector = AssetInspector::new_raw(storage, handle.entity());
        self.produce_futures
            .write()
            .map_err(|error| format!("{error}"))?
            .insert(handle, Some((future_spawner)(inspector)));
        storage.insert(handle.entity(), (AssetAwaitsAsyncProducing,))?;
        Ok(())
    }

    fn maintain(&mut self, storage: &mut World) -> Result<(), Box<dyn Error>> {
        let mut cx = Context::from_waker(Waker::noop());
        let mut futures = self
            .process_futures
            .write()
            .map_err(|error| format!("{error}"))?;
        for (handle, future) in futures.iter_mut() {
            if let Some((mut f, access)) = future.take() {
                let (lazy, _lifetime) = ManagedLazy::make(storage);
                *access.0.write().map_err(|error| format!("{error}"))? = lazy;
                match f.as_mut().poll(&mut cx) {
                    Poll::Ready(Ok(result)) => {
                        storage.remove::<(AssetAwaitsAsyncProcessing,)>(handle.entity())?;
                        storage.insert(handle.entity(), result)?;
                    }
                    Poll::Ready(Err(e)) => {
                        return Err(e);
                    }
                    Poll::Pending => {
                        *future = Some((f, access));
                    }
                }
            }
        }
        futures.retain(|_, v| v.is_some());
        let mut futures = self
            .produce_futures
            .write()
            .map_err(|error| format!("{error}"))?;
        for (handle, future) in futures.iter_mut() {
            if let Some(mut f) = future.take() {
                match f.as_mut().poll(&mut cx) {
                    Poll::Ready(Ok(result)) => {
                        storage.remove::<(AssetAwaitsAsyncProducing,)>(handle.entity())?;
                        storage.insert(handle.entity(), (AssetBytesAreReadyToStore(result),))?;
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
