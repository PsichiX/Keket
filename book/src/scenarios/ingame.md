# Basic in-game setup

Here we will showcase a basic usage of `Keket` integrated with some application.
In this example we will use [Spitfire](https://github.com/PsichiX/spitfire) crate.

<details>
<summary>See `use` section</summary>

```rust,ignore
{{#rustdoc_include ../../../crates/_/examples/ingame.rs:use}}
```

</details>

## Main function

Main function looks boring - all we do is we run `App` with `State` object.

```rust,ignore
{{#rustdoc_include ../../../crates/_/examples/ingame.rs:main}}
```

## State struct

`State` type holds `AssetDatabase` along with some other data useful for drawing,
fixed time step mechanism and referencing assets.

```rust,ignore
{{#rustdoc_include ../../../crates/_/examples/ingame.rs:state_struct}}
```

## State `Default` impl

In `Default` implementatio we setup app state.

Take a look at how we setup `AssetDatabase` protocols:

```rust,ignore
{{#rustdoc_include ../../../crates/_/examples/ingame.rs:state_impl_default}}
```

In there you can see bundle asset protocols wrapping custom shader and texture
asset processors.

## State `AppState` impl

Then we implement `AppState` for our `State` type, where in `on_init` we setup
graphics and load scene group asset, and in `on_redraw` we run asset database
maintanance periodically and then try to render Ferris sprite only if shader and
texture assets are ready.

```rust,ignore
{{#rustdoc_include ../../../crates/_/examples/ingame.rs:state_impl_appstate}}
```

## State impl

In `State` implementation we finally do our first interesting bit - since asset
protocols does not have access to our state, we can process prepared shader and
texture assets and then build GPU objects for these assets and put them into
their assets.

```rust,ignore
{{#rustdoc_include ../../../crates/_/examples/ingame.rs:state_impl}}
```

## Texture asset processor

Now let's see how we have made our texture asset processor:

```rust,ignore
{{#rustdoc_include ../../../crates/_/examples/ingame.rs:texture_protocol}}
```

All we do here is we decode PNG image into texture decoded bytes along with
texture dimensions.

## Shader asset processor

Shader asset processor is a lot more interesting:

```rust,ignore
{{#rustdoc_include ../../../crates/_/examples/ingame.rs:shader_protocol}}
```

Since this asset is just a descriptor that holds references to dependencies such
as vertex anf fragment shader code, in asset processing we only schedule
dependency loading and then in `maintain` method we scan database storage for
`ShaderAssetInfo` component, along with dependency relations, and we test if all
dependencies are resolved (they are text assets so all should have `String`
component when complete). When all dependencies are complete, we can read them
and construct `ShaderAsset` with shader programs code for game to build GPU
shader objects.
