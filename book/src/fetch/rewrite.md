# Rewrite asset path wrapper

`RewriteAssetFetch` allows to rewrite requested asset path to some other.
This is useful for scenarios like localized assets or assets versioning, where
there might be different versions of assets based on some runtime state.

```rust,ignore
let language = Arc::new(RwLock::new("en"));
let language2 = language.clone();

let mut database = AssetDatabase::default()
    .with_protocol(TextAssetProtocol)
    .with_fetch(RewriteAssetFetch::new(
        FileAssetFetch::default().with_root("resources"),
        move |path| {
            // Make localized asset path based on current language settings.
            Ok(AssetPath::from_parts(
                path.protocol(),
                &format!(
                    "{}.{}{}",
                    path.path_without_extension(),
                    *language2.read().unwrap(),
                    path.path_dot_extension().unwrap_or_default()
                ),
                path.meta(),
            ))
        },
    ));

// Gets `text://localized.en.txt`.
let asset = database.ensure("text://localized.txt")?;
println!("English: {}", asset.access::<&String>(&database));

// Change language.
*language.write().unwrap() = "de";
database.storage.clear();

// Gets `text://localized.de.txt`.
let asset = database.ensure("text://localized.txt")?;
println!("German: {}", asset.access::<&String>(&database));
```

> Rewritten asset paths do not change path in asset entity - this is deliberate
> design decision to make outside systems not care about possible asset path
> change when trying to resolve asset handle by its path.
