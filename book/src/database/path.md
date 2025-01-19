# Asset Path

`AssetPath` serves as thin wrapper around copy-on-write string with cached ranges
of standarized path (`<protocol>://<path>?<key1>=<value1>&<flag>`) components:

- protocol
- path
- optional meta data

Example asset paths:

- `ui/texts/lorem.txt` - asset path without protocol, only path part.
- `text://ui/texts/lorem.txt` - asset path with protocol.
- `text://ui/texts/lorem.txt?v=3&uppercase` - asset path with path and meta key-values.

Most common asset paths are ones with protocol and meta key-values. Paths without
protocols are possible, but not useful, unless you register an asset protocol
that handles them (which is almost never the case, but one could have it as
protocol resolved by any other means - since library is modular, user can setup
database however they like).

Asset paths are useful mostly in getting asset handles by their asset paths, or
storage queries/lookups by specific asset path.

```rust,ignore
let path = AssetPath::new("text://lorem.txt?v=3&uppercase");

println!("- protocol: {:?}", path.protocol());
for segment in path.path_parts() {
    println!("  - path segment: {:?}", segment);
}
for (key, value) in path.meta_items() {
    println!("  - path meta key: {:?} value: {:?}", key, value);
}

let lorem = database.ensure(path)?;
```
