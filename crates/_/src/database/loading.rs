use crate::database::{handle::AssetHandle, path::AssetPathStatic, AssetDatabase};
use anput::component::Component;
use std::{collections::HashSet, error::Error};

/// A struct to track the loading status of assets in the database.
#[derive(Debug, Default, Clone)]
pub struct AssetsLoadingTracker {
    handles: HashSet<AssetHandle>,
}

impl AssetsLoadingTracker {
    /// Creates a modified `AssetsLoadingTracker` instance.
    ///
    /// # Arguments
    /// - `handle`: An `AssetHandle` to track.
    ///
    /// # Returns
    /// A modified `AssetsLoadingTracker` instance with the specified handle tracked.
    pub fn with(mut self, handle: AssetHandle) -> Self {
        self.track(handle);
        self
    }

    /// Creates a modified `AssetsLoadingTracker` instance with multiple handles.
    ///
    /// # Arguments
    /// - `handles`: An iterable collection of `AssetHandle` to track.
    ///
    /// # Returns
    /// A modified `AssetsLoadingTracker` instance with the specified handles tracked.
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

    /// Reports the status of assets in the database.
    ///
    /// # Arguments
    /// - `out_status`: A mutable reference to output `AssetsLoadingStatus`.
    pub fn report(&self, database: &AssetDatabase, out_status: &mut AssetsLoadingStatus) {
        out_status.clear();
        for handle in &self.handles {
            if handle.does_exists(database) {
                if handle.awaits_resolution(database) {
                    out_status.awaiting_resolution.add(*handle);
                } else if handle.bytes_are_ready_to_process(database) {
                    out_status.with_bytes_ready_to_process.add(*handle);
                } else if handle.awaits_deferred_job(database) {
                    out_status.awaiting_deferred_job.add(*handle);
                } else {
                    out_status.ready_to_use.add(*handle);
                }
            }
        }
    }
}

/// A struct to represent the loading status of assets category.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssetsLoadingStatusCategory {
    Amount(usize),
    List(Vec<AssetHandle>),
}

impl Default for AssetsLoadingStatusCategory {
    fn default() -> Self {
        AssetsLoadingStatusCategory::amount()
    }
}

impl AssetsLoadingStatusCategory {
    /// Creates an empty `AssetsLoadingStatusCategory` as amount variant.
    pub fn amount() -> Self {
        AssetsLoadingStatusCategory::Amount(0)
    }

    /// Creates an empty `AssetsLoadingStatusCategory` as list variant.
    pub fn list() -> Self {
        AssetsLoadingStatusCategory::List(Default::default())
    }

    /// Tells number of assets in the category.
    pub fn len(&self) -> usize {
        match self {
            AssetsLoadingStatusCategory::Amount(len) => *len,
            AssetsLoadingStatusCategory::List(list) => list.len(),
        }
    }

    /// Tells if the category is empty.
    pub fn is_empty(&self) -> bool {
        match self {
            AssetsLoadingStatusCategory::Amount(len) => *len == 0,
            AssetsLoadingStatusCategory::List(list) => list.is_empty(),
        }
    }

    /// Clears the category.
    pub fn clear(&mut self) {
        match self {
            AssetsLoadingStatusCategory::Amount(len) => *len = 0,
            AssetsLoadingStatusCategory::List(list) => list.clear(),
        }
    }

    /// Adds an asset handle to the category.
    pub fn add(&mut self, handle: AssetHandle) {
        match self {
            AssetsLoadingStatusCategory::Amount(len) => *len += 1,
            AssetsLoadingStatusCategory::List(list) => list.push(handle),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct AssetsLoadingStatus {
    pub awaiting_resolution: AssetsLoadingStatusCategory,
    pub with_bytes_ready_to_process: AssetsLoadingStatusCategory,
    pub awaiting_deferred_job: AssetsLoadingStatusCategory,
    pub ready_to_use: AssetsLoadingStatusCategory,
}

impl AssetsLoadingStatus {
    /// Creates a new `AssetsLoadingStatus` instance with all categories as amount variants.
    pub fn amount() -> Self {
        AssetsLoadingStatus {
            awaiting_resolution: AssetsLoadingStatusCategory::amount(),
            with_bytes_ready_to_process: AssetsLoadingStatusCategory::amount(),
            awaiting_deferred_job: AssetsLoadingStatusCategory::amount(),
            ready_to_use: AssetsLoadingStatusCategory::amount(),
        }
    }

    /// Creates a new `AssetsLoadingStatus` instance with all categories as list variants.
    pub fn list() -> Self {
        AssetsLoadingStatus {
            awaiting_resolution: AssetsLoadingStatusCategory::list(),
            with_bytes_ready_to_process: AssetsLoadingStatusCategory::list(),
            awaiting_deferred_job: AssetsLoadingStatusCategory::list(),
            ready_to_use: AssetsLoadingStatusCategory::list(),
        }
    }

    /// Clears all categories in the loading status.
    pub fn clear(&mut self) {
        self.awaiting_resolution.clear();
        self.with_bytes_ready_to_process.clear();
        self.awaiting_deferred_job.clear();
        self.ready_to_use.clear();
    }

    /// Returns the progress of the loading status.
    ///
    /// # Returns
    /// An `AssetsLoadingProgress` struct representing the progress of the loading status.
    pub fn progress(&self) -> AssetsLoadingProgress {
        AssetsLoadingProgress {
            awaiting_resolution: self.awaiting_resolution.len(),
            with_bytes_ready_to_process: self.with_bytes_ready_to_process.len(),
            awaiting_deferred_job: self.awaiting_deferred_job.len(),
            ready_to_use: self.ready_to_use.len(),
        }
    }
}

/// A struct to represent the loading progress of assets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AssetsLoadingProgress {
    pub awaiting_resolution: usize,
    pub with_bytes_ready_to_process: usize,
    pub awaiting_deferred_job: usize,
    pub ready_to_use: usize,
}

impl AssetsLoadingProgress {
    /// Returns the total number of assets in the loading progress.
    pub fn total(&self) -> usize {
        self.awaiting_resolution
            + self.with_bytes_ready_to_process
            + self.awaiting_deferred_job
            + self.ready_to_use
    }

    /// Tells if the loading progress is complete.
    pub fn is_complete(&self) -> bool {
        self.awaiting_resolution == 0
            && self.with_bytes_ready_to_process == 0
            && self.awaiting_deferred_job == 0
    }

    /// Tells if the loading progress is in progress.
    pub fn is_in_progress(&self) -> bool {
        !self.is_complete()
    }

    /// Returns the factor of the loading progress (0-1).
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
