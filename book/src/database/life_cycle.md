# Asset life cycle

There is standarized progression in asset life cycle:

- **Awaits for resolution** - database triggers fetching bytes for these.
- **Bytes are ready to be processed** - database takes those loaded bytes from asset
  and runs asset protocol (by asset path protocol part) on these bytes.
- **Asset is ready to use** - bytes are decoded into asset components and put in
  given asset for use by outside systems.
- **Asset is unloaded** - asset entity gets despawned and when ensured again it will
  be marked as awaiting resolution.

> It might be the case where sometimes one or more of these steps are missed, for
> example when asset is added manually (user has all asset components made and
> spawns asset directly into storage, without fetching bytes, etc) - in that case
> given asset never goes through asset fetch engines and asset protocols (yet is
> still discoverable by change detection).
