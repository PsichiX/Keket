//! # Keket - Asset Management for Rust
//! **A powerful and flexible modular library for asset fetching, caching, and hot reloading**
//!
//! This crate provides a robust framework for managing assets in Rust, from loading and fetching assets
//! from various sources (HTTP, local file system, databases, etc.) to advanced features like asset
//! hot-reloading, deferred loading, and integration with game engines or other large projects.
//! It leverages dynamic component-based bundles to easily manage and interact with assets on demand.
//!
//! ### Key Features:
//! - **Flexible Asset Fetching**: Fetch assets from a variety of sources such as the local filesystem, HTTP URLs, or databases.
//! - **Dynamic Bundling**: Combine different pieces of asset data in a flexible way using the `DynamicBundle` container.
//! - **Hot Reloading**: Supports automatic hot reloading of assets, allowing you to dynamically update content without restarting the application.
//! - **Deferred Loading**: Perform heavy asset loading tasks asynchronously using background jobs to prevent blocking the main application thread.
//! - **Multi-Source Support**: Load assets from a range of containers (e.g., file system, in-memory collections, or databases), and even support custom ones with the flexible `ContainerPartialFetch` trait.
//! - **Error Handling**: Rich error handling mechanism that makes debugging asset loading problems quick and easy.
//!
//! ### Example Usage:
//! ```rust
//! use keket::{
//!     database::{path::AssetPath, AssetDatabase},
//!     fetch::file::FileAssetFetch,
//!     protocol::{bytes::BytesAssetProtocol, text::TextAssetProtocol},
//! };
//! use std::{error::Error, fs::Metadata, path::PathBuf};
//!
//! fn main() -> Result<(), Box<dyn Error>> {
//!     # std::env::set_current_dir("../../"); // change path for tests run.
//!     let mut database = AssetDatabase::default()
//!         .with_protocol(TextAssetProtocol)
//!         .with_protocol(BytesAssetProtocol)
//!         .with_fetch(FileAssetFetch::default().with_root("resources"));
//!
//!     let lorem = database.ensure("text://lorem.txt")?;
//!     println!("Lorem Ipsum: {}", lorem.access::<&String>(&database));
//!
//!     let trash = database.ensure("bytes://trash.bin")?;
//!     println!("Bytes: {:?}", trash.access::<&Vec<u8>>(&database));
//!
//!     for (asset_path, file_path, metadata) in database
//!         .storage
//!         .query::<true, (&AssetPath, &PathBuf, &Metadata)>()
//!     {
//!         println!(
//!             "Asset: `{}` at location: {:?} has metadata: {:#?}",
//!             asset_path, file_path, metadata
//!         );
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Use Cases:
//! - **Game Development**: Load and manage game assets (e.g., textures, sound files) from different sources and automatically reload them during runtime.
//! - **Web Applications**: Dynamically fetch and cache static files from HTTP endpoints and the file system.
//! - **Data-Driven Applications**: Fetch and manage resources such as user-generated content, configuration files, or assets in a multi-environment scenario.
//! - **Asset-Centric Systems**: Any system that requires efficient asset management with flexible, reactive fetching behaviors.
//!
//! Whether you're building games, web apps, or anything else that requires managing and fetching assets, this crate has you covered with ease and flexibility.
//! Enjoy on-the-fly reloading, asynchronous fetching, and a simple, intuitive API designed with efficiency in mind.

pub mod database;
pub mod fetch;
pub mod protocol;

pub mod third_party {
    pub use anput;
}
