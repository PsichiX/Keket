use crate::database::{handle::AssetHandle, path::AssetPathStatic};
use std::{error::Error, sync::mpsc::Sender};

/// Represents different kinds of events that can occur for an asset.
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
    /// Checks if the event represents a finished state (either success or failure).
    pub fn is_done(self) -> bool {
        matches!(self, Self::BytesProcessed) || self.failure()
    }

    /// Checks if the event is still in progress.
    pub fn in_progress(self) -> bool {
        !self.is_done()
    }

    /// Checks if the event represents a successful state.
    pub fn success(self) -> bool {
        !self.failure()
    }

    /// Checks if the event represents a failure state.
    pub fn failure(self) -> bool {
        matches!(
            self,
            Self::BytesFetchingFailed | Self::BytesProcessingFailed
        )
    }
}

/// Represents an event related to an asset, combining a handle and an event kind.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AssetEvent {
    /// The handle of the asset associated with this event.
    pub handle: AssetHandle,
    /// The kind of event that occurred.
    pub kind: AssetEventKind,
    /// The path of the asset associated with this event.
    pub path: AssetPathStatic,
}

/// A trait for listeners that handle asset events.
///
/// Implementers of this trait can respond to dispatched asset events.
pub trait AssetEventListener: Send + Sync {
    /// Called when an asset event is dispatched.
    ///
    /// # Arguments
    /// - `event`: The asset event to handle.
    ///
    /// # Returns
    /// A `Result` indicating success or an error.
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

/// A unique identifier for an asset event listener binding.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AssetEventBinding(usize);

/// A manager for asset event listener bindings.
///
/// This allows for adding, removing, and dispatching events to listeners.
#[derive(Default)]
pub struct AssetEventBindings {
    id_generator: usize,
    // [(binding, listener, dispatch once)]
    bindings: Vec<(AssetEventBinding, Box<dyn AssetEventListener>, bool)>,
}

impl AssetEventBindings {
    /// Adds a new listener and returns its binding identifier.
    ///
    /// # Arguments
    /// - `listener`: The listener to be added.
    ///
    /// # Returns
    /// A unique binding identifier for the listener.
    pub fn bind(&mut self, listener: impl AssetEventListener + 'static) -> AssetEventBinding {
        let id = AssetEventBinding(self.id_generator);
        self.id_generator = self.id_generator.overflowing_add(1).0;
        self.bindings.push((id, Box::new(listener), false));
        id
    }

    /// Adds a new listener and returns its binding identifier.
    /// The listener will be automatically removed after being dispatched once.
    ///
    /// # Arguments
    /// - `listener`: The listener to be added.
    ///
    /// # Returns
    /// A unique binding identifier for the listener.
    pub fn bind_once(&mut self, listener: impl AssetEventListener + 'static) -> AssetEventBinding {
        let id = AssetEventBinding(self.id_generator);
        self.id_generator = self.id_generator.overflowing_add(1).0;
        self.bindings.push((id, Box::new(listener), true));
        id
    }

    /// Removes a listener by its binding identifier.
    ///
    /// # Arguments
    /// - `binding`: The identifier of the listener to remove.
    ///
    /// # Returns
    /// The removed listener, if found.
    pub fn unbind(&mut self, binding: AssetEventBinding) -> Option<Box<dyn AssetEventListener>> {
        self.bindings
            .iter()
            .position(|(listener_binding, _, _)| *listener_binding == binding)
            .map(|index| self.bindings.swap_remove(index).1)
    }

    /// Clears all event listener bindings.
    pub fn clear(&mut self) {
        self.bindings.clear();
    }

    /// Checks if there are no active bindings.
    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }

    /// Returns the number of active bindings.
    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    /// Returns an iterator over all binding identifiers.
    pub fn bindings(&self) -> impl Iterator<Item = AssetEventBinding> + '_ {
        self.bindings.iter().map(|(binding, _, _)| *binding)
    }

    /// Dispatches an asset event to all listeners.
    /// Listeners that were bound with `bind_once` will be removed after dispatch.
    ///
    /// # Arguments
    /// - `event`: The event to be dispatched.
    ///
    /// # Returns
    /// A `Result` indicating success or an error.
    pub fn dispatch(&mut self, event: AssetEvent) -> Result<(), Box<dyn Error>> {
        for (_, listener, _) in &mut self.bindings {
            listener.on_dispatch(event.clone())?;
        }
        self.bindings
            .retain(|(_, _, dispatch_once)| !*dispatch_once);
        Ok(())
    }
}
