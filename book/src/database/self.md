# Asset Database

Let's dive into central point of the library - asset database.

`AssetDatabase` is a thin, higher level abstraction for underlying (public) ECS
storage where assets data live.

ECS as storage allowed to treat assets as rows in database tables, where single
asset can have multiple data columns associated with given asset. This opened up
a possibility for fast queries and lookups on multiple entities that pass given
requirements (set of components).

This is very important for systems which process and modify assets in specific
for them ways (in `Keket` those systems are asset fetch engines and asset
protocols - more about them later in the book).

We can use queries and lookups to process only assets that have given set of
requirements based on the data they store. For example when we load assets with
`FileAssetFetch`, we get not only asset data, but also meta data as `PathBuf` and
`Metadata` components, so if we want to for example scan database for all assets
that come from file system, we can just query them with `PathBuf` component and
use that fast query for asset statistics report.

```rust,ignore
{{#rustdoc_include ../../../crates/_/examples/01_hello_world.rs:fs_query}}
```

Since storage is just an ECS, every asset entity always must store at least
`AssetPath` component to be considered valid asset - every component other than
asset path is considered asset data that is either usable for outside systems,
or for asset fetch engines and asset protocols as meta data for them in asset
resolution process.

> When an asset entity stores some components without asset path, then it is
> never considered an actual asset, and rather an entity not important to asset
> management - some systems might find it useful for them, but it is highly not
> advised to put there anything other than assets.
