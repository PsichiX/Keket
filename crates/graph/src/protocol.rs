use keket::{
    database::{
        inspector::AssetInspector,
        path::AssetPathStatic,
        reference::{AssetRef, SmartAssetRef},
    },
    protocol::bundle::{
        BundleWithDependencies, BundleWithDependenciesProcessor, StoreWithDependencies,
    },
    third_party::anput::component::Component,
};
use std::{
    error::Error,
    ops::{Deref, DerefMut},
};

/// Defines a trait for components that can have asset dependencies.
///
/// This trait is useful for `AssetTreeProcessor` protocol to gather asset
/// dependencies to load.
pub trait AssetTree: Component {
    /// Returns an iterator over the asset dependencies of this component.
    ///
    /// The dependencies are represented as `AssetPathStatic`, which
    /// are static paths to the assets that this component depends on.
    ///
    /// # Returns
    /// An iterator over `AssetPathStatic` representing the asset dependencies.
    fn asset_dependencies(&self) -> impl IntoIterator<Item = AssetPathStatic>;
}

impl<T: AssetTree> AssetTree for Option<T> {
    fn asset_dependencies(&self) -> impl IntoIterator<Item = AssetPathStatic> {
        self.as_ref()
            .into_iter()
            .flat_map(|asset| asset.asset_dependencies())
    }
}

impl AssetTree for AssetPathStatic {
    fn asset_dependencies(&self) -> impl IntoIterator<Item = AssetPathStatic> {
        std::iter::once(self.clone().into_static())
    }
}

impl AssetTree for AssetRef {
    fn asset_dependencies(&self) -> impl IntoIterator<Item = AssetPathStatic> {
        std::iter::once(self.path().clone().into_static())
    }
}

impl AssetTree for SmartAssetRef {
    fn asset_dependencies(&self) -> impl IntoIterator<Item = AssetPathStatic> {
        std::iter::once(self.path().clone().into_static())
    }
}

/// A wrapper type for components that do not have any asset dependencies.
///
/// This type is useful when you want to use a component as an asset tree
/// but it does not implement `AssetTree` trait or should not give any dependencies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct NoDeps<T: Component>(pub T);

impl<T: Component> Deref for NoDeps<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Component> DerefMut for NoDeps<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Component> AssetTree for NoDeps<T> {
    fn asset_dependencies(&self) -> impl IntoIterator<Item = AssetPathStatic> {
        []
    }
}

/// AssetTreeProcessor is a processor for components that implement the
/// `AssetTree` trait. It allows deserializing and serializing assets, as well
/// as processing them with dependencies based on what their `asset_dependencies`
/// method reports.
pub struct AssetTreeProcessor<T: AssetTree> {
    #[allow(clippy::type_complexity)]
    deserializer: Box<dyn FnMut(Vec<u8>) -> Result<T, Box<dyn Error>> + Send + Sync>,
    #[allow(clippy::type_complexity)]
    serializer: Option<Box<dyn FnMut(&T) -> Result<Vec<u8>, Box<dyn Error>> + Send + Sync>>,
}

impl<T: AssetTree> AssetTreeProcessor<T> {
    /// Creates a new `AssetTreeProcessor` with the given deserializer.
    ///
    /// The deserializer is a function that takes a `Vec<u8>` and returns a
    /// `Result<T, Box<dyn Error>>`, where `T` is the type of the asset tree component.
    ///
    /// # Arguments
    /// - `deserializer`: A function that deserializes bytes into an asset tree
    ///   component of type `T`.
    ///
    /// # Returns
    /// A new instance of `AssetTreeProcessor`.
    pub fn new(
        deserializer: impl FnMut(Vec<u8>) -> Result<T, Box<dyn Error>> + Send + Sync + 'static,
    ) -> Self {
        Self {
            deserializer: Box::new(deserializer),
            serializer: None,
        }
    }

    /// Sets a serializer for the asset tree component.
    ///
    /// This method allows you to provide a function that serializes an asset tree
    /// component of type `T` into a `Vec<u8>`. The serializer must be a function that
    /// takes a reference to `T` and returns a `Result<Vec<u8>, Box<dyn Error>>`.
    ///
    /// # Arguments
    /// - `serializer`: A function that serializes an asset tree component of type `T`
    ///   into a `Vec<u8>`.
    ///
    /// # Returns
    /// A mutable reference to `Self`, allowing for method chaining.
    pub fn with_serializer(
        mut self,
        serializer: impl FnMut(&T) -> Result<Vec<u8>, Box<dyn Error>> + Send + Sync + 'static,
    ) -> Self {
        self.serializer = Some(Box::new(serializer));
        self
    }
}

impl<T: AssetTree> BundleWithDependenciesProcessor for AssetTreeProcessor<T> {
    type Bundle = (T,);

    fn process_bytes(
        &mut self,
        bytes: Vec<u8>,
    ) -> Result<BundleWithDependencies<Self::Bundle>, Box<dyn Error>> {
        let asset = (self.deserializer)(bytes)?;
        let dependencies = T::asset_dependencies(&asset)
            .into_iter()
            .collect::<Vec<_>>();
        let mut result = BundleWithDependencies::new((asset,));
        result.dependencies = dependencies;
        Ok(result)
    }

    fn produce_bytes(
        &mut self,
        inspector: AssetInspector,
    ) -> Result<StoreWithDependencies, Box<dyn Error>> {
        let Some(serializer) = self.serializer.as_mut() else {
            return Err(format!(
                "Serializer is not set for AssetTreeProcessor<{}>",
                std::any::type_name::<T>(),
            )
            .into());
        };
        let component = inspector.access_checked::<&T>().ok_or_else(|| {
            format!(
                "Could not get {} asset component",
                std::any::type_name::<T>(),
            )
        })?;
        let bytes = (serializer)(component)?;
        let dependencies = T::asset_dependencies(component)
            .into_iter()
            .collect::<Vec<_>>();
        let mut result = StoreWithDependencies::new(bytes);
        result.dependencies = dependencies;
        Ok(result)
    }
}
