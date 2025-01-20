# Asset Events

`AssetEventBindings` is a container for event listeners that will be notified
about particular asset (or all assets) progression.

User can put asset event bindings into particular asset, then some code can bind
its listener (anything that implements `AssetEventListener` trait, for example
closure or `Sender`), and once asset progression changes, that will be dispatched
to registered listeners. User can also listen for all asset events by registering
to event bindings in asset database.

```rust,ignore
{{#rustdoc_include ../../../crates/_/examples/04_events.rs:events}}
```
