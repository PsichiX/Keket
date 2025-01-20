# Introduction

Welcome to `Keket`!

Modern, flexible, modular Asset Management library built on top of ECS as its
storage.

```rust,ignore
{{#rustdoc_include ../../crates/_/examples/01_hello_world.rs:main}}
```

## Goal

`Keket` started as an experiment to tell how asset management would look like
with ECS as its storage, how we could make it play with modularity, what benefits
does ECS storage gives us.

Soon after first version got done, i've realized that it is quite powerful and
easily extendable, when we treat assets as entities with components as their data
(and meta data) and asset loaders (and systems outside of asset management) can
process and change assets in bulk the way they need, while not forcing any
particular closed specs structure on the assets themselves.
