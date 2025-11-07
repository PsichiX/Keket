use crate::{
    database::{AssetDatabase, handle::AssetHandle, path::AssetPathStatic},
    fetch::{AssetAwaitsAsyncFetch, AssetAwaitsResolution, AssetBytesAreReadyToProcess},
    store::{AssetAwaitsAsyncStore, AssetAwaitsStoring, AssetBytesAreReadyToStore},
};
use anput::component::Component;
use std::{collections::HashSet, error::Error};

/// A struct to track status of assets in the database.
#[derive(Debug, Default, Clone)]
pub struct AssetsTracker {
    handles: HashSet<AssetHandle>,
}

impl AssetsTracker {
    /// Creates a modified `AssetsTracker` instance.
    ///
    /// # Arguments
    /// - `handle`: An `AssetHandle` to track.
    ///
    /// # Returns
    /// A modified `AssetsTracker` instance with the specified handle tracked.
    pub fn with(mut self, handle: AssetHandle) -> Self {
        self.track(handle);
        self
    }

    /// Creates a modified `AssetsTracker` instance with multiple handles.
    ///
    /// # Arguments
    /// - `handles`: An iterable collection of `AssetHandle` to track.
    ///
    /// # Returns
    /// A modified `AssetsTracker` instance with the specified handles tracked.
    pub fn with_many(mut self, handles: impl IntoIterator<Item = AssetHandle>) -> Self {
        self.track_many(handles);
        self
    }

    /// Track a single asset handle.
    ///
    /// # Arguments
    /// - `handle`: An `AssetHandle` to track.
    pub fn track(&mut self, handle: AssetHandle) {
        self.handles.insert(handle);
    }

    /// Track multiple asset handles.
    ///
    /// # Arguments
    /// - `handles`: An iterable collection of `AssetHandle` to track.
    pub fn track_many(&mut self, handles: impl IntoIterator<Item = AssetHandle>) {
        self.handles.extend(handles);
    }

    /// Untrack a single asset handle.
    ///
    /// # Arguments
    /// - `handle`: An `AssetHandle` to untrack.
    pub fn untrack(&mut self, handle: AssetHandle) {
        self.handles.remove(&handle);
    }

    /// Untrack multiple asset handles.
    ///
    /// # Arguments
    /// - `handles`: An iterable collection of `AssetHandle` to untrack.
    pub fn untrack_many(&mut self, handles: impl IntoIterator<Item = AssetHandle>) {
        for handle in handles {
            self.handles.remove(&handle);
        }
    }

    /// Get number of tracked handles.
    pub fn len(&self) -> usize {
        self.handles.len()
    }

    /// Check if there are no tracked handles.
    pub fn is_empty(&self) -> bool {
        self.handles.is_empty()
    }

    /// Return an iterator over the tracked handles.
    pub fn iter(&self) -> impl Iterator<Item = AssetHandle> + '_ {
        self.handles.iter().copied()
    }

    /// Reports the status of tracked assets in the database.
    ///
    /// # Arguments
    /// - `out_status`: A mutable reference to output `AssetsStatus`.
    pub fn report(&self, database: &AssetDatabase, out_status: &mut AssetsStatus) {
        out_status.clear();
        for handle in &self.handles {
            if handle.does_exists(database) {
                if handle.has::<AssetAwaitsStoring>(database) {
                    out_status.awaiting_storing.add(*handle);
                } else if handle.has::<AssetBytesAreReadyToStore>(database) {
                    out_status.with_bytes_ready_to_store.add(*handle);
                } else if handle.has::<AssetAwaitsAsyncStore>(database) {
                    out_status.awaiting_async_store.add(*handle);
                } else if handle.has::<AssetAwaitsResolution>(database) {
                    out_status.awaiting_resolution.add(*handle);
                } else if handle.has::<AssetBytesAreReadyToProcess>(database) {
                    out_status.with_bytes_ready_to_process.add(*handle);
                } else if handle.has::<AssetAwaitsAsyncFetch>(database) {
                    out_status.awaiting_async_fetch.add(*handle);
                } else {
                    out_status.ready_to_use.add(*handle);
                }
            }
        }
    }
}

/// A struct to represent status of assets category.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssetsStatusCategory {
    Amount(usize),
    List(Vec<AssetHandle>),
}

impl Default for AssetsStatusCategory {
    fn default() -> Self {
        AssetsStatusCategory::amount()
    }
}

impl AssetsStatusCategory {
    /// Creates an empty `AssetsStatusCategory` as amount variant.
    pub fn amount() -> Self {
        AssetsStatusCategory::Amount(0)
    }

    /// Creates an empty `AssetsStatusCategory` as list variant.
    pub fn list() -> Self {
        AssetsStatusCategory::List(Default::default())
    }

    /// Tells number of assets in the category.
    pub fn len(&self) -> usize {
        match self {
            AssetsStatusCategory::Amount(len) => *len,
            AssetsStatusCategory::List(list) => list.len(),
        }
    }

    /// Tells if the category is empty.
    pub fn is_empty(&self) -> bool {
        match self {
            AssetsStatusCategory::Amount(len) => *len == 0,
            AssetsStatusCategory::List(list) => list.is_empty(),
        }
    }

    /// Clears the category.
    pub fn clear(&mut self) {
        match self {
            AssetsStatusCategory::Amount(len) => *len = 0,
            AssetsStatusCategory::List(list) => list.clear(),
        }
    }

    /// Adds an asset handle to the category.
    pub fn add(&mut self, handle: AssetHandle) {
        match self {
            AssetsStatusCategory::Amount(len) => *len += 1,
            AssetsStatusCategory::List(list) => list.push(handle),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct AssetsStatus {
    pub awaiting_storing: AssetsStatusCategory,
    pub with_bytes_ready_to_store: AssetsStatusCategory,
    pub awaiting_async_store: AssetsStatusCategory,
    pub awaiting_resolution: AssetsStatusCategory,
    pub with_bytes_ready_to_process: AssetsStatusCategory,
    pub awaiting_async_fetch: AssetsStatusCategory,
    pub ready_to_use: AssetsStatusCategory,
}

impl AssetsStatus {
    /// Creates a new `AssetsStatus` instance with all categories as amount variants.
    pub fn amount() -> Self {
        AssetsStatus {
            awaiting_storing: AssetsStatusCategory::amount(),
            with_bytes_ready_to_store: AssetsStatusCategory::amount(),
            awaiting_async_store: AssetsStatusCategory::amount(),
            awaiting_resolution: AssetsStatusCategory::amount(),
            with_bytes_ready_to_process: AssetsStatusCategory::amount(),
            awaiting_async_fetch: AssetsStatusCategory::amount(),
            ready_to_use: AssetsStatusCategory::amount(),
        }
    }

    /// Creates a new `AssetsStatus` instance with all categories as list variants.
    pub fn list() -> Self {
        AssetsStatus {
            awaiting_storing: AssetsStatusCategory::list(),
            with_bytes_ready_to_store: AssetsStatusCategory::list(),
            awaiting_async_store: AssetsStatusCategory::list(),
            awaiting_resolution: AssetsStatusCategory::list(),
            with_bytes_ready_to_process: AssetsStatusCategory::list(),
            awaiting_async_fetch: AssetsStatusCategory::list(),
            ready_to_use: AssetsStatusCategory::list(),
        }
    }

    /// Clears all categories in status.
    pub fn clear(&mut self) {
        self.awaiting_resolution.clear();
        self.with_bytes_ready_to_process.clear();
        self.awaiting_async_fetch.clear();
        self.ready_to_use.clear();
    }

    /// Returns the progress of status.
    ///
    /// # Returns
    /// An `AssetsProgress` struct representing the progress of status.
    pub fn progress(&self) -> AssetsProgress {
        AssetsProgress {
            awaiting_storing: self.awaiting_storing.len(),
            with_bytes_ready_to_store: self.with_bytes_ready_to_store.len(),
            awaiting_async_store: self.awaiting_async_store.len(),
            awaiting_resolution: self.awaiting_resolution.len(),
            with_bytes_ready_to_process: self.with_bytes_ready_to_process.len(),
            awaiting_async_fetch: self.awaiting_async_fetch.len(),
            ready_to_use: self.ready_to_use.len(),
        }
    }
}

/// A struct to represent progress of assets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AssetsProgress {
    pub awaiting_storing: usize,
    pub with_bytes_ready_to_store: usize,
    pub awaiting_async_store: usize,
    pub awaiting_resolution: usize,
    pub with_bytes_ready_to_process: usize,
    pub awaiting_async_fetch: usize,
    pub ready_to_use: usize,
}

impl AssetsProgress {
    /// Creates a new `AssetsProgress` instance with only storing-related fields.
    pub fn only_storing(&self) -> Self {
        AssetsProgress {
            awaiting_storing: self.awaiting_storing,
            with_bytes_ready_to_store: self.with_bytes_ready_to_store,
            awaiting_async_store: self.awaiting_async_store,
            awaiting_resolution: 0,
            with_bytes_ready_to_process: 0,
            awaiting_async_fetch: 0,
            ready_to_use: 0,
        }
    }

    /// Creates a new `AssetsProgress` instance with only fetching-related fields.
    pub fn only_fetching(&self) -> Self {
        AssetsProgress {
            awaiting_storing: 0,
            with_bytes_ready_to_store: 0,
            awaiting_async_store: 0,
            awaiting_resolution: self.awaiting_resolution,
            with_bytes_ready_to_process: self.with_bytes_ready_to_process,
            awaiting_async_fetch: self.awaiting_async_fetch,
            ready_to_use: self.ready_to_use,
        }
    }

    /// Returns the total number of assets in progress.
    pub fn total(&self) -> usize {
        self.awaiting_storing
            + self.with_bytes_ready_to_store
            + self.awaiting_async_store
            + self.awaiting_resolution
            + self.with_bytes_ready_to_process
            + self.awaiting_async_fetch
            + self.ready_to_use
    }

    /// Tells if progress is complete.
    pub fn is_complete(&self) -> bool {
        self.awaiting_storing == 0
            && self.with_bytes_ready_to_store == 0
            && self.awaiting_async_store == 0
            && self.awaiting_resolution == 0
            && self.with_bytes_ready_to_process == 0
            && self.awaiting_async_fetch == 0
    }

    /// Tells if progress is in progress.
    pub fn is_in_progress(&self) -> bool {
        !self.is_complete()
    }

    /// Returns the factor of progress (0-1).
    pub fn factor(&self) -> f32 {
        let total = self.total();
        if total == 0 {
            1.0
        } else {
            self.ready_to_use as f32 / total as f32
        }
    }
}

/// Helper type to handle one-shot single asset loading which consumes an asset
/// and deletes it and its dependencies after consumption.
/// Typical usecase scenario is loading an asset from a file system and using
/// its loaded data immediately without leaving asset in database.
pub enum ConsumedSingleAssetLoader<T: Component + Default> {
    Path(AssetPathStatic),
    Handle(AssetHandle),
    Data(T),
    Error(Box<dyn Error>),
}

impl<T: Component + Default> ConsumedSingleAssetLoader<T> {
    /// Creates a new `ConsumedSingleAssetLoader` instance from asset path.
    pub fn path(path: impl Into<AssetPathStatic>) -> Self {
        ConsumedSingleAssetLoader::Path(path.into())
    }

    /// Creates a new `ConsumedSingleAssetLoader` instance from asset handle.
    pub fn handle(handle: AssetHandle) -> Self {
        ConsumedSingleAssetLoader::Handle(handle)
    }

    /// Tells if asset loading is complete (either has consumed data or an error).
    pub fn is_complete(&self) -> bool {
        matches!(
            self,
            ConsumedSingleAssetLoader::Data(_) | ConsumedSingleAssetLoader::Error(_)
        )
    }

    /// Tells if asset loading is in progress (either has path or handle).
    pub fn is_in_progress(&self) -> bool {
        matches!(
            self,
            ConsumedSingleAssetLoader::Path(_) | ConsumedSingleAssetLoader::Handle(_)
        )
    }

    /// Maintains asset loader state by handling asset resolution and consuming
    /// its content when ready.
    pub fn maintain(&mut self, database: &mut AssetDatabase) {
        match self {
            ConsumedSingleAssetLoader::Path(path) => {
                let handle = match database.ensure(path.clone()) {
                    Ok(handle) => handle,
                    Err(err) => {
                        *self = ConsumedSingleAssetLoader::Error(err);
                        return;
                    }
                };
                *self = ConsumedSingleAssetLoader::Handle(handle);
            }
            ConsumedSingleAssetLoader::Handle(handle) => {
                let mut component = match database.storage.component_mut::<true, T>(handle.entity())
                {
                    Ok(component) => component,
                    Err(err) => {
                        *self = ConsumedSingleAssetLoader::Error(err.into());
                        return;
                    }
                };
                let data = std::mem::take(&mut *component);
                drop(component);
                handle.delete(database);
                *self = ConsumedSingleAssetLoader::Data(data);
            }
            _ => {}
        }
    }
}
