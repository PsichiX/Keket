# Asset Events

`AssetEventBindings` is a container for event listeners that will be notified
about particular asset (or all assets) progression.

User can put asset event bindings into particular asset, then some code can bind
its listener (anything that implements `AssetEventListener` trait, for example
closure or `Sender`), and once asset progression changes, that will be dispatched
to registered listeners. User can also listen for all asset events by registering
to event bindings in asset database.

```rust,ignore
// We can bind closures to asset event bindings for any asset progression tracking.
database.events.bind(|event| {
    println!("Asset closure event: {:?}", event);
    Ok(())
});

// Create channel for asset events communication.
let (tx, rx) = channel();

// Start loading asset and its dependencies.
let group = database.ensure("group://group.txt")?;
// We can also bind sender to asset event bindings.
group.ensure::<AssetEventBindings>(&mut database)?.bind(tx);

while database.is_busy() {
    database.maintain()?;
}

// Read sent events from receiver.
while let Ok(event) = rx.try_recv() {
    println!("Group channel event: {:?}", event);
}
```
