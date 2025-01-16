use crate::database::handle::AssetHandle;
use std::{error::Error, sync::mpsc::Sender};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AssetEventKind {
    AwaitsResolution,
    AwaitsDeferredJob,
    BytesReadyToProcess,
    BytesProcessed,
    Unloaded,
    BytesFetchingFailed,
    BytesProcessingFailed,
}

impl AssetEventKind {
    pub fn is_done(self) -> bool {
        matches!(self, Self::BytesProcessed) || self.failure()
    }

    pub fn in_progress(self) -> bool {
        !self.is_done()
    }

    pub fn success(self) -> bool {
        !self.failure()
    }

    pub fn failure(self) -> bool {
        matches!(
            self,
            Self::BytesFetchingFailed | Self::BytesProcessingFailed
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AssetEvent {
    pub handle: AssetHandle,
    pub kind: AssetEventKind,
}

pub trait AssetEventListener: Send + Sync {
    fn on_dispatch(&mut self, event: AssetEvent) -> Result<(), Box<dyn Error>>;
}

impl AssetEventListener for Sender<AssetEvent> {
    fn on_dispatch(&mut self, event: AssetEvent) -> Result<(), Box<dyn Error>> {
        self.send(event)?;
        Ok(())
    }
}

impl<F> AssetEventListener for F
where
    F: FnMut(AssetEvent) -> Result<(), Box<dyn Error>> + Send + Sync,
{
    fn on_dispatch(&mut self, event: AssetEvent) -> Result<(), Box<dyn Error>> {
        self(event)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AssetEventBinding(usize);

#[derive(Default)]
pub struct AssetEventBindings {
    id_generator: usize,
    bindings: Vec<(AssetEventBinding, Box<dyn AssetEventListener>)>,
}

impl AssetEventBindings {
    pub fn bind(&mut self, listener: impl AssetEventListener + 'static) -> AssetEventBinding {
        let id = AssetEventBinding(self.id_generator);
        self.id_generator = self.id_generator.overflowing_add(1).0;
        self.bindings.push((id, Box::new(listener)));
        id
    }

    pub fn unbind(&mut self, binding: AssetEventBinding) -> Option<Box<dyn AssetEventListener>> {
        self.bindings
            .iter()
            .position(|(listener_binding, _)| *listener_binding == binding)
            .map(|index| self.bindings.swap_remove(index).1)
    }

    pub fn clear(&mut self) {
        self.bindings.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }

    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    pub fn bindings(&self) -> impl Iterator<Item = AssetEventBinding> + '_ {
        self.bindings.iter().map(|(binding, _)| *binding)
    }

    pub fn dispatch(&mut self, event: AssetEvent) -> Result<(), Box<dyn Error>> {
        for (_, listener) in &mut self.bindings {
            listener.on_dispatch(event)?;
        }
        Ok(())
    }
}
