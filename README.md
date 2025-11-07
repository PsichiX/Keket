# Keket - Asset Management for Rust

[![crates-io version](https://img.shields.io/crates/v/keket)](https://crates.io/keket)
[![docs.rs](https://img.shields.io/docsrs/keket)](https://docs.rs/keket)
[![book](https://img.shields.io/badge/book-Keket-orange?style=flat&link=https%3A%2F%2Fpsichix.github.io%2FKeket%2F)](https://psichix.github.io/Keket)
[![workflow](https://img.shields.io/github/actions/workflow/status/PsichiX/Keket/rust.yml)](https://github.com/PsichiX/Keket/actions/workflows/rust.yml)

**Database-like Asset management on top of ECS storage.**

---

This crate provides a robust framework for managing assets in Rust, from loading and fetching assets
from various sources (HTTP, local file system, databases, etc.) to advanced features like asset
hot-reloading, deferred and async loading, and integration with game engines or other large projects.
It leverages dynamic component-based bundles to easily manage and interact with assets on demand.

## Key Features

- **Flexible Asset Fetching**: Fetch assets from a variety of sources such as the local filesystem, HTTP URLs, or databases.
- **Dynamic Bundling**: Combine different pieces of asset data in a flexible way using the `DynamicBundle` container.
- **Hot Reloading**: Supports automatic hot reloading of assets, allowing you to dynamically update content without restarting the application.
- **Deferred Loading**: Perform heavy asset loading tasks asynchronously using background jobs to prevent blocking the main application thread.
- **Async Loading**: Perform heavy asset loading tasks asynchronously using custom async/await futures.
- **Multi-Source Support**: Load assets from a range of containers (e.g., file system, in-memory collections, or databases), and even support custom ones with the flexible `ContainerPartialFetch` trait.
- **Error Handling**: Rich error handling mechanism that makes debugging asset loading problems quick and easy.

## Example Usage

```rust
use keket::{
    database::{path::AssetPath, AssetDatabase},
    fetch::file::FileAssetFetch,
    protocol::{bundle::BundleAssetProtocol, bytes::BytesAssetProtocol, text::TextAssetProtocol},
};
use serde_json::Value;
use std::{error::Error, fs::Metadata, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    let mut database = AssetDatabase::default()
        .with_protocol(TextAssetProtocol)
        .with_protocol(BytesAssetProtocol)
        .with_protocol(BundleAssetProtocol::new("json", |bytes: Vec<u8>| {
            Ok((serde_json::from_slice::<Value>(&bytes)?,).into())
        }))
        .with_fetch(FileAssetFetch::default().with_root("resources"));

    let lorem = database.ensure("text://lorem.txt")?;
    println!("Lorem Ipsum: {}", lorem.access::<&String>(&database));

    let person = database.ensure("json://person.json")?;
    println!("Person: {:#?}", person.access::<&Value>(&database));

    let trash = database.ensure("bytes://trash.bin")?;
    println!("Bytes: {:?}", trash.access::<&Vec<u8>>(&database));

    for (asset_path, file_path, metadata) in database
        .storage
        .query::<true, (&AssetPath, &PathBuf, &Metadata)>()
    {
        println!(
            "Asset: `{}` at location: {:?} has metadata: {:#?}",
            asset_path, file_path, metadata
        );
    }

    Ok(())
}
```

More examples:

- [Hello World!](https://github.com/PsichiX/Keket/tree/master/crates/_/examples/01_hello_world.rs)
- [ZIP archive](https://github.com/PsichiX/Keket/tree/master/crates/_/examples/02_zip.rs)
- [Dependencies](https://github.com/PsichiX/Keket/tree/master/crates/_/examples/03_dependencies.rs)
- [Events](https://github.com/PsichiX/Keket/tree/master/crates/_/examples/04_events.rs)
- [Deferred loading](https://github.com/PsichiX/Keket/tree/master/crates/_/examples/05_deferred_fetch.rs)
- [Routing](https://github.com/PsichiX/Keket/tree/master/crates/_/examples/06_router_fetch.rs)
- [Fallback assets](https://github.com/PsichiX/Keket/tree/master/crates/_/examples/07_fallback.rs)
- [DLC asset packs](https://github.com/PsichiX/Keket/tree/master/crates/_/examples/08_dlcs_asset_packs.rs)
- [Hot reloading](https://github.com/PsichiX/Keket/tree/master/crates/_/examples/09_hot_reloading.rs)
- [Asset references](https://github.com/PsichiX/Keket/tree/master/crates/_/examples/10_references.rs)
- [Custom simple protocol](https://github.com/PsichiX/Keket/tree/master/crates/_/examples/11_custom_protocol_simple.rs)
- [Custom advanced protocol](https://github.com/PsichiX/Keket/tree/master/crates/_/examples/12_custom_protocol_advanced.rs)
- [Custom fetch engine](https://github.com/PsichiX/Keket/tree/master/crates/_/examples/13_custom_fetch.rs)
- [Assets versioning](https://github.com/PsichiX/Keket/tree/master/crates/_/examples/14_assets_versioning.rs)
- [Localized assets](https://github.com/PsichiX/Keket/tree/master/crates/_/examples/15_localized_assets.rs)
- [Extract from asset](https://github.com/PsichiX/Keket/tree/master/crates/_/examples/16_extract_from_asset.rs)
- [Smart references](https://github.com/PsichiX/Keket/tree/master/crates/_/examples/17_smart_references.rs)
- [Temporary fetch](https://github.com/PsichiX/Keket/tree/master/crates/_/examples/18_temporary_fetch.rs)
- [Loading progress](https://github.com/PsichiX/Keket/tree/master/crates/_/examples/19_loading_progress.rs)
- [Consumed asset load](https://github.com/PsichiX/Keket/tree/master/crates/_/examples/20_consumed_asset_load.rs)
- [Store asset](https://github.com/PsichiX/Keket/tree/master/crates/_/examples/21_store_asset.rs)
- [Store custom asset](https://github.com/PsichiX/Keket/tree/master/crates/_/examples/22_store_custom_asset.rs)
- [Async loading](https://github.com/PsichiX/Keket/tree/master/crates/_/examples/23_future_fetch.rs)
- [Async saving](https://github.com/PsichiX/Keket/tree/master/crates/_/examples/24_future_store.rs)
- [Tokio + Axum web server asset provider](https://github.com/PsichiX/Keket/tree/master/crates/_/examples/25_tokio_axum.rs)
- [Throttled loading](https://github.com/PsichiX/Keket/tree/master/crates/_/examples/26_throttled_fetch.rs)
- [Async protocol](https://github.com/PsichiX/Keket/tree/master/crates/_/examples/27_future_protocol.rs)
- [Asset graph](https://github.com/PsichiX/Keket/tree/master/crates/graph/examples/hello_graph.rs)
- [HTTP fetch engine](https://github.com/PsichiX/Keket/tree/master/crates/http/examples/hello_http.rs)
- [REDB fetch engine](https://github.com/PsichiX/Keket/tree/master/crates/redb/examples/hello_redb.rs)
- [Asset server fetch engine](https://github.com/PsichiX/Keket/tree/master/crates/client/examples/hello_client.rs)
- [In-game scenario](https://github.com/PsichiX/Keket/tree/master/crates/_/examples/ingame.rs)

## Architecture

Keket asset management design is built around modularity to allow users to change parts to build their asset pipeline tailored to their needs.

Key concepts:

- `AssetDatabase` - central place where assets live. ECS storage allows for ergonomics of access and processing of huge amount of data at once in database-like manner.
- `AssetHandle` - wrapper around asset entity that allows easy direct operations and access to resolved asset. Asset database
- `AssetPath` - specialized path that describes an asset identifier (`<protocol>://<path/to/asset.extension>?<meta-information>`).
  Examples:
  - `image://grass.png`
  - `mesh://teapot.gltf?lod=2`
- `AssetRef` - wrapper around asset path and cached asset handle. Useful for serialization. Allows to resolve asset once and reuse cached handle later.
- `SmartAssetRef` - Reference-counted version of `AssetRef` that allows automatic unloading assets that aren't referenced anywhere.
- `AssetFetch` - an engine telling where from asset bytes are loaded. Basic one is `FileAssetFetch`.
- `AssetStore` - an engine telling where to asset bytes are stored. Basic one is `FileAssetStore`.
- `AssetProtocol` - an engine telling how fetched asset bytes are decoded into asset components, selected based on asset path protocol part. For example `TextAssetProtocol` or `BytesAssetProtocol`.

## Use Cases

- **Game Development**: Load and manage game assets (e.g., textures, sound files) from different sources and automatically reload them during runtime.
- **Web Applications**: Dynamically fetch and cache static files from HTTP endpoints and the file system.
- **Data-Driven Applications**: Fetch and manage resources such as user-generated content, configuration files, or assets in a multi-environment scenario.
- **Asset-Centric Systems**: Any system that requires efficient asset management with flexible, reactive fetching behaviors.

Whether you're building games, web apps, or anything else that requires managing and fetching assets, this crate has you covered with ease and flexibility.
Enjoy on-the-fly reloading, asynchronous fetching, and a simple, intuitive API designed with efficiency in mind.
