use crate::{
    database::{
        handle::{AssetDependency, AssetHandle},
        path::AssetPath,
    },
    fetch::AssetAwaitsResolution,
    protocol::AssetProtocol,
};
use anput::{bundle::Bundle, world::World};
use std::error::Error;

pub struct BundleWithDependencies<B: Bundle> {
    pub bundle: B,
    pub dependencies: Vec<AssetPath<'static>>,
}

impl<B: Bundle> BundleWithDependencies<B> {
    pub fn new(bundle: B) -> Self {
        Self {
            bundle,
            dependencies: Default::default(),
        }
    }

    pub fn dependency(mut self, path: impl Into<AssetPath<'static>>) -> Self {
        self.dependencies.push(path.into());
        self
    }

    pub fn dependencies(mut self, paths: impl IntoIterator<Item = AssetPath<'static>>) -> Self {
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

impl<B: Bundle> From<(B, Vec<AssetPath<'static>>)> for BundleWithDependencies<B> {
    fn from((bundle, dependencies): (B, Vec<AssetPath<'static>>)) -> Self {
        Self {
            bundle,
            dependencies,
        }
    }
}

pub trait BundleWithDependenciesProcessor: Send + Sync {
    type Bundle: Bundle;

    fn process_bytes(
        &mut self,
        bytes: Vec<u8>,
    ) -> Result<BundleWithDependencies<Self::Bundle>, Box<dyn Error>>;
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

pub struct BundleAssetProtocol<Processor: BundleWithDependenciesProcessor> {
    name: String,
    processor: Processor,
}

impl<Processor: BundleWithDependenciesProcessor> BundleAssetProtocol<Processor> {
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
}
