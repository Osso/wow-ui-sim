# Pinned PTR Blizzard UI source

PTR 12.1.5.69594 uses an immutable Blizzard CDN content index, independent of the locally installed retail/PTR product. Gethe supplies source path discovery and revision identity, never runtime replacement bytes. Retail and Classic source acquisition remain unchanged.

## What it must do

- [x] Reject old `wowt` cache provenance for the pinned `wowxptr` build, even when required files exist.
- [x] Verify encoded BLTE identity and decoded content size/hash before accepting source bytes.
- [x] Require exact HTTP partial-content ranges; reject full-archive or wrong-range responses.
- [x] Retry transient HTTP failures with bounded backoff and respect `Retry-After`.
- [ ] Synchronize every pinned source entry, write completion only after all entries succeed, and reuse only hash-matching files.
- [ ] Load PTR startup and representative panels against this cache; a version label alone is not compatibility proof.

## How it works

- [Content-index regeneration](../ptr-cdn-content-index.md)
- [Client profiles](client-profiles.md)
- [Patch update procedure](../updating-blizzard-ui-to-a-new-patch.md)

## Implementation inventory

- `data/blizzard-ui-builds/ptr.json`: Blizzard build/config identity, Gethe revision, and source encoding/content keys with archive locations.
- `data/blizzard-ui-files/ptr.txt`: matching complete source path inventory.
- `src/blizzard_ui_sync/pinned.rs`: provenance, cache validation and bounded parallel synchronization.
- `src/blizzard_ui_sync/pinned_download.rs`: range retrieval and BLTE/content validation.
- `Cargo.toml`: direct `cascette-formats` dependency reuses the already-pinned BLTE decoder; `httpdate` parses HTTP-date `Retry-After` values.

## Tests asserting this spec

- `src/blizzard_ui_sync/pinned.rs`: exact versus legacy provenance.
- `src/blizzard_ui_sync/pinned_download.rs`: decoded bytes, corrupt encoding/content, local HTTP range and transient-response behavior.

## Known gaps (current cycle)

- [ ] Full CDN synchronization and startup/panel acceptance pending.

## Out of scope

- General-purpose TVFS support or repairing upstream CASC parsers: this runtime consumes a generated index for one pinned build.
- Gethe content fallback, relabeling installed `wowt` files, or silently following a newer product head.
