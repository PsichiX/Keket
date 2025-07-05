use crate::protocol::AssetTree;
use keket::{
    database::{
        AssetDatabase,
        handle::AssetHandle,
        path::AssetPathStatic,
        reference::{AssetRef, AssetResolved},
    },
    third_party::anput::component::Component,
};
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

/// AssetNode represents a node in the asset graph, which is a reference to an
/// asset that can be resolved to a specific component in asset.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "AssetPathStatic", into = "AssetPathStatic")]
pub struct AssetNode<T: AssetTree> {
    inner: AssetRef,
    #[serde(skip)]
    _phantom: PhantomData<fn() -> T>,
}

impl<T: AssetTree> AssetNode<T> {
    /// Creates a new AssetNode from a static asset path.
    ///
    /// # Arguments
    /// - `path`: The path to the asset, which can be any type that can be
    ///   converted into `AssetPathStatic`.
    ///
    /// # Returns
    /// A new `AssetNode` instance that references the asset at the given path.
    pub fn new(path: impl Into<AssetPathStatic>) -> Self {
        Self::from_ref(AssetRef::new(path))
    }

    /// Creates a new AssetNode from an existing AssetRef.
    ///
    /// # Arguments
    /// - `inner`: The `AssetRef` that this node will reference.
    ///
    /// # Returns
    /// A new `AssetNode` instance that wraps the provided `AssetRef`.
    pub fn from_ref(inner: AssetRef) -> Self {
        Self {
            inner,
            _phantom: PhantomData,
        }
    }

    /// Returns a reference to the inner `AssetRef` of this node.
    pub fn as_ref(&self) -> AssetRef {
        self.inner.clone()
    }

    /// Invalidates the asset node, making internal `AssetRef` unresolved.
    ///
    /// # Returns
    /// A `Result` indicating success or failure. If successful, the asset node
    /// is marked as invalidated, and any future attempts to resolve it will
    /// require re-fetching or re-resolving the asset.
    pub fn invalidate(&self) -> Result<(), Box<dyn Error>> {
        self.inner.invalidate()
    }

    /// Returns the path of the asset node.
    ///
    /// # Returns
    /// A reference to the `AssetPathStatic` associated with this node.
    pub fn path(&self) -> &AssetPathStatic {
        self.inner.path()
    }

    /// Returns the handle of the asset node.
    ///
    /// # Returns
    /// A `Result` containing the `AssetHandle` if successful, or an error if
    /// the handle cannot be retrieved.
    pub fn handle(&self) -> Result<AssetHandle, Box<dyn Error>> {
        self.inner.handle()
    }

    /// Resolves the asset node to a specific component in the asset database.
    ///
    /// # Arguments
    /// - `database`: A reference to the `AssetDatabase` where the asset is stored.
    ///
    /// # Returns
    /// A `Result` containing an `AssetNodeResolved` instance if successful, or an
    /// error if the resolution fails. The `AssetNodeResolved` provides access to
    /// the resolved component, allowing read and write operations.
    pub fn resolve<'a>(
        &'a self,
        database: &'a AssetDatabase,
    ) -> Result<AssetNodeResolved<'a, T>, Box<dyn Error>> {
        self.inner
            .resolve(database)
            .map(|resolved| AssetNodeResolved {
                inner: resolved,
                _phantom: PhantomData,
            })
    }

    /// Ensures that the asset node is resolved and available in the asset database.
    ///
    /// # Arguments
    /// - `database`: A mutable reference to the `AssetDatabase` where the asset is stored.
    ///
    /// # Returns
    /// A `Result` containing an `AssetNodeResolved` instance if successful, or an
    /// error if the resolution fails. The `AssetNodeResolved` provides access to
    /// the resolved component, allowing read and write operations.
    pub fn ensure<'a>(
        &'a self,
        database: &'a mut AssetDatabase,
    ) -> Result<AssetNodeResolved<'a, T>, Box<dyn Error>> {
        self.inner
            .ensure(database)
            .map(|resolved| AssetNodeResolved {
                inner: resolved,
                _phantom: PhantomData,
            })
    }
}

impl<T: AssetTree> Clone for AssetNode<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<T: AssetTree> From<AssetPathStatic> for AssetNode<T> {
    fn from(path: AssetPathStatic) -> Self {
        Self::new(path)
    }
}

impl<T: AssetTree> From<AssetNode<T>> for AssetPathStatic {
    fn from(value: AssetNode<T>) -> Self {
        value.path().clone()
    }
}

impl<T: AssetTree> AssetTree for AssetNode<T> {
    fn asset_dependencies(&self) -> impl IntoIterator<Item = AssetPathStatic> {
        std::iter::once(self.inner.path().clone().into_static())
    }
}

/// AssetNodeResolved represents a resolved asset node, providing access to the
/// component associated with the asset. It allows both read and write access
/// to the component, ensuring that the asset is properly resolved before
/// accessing its data.
pub struct AssetNodeResolved<'a, T: Component> {
    inner: AssetResolved<'a>,
    _phantom: PhantomData<fn() -> T>,
}

impl<'a, T: Component> AssetNodeResolved<'a, T> {
    /// Gives read access to component of the asset.
    ///
    /// # Returns
    /// An `Option` containing a reference to the component if it is accessible,
    /// or `None` if the component is not accessible.
    pub fn read(&self) -> Option<&T> {
        self.inner.access_checked::<&T>()
    }

    /// Gives write access to component of the asset.
    ///
    /// # Returns
    /// An `Option` containing a mutable reference to the component if it is
    /// accessible, or `None` if the component is not accessible.
    pub fn write(&self) -> Option<&mut T> {
        self.inner.access_checked::<&mut T>()
    }

    /// Gives unchecked read access to component of the asset.
    ///
    /// # Returns
    /// A reference to the component, allowing read access without checking
    /// if the component is accessible. This method can panic if the component
    /// is not accessible, so it should be used with caution.
    pub fn read_unchecked(&self) -> &T {
        self.inner.access::<&T>()
    }

    /// Gives unchecked write access to component of the asset.
    ///
    /// # Returns
    /// A mutable reference to the component, allowing write access without
    /// checking if the component is accessible. This method can panic if the
    /// component is not accessible, so it should be used with caution.
    pub fn write_unchecked(&self) -> &mut T {
        self.inner.access::<&mut T>()
    }
}

impl<'a, T: Component> Deref for AssetNodeResolved<'a, T> {
    type Target = AssetResolved<'a>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, T: Component> DerefMut for AssetNodeResolved<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
