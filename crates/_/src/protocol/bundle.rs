use crate::{
    database::{
        handle::{AssetDependency, AssetHandle},
        path::AssetPathStatic,
    },
    fetch::AssetAwaitsResolution,
    protocol::AssetProtocol,
};
use anput::{bundle::Bundle, world::World};
use std::error::Error;

/// Represents a bundle of assets combined with their dependencies.
///
/// This struct encapsulates a bundle (`B`) along with a list of asset paths (`dependencies`)
/// representing its external dependencies.
pub struct BundleWithDependencies<B: Bundle> {
    /// The actual bundle data.
    pub bundle: B,
    /// A list of asset paths representing dependencies.
    pub dependencies: Vec<AssetPathStatic>,
}

impl<B: Bundle> BundleWithDependencies<B> {
    /// Creates a new `BundleWithDependencies` with the specified bundle and no dependencies.
    pub fn new(bundle: B) -> Self {
        Self {
            bundle,
            dependencies: Default::default(),
        }
    }

    /// Adds a single dependency to the list and returns the modified instance.
    pub fn dependency(mut self, path: impl Into<AssetPathStatic>) -> Self {
        self.dependencies.push(path.into());
        self
    }

    /// Optionally adds a dependency if the provided path is `Some`.
    pub fn maybe_dependency(mut self, path: Option<impl Into<AssetPathStatic>>) -> Self {
        if let Some(path) = path {
            self.dependencies.push(path.into());
        }
        self
    }

    /// Adds multiple dependencies from an iterable and returns the modified instance.
    pub fn dependencies(mut self, paths: impl IntoIterator<Item = AssetPathStatic>) -> Self {
        self.dependencies.extend(paths);
        self
    }
}

impl<B: Bundle> From<B> for BundleWithDependencies<B> {
    fn from(bundle: B) -> Self {
        Self {
            bundle,
            dependencies: Default::default(),
        }
    }
}

impl<B: Bundle> From<(B, Vec<AssetPathStatic>)> for BundleWithDependencies<B> {
    fn from((bundle, dependencies): (B, Vec<AssetPathStatic>)) -> Self {
        Self {
            bundle,
            dependencies,
        }
    }
}

/// Defines a trait for processing raw bytes into a `BundleWithDependencies`.
pub trait BundleWithDependenciesProcessor: Send + Sync {
    /// Returned bundle of asset components.
    type Bundle: Bundle;

    /// Processes a vector of bytes and returns a `BundleWithDependencies`.
    fn process_bytes(
        &mut self,
        bytes: Vec<u8>,
    ) -> Result<BundleWithDependencies<Self::Bundle>, Box<dyn Error>>;

    /// Maintains internal state of processor.
    #[allow(unused_variables)]
    fn maintain(&mut self, storage: &mut World) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}

impl<B, F> BundleWithDependenciesProcessor for F
where
    B: Bundle,
    F: FnMut(Vec<u8>) -> Result<BundleWithDependencies<B>, Box<dyn Error>> + Send + Sync,
{
    type Bundle = B;

    fn process_bytes(
        &mut self,
        bytes: Vec<u8>,
    ) -> Result<BundleWithDependencies<Self::Bundle>, Box<dyn Error>> {
        self(bytes)
    }
}

/// Protocol for handling bundles using a user-defined processor.
pub struct BundleAssetProtocol<Processor: BundleWithDependenciesProcessor> {
    name: String,
    processor: Processor,
}

impl<Processor: BundleWithDependenciesProcessor> BundleAssetProtocol<Processor> {
    /// Creates a new instance with the specified name and processor.
    pub fn new(name: impl ToString, processor: Processor) -> Self {
        Self {
            name: name.to_string(),
            processor,
        }
    }
}

impl<Processor: BundleWithDependenciesProcessor> AssetProtocol for BundleAssetProtocol<Processor> {
    fn name(&self) -> &str {
        &self.name
    }

    fn process_bytes(
        &mut self,
        handle: AssetHandle,
        storage: &mut World,
        bytes: Vec<u8>,
    ) -> Result<(), Box<dyn Error>> {
        let BundleWithDependencies {
            bundle,
            dependencies,
        } = self.processor.process_bytes(bytes)?;
        storage.insert(handle.entity(), bundle)?;
        for path in dependencies {
            let entity = storage.spawn((path, AssetAwaitsResolution))?;
            storage.relate::<true, _>(AssetDependency, handle.entity(), entity)?;
        }
        Ok(())
    }

    fn maintain(&mut self, storage: &mut World) -> Result<(), Box<dyn Error>> {
        self.processor.maintain(storage)
    }
}
