use crate::{
    database::{path::AssetPath, reference::AssetRef},
    fetch::AssetFetch,
};
use anput::{bundle::Bundle, entity::Entity, world::World};
use std::{
    error::Error,
    marker::PhantomData,
    sync::Arc,
    thread::{spawn, JoinHandle},
};

pub struct AssetAwaitsDeferredJob;

pub trait DeferredAssetJob: Send + Sync + 'static {
    type Result: Bundle + Send + 'static;

    fn execute(&self, path: AssetPath) -> Self::Result;
}

pub struct DeferredAssetFetch<Job: DeferredAssetJob> {
    #[allow(clippy::type_complexity)]
    tasks: Vec<(Entity, JoinHandle<Job::Result>)>,
    job: Arc<Job>,
    _phantom: PhantomData<fn() -> Job>,
}

impl<Job: DeferredAssetJob> DeferredAssetFetch<Job> {
    pub fn new(job: Job) -> Self {
        Self {
            tasks: Default::default(),
            job: Arc::new(job),
            _phantom: PhantomData,
        }
    }
}

impl<Job: DeferredAssetJob> AssetFetch for DeferredAssetFetch<Job> {
    fn load_bytes(
        &mut self,
        reference: AssetRef,
        path: AssetPath,
        storage: &mut World,
    ) -> Result<(), Box<dyn Error>> {
        let path = path.into_static();
        let job = self.job.clone();
        self.tasks
            .push((reference.entity(), spawn(move || job.execute(path))));
        storage.insert(reference.entity(), (AssetAwaitsDeferredJob,))?;
        Ok(())
    }

    fn maintain(&mut self, storage: &mut World) -> Result<(), Box<dyn Error>> {
        let complete = self
            .tasks
            .iter()
            .enumerate()
            .filter(|(_, (_, join))| join.is_finished())
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        for index in complete.into_iter().rev() {
            let (entity, join) = self.tasks.swap_remove(index);
            let result = join.join().map_err(|_| {
                format!(
                    "Error during job execution of `{}` asset!",
                    storage
                        .component::<true, AssetPath>(entity)
                        .map(|path| path.content().to_owned())
                        .unwrap_or_default()
                )
            })?;
            storage.remove::<(AssetAwaitsDeferredJob,)>(entity)?;
            storage.insert(entity, result)?;
        }
        Ok(())
    }
}
