# ServerSnapshot imports

ServerSnapshot imports action bars and carried bags from account SavedVariables captured by the project-owned live-WoW addon. It is a local file handoff, not external game-server transport.

## Capture and installation boundary

`docs/addons/ServerSnapshot/` writes `ServerSnapshotDB` to:

```text
WTF/Account/<ACCOUNT>/SavedVariables/ServerSnapshot.lua
```

Commit `43c179b04` added carried-bag capture; `da3c1ae89` modeled capacity-backed bag state and queries; `5e0e9150d` added import and the Python deployment script. The installed 0.3.0 addon was user-approved and deployed with that script to `/syncthing/World of Warcraft/_retail_/Interface/AddOns/ServerSnapshot`. Its Lua SHA-256 is `6773149b4dee5d5a7444ad82c53631cce019677fa09348e8b1193e53b03d3f78`; its TOC SHA-256 is `3ba5ad085cc6539fb19a7000993a6d6057f7373434e8a0ece94b2775719caea5`; both match the repository source.

The installed 0.3.0 Lua/TOC hashes were reverified against repository source at final proof `29d2ee30e`. No fresh native capture exists: the actual SavedVariables remains the July 5 snapshot without `bags`, so it cannot prove import or reconstruct inventory. The user must log into the source character, run `/ssnap`, then `/reload` or log out so WoW persists a fresh database. The existing-binary RED artifact `/tmp/retail-regression/bag-red.*` reads that old no-bag capture: backpack capacity `16`, reagent capacity `0`.

## Startup selection and action bars

The normal `SavedVariablesManager` source loads the file after SavedVariables configuration and EditMode cache loading but before Blizzard addons. The importer selects `ServerSnapshotDB.lastCharacterKey` when present, otherwise the newest `capturedAt` snapshot.

Action-bar import clears simulator action bars and imports only `type = "spell"` rows, using `spellID` or `id`; empty and non-spell entries remain ignored. This seeds `SimState.action_bars`, so `HasAction`, `GetActionInfo`, and action-button setup observe captured spell slots.

## Carried bags

The addon captures bag IDs `0` through `maxBagID` (retail carried backpack, equipped bags, and reagent bag), with capacity, family mask, optional equipped-container identity, and occupied item ID/count/available hyperlink. It refreshes on login/world entry and `BAG_UPDATE_DELAYED` or `BAG_CONTAINER_UPDATE`; failed or unready reads omit `snapshot.bags` rather than encode empty inventory.

A present `snapshot.bags` is an authoritative replacement of carried containers: empty slots and unequipped bags clear stale carried state, while missing `bags` preserves it. Import validates the complete domain before mutation, uses captured capacities for container queries and placement, preserves captured links, then emits bag notifications. It imports independently of action bars. At final proof `29d2ee30e`, 373 bounded integration assertions passed, but `server_snapshot_capture_bags` failed one delayed-event fixture (`callback` nil); import and downstream bag filters passed. Commit `904b068fe` corrects that fixture after the proof, but has not received Rust GREEN verification.

The maintained [bag contract](../../specs/server-snapshot-bags.md) is the payload and behavior authority. It explicitly excludes bank/account-bank capture, general equipment capture, external networking, and SavedVariables modification.

## Sources

- [ServerSnapshot addon README](../../addons/ServerSnapshot/README.md) — live capture, persistence, and install contract
- [bag contract](../../specs/server-snapshot-bags.md) — payload, validation, replacement, and exclusions
- [server_snapshot_import.rs](../../../src/server_snapshot_import.rs) — startup import entry point
- [server_snapshot_import tests](../../../tests/server_snapshot_import.rs) — importer coverage

## See Also

- [[addon-loading]] — SavedVariables loading and startup sequence
- [[lua-api]] — action-bar and bag API surface
