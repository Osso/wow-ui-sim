# CASC asset cache

The simulator reads textures and fonts from a live WoW install via three stacked caches. Each layer has a distinct purpose, lifetime, and on-disk location; this page explains what they cache, how they're wired together, and the measured cost of hits and misses.

## Cache layers

| Layer | Stores | Location | Built / refreshed by | Lifetime |
|---|---|---|---|---|
| FDID resolution | `fdid → (content_key, encoding_key)` | `~/.cache/asset-resolver/casc/<product>/<build-key>/resolution.sqlite` (~85 MB, ~1.87 M rows) | `asset-resolver` on first miss or stale cache; `casc_refresh` can prewarm it | Persistent across processes; rebuilt when WoW patches |
| BLP byte cache | Extracted BLP files keyed by listfile path | `~/.cache/wow-ui-sim/casc-extract/<Interface/...>.blp` | `asset_resolver::ensure_cached` on first miss | Persistent across processes; never invalidated automatically |
| Blizzard UI source cache | Extracted Blizzard Lua/XML/TOC source files | `~/.cache/wow-ui-sim/blizzard-ui/<profile>/AddOns/<Blizzard_...>` | `wow-cli casc sync-blizzard-ui` or GUI startup when the cache is missing | Persistent across processes; guarded by `.wow-ui-sim-blizzard-ui-complete` |
| In-memory texture cache | `TextureData` (decoded RGBA) keyed by normalised wow path | `TextureManager::cache` | `TextureManager::load` on first hit | Per-process |

The resolution sqlite is generated cache data, not repo data. wow-ui-sim constructs `asset-resolver` with an explicit cache root: `$ASSET_RESOLVER_CACHE_DIR` when set, otherwise `$XDG_CACHE_HOME/asset-resolver` or `~/.cache/asset-resolver`. The path is product/build-key scoped so retail, classic, and future patch builds do not overwrite each other. Runtime loading no longer depends on `GAME_ENGINE_SHARED_ROOT` or a sibling game-engine checkout.

Without the resolution sqlite, first CASC use reads the local build config, extracts fresh `root.bin` and `encoding.bin` from the WoW install into the cache directory, and builds the sqlite. Warm starts become a single `Connection::open` plus per-FDID sqlite SELECTs.

## Lookup flow

```
TextureManager::load(wow_path)
  ├─ in-memory hit?         → return TextureData     [≈ 2 µs]
  ├─ resolve_path(normalised)
  │    ├─ addon dir?        → file path
  │    ├─ try_casc_resolve
  │    │    ├─ asset_resolver::lookup_path           (listfile, multiple ext candidates)
  │    │    ├─ out_path.exists()?                    [BLP cache hit → return path]
  │    │    └─ asset_resolver::ensure_cached(fdid)
  │    │         ├─ first call per process: init CASC
  │    │         │    ├─ Installation::open + initialize  (load 45 .idx + open .data archives)
  │    │         │    └─ open/rebuild resolution sqlite   (cache hit, or root/encoding refresh)
  │    │         ├─ resolve_fdid(fdid)               (sqlite SELECT → encoding key)
  │    │         ├─ read_file_by_encoding_key        (archive read + BLTE decompress)
  │    │         └─ write to out_path                (BLP cache populated)
  │    └─ extracted-Interface fallback under <wow>/_retail_/BlizzardInterfaceArt
  └─ load_texture_file(path) + insert in-memory cache
```

`Installation::initialize` does **not** parse `root.bin` or `encoding.bin` on warm starts — that work is delegated to the resolution sqlite. If the sqlite is missing or stale, `asset-resolver` rebuilds it once from the current local CASC build.

## Measured costs

Numbers below are from `examples/casc_bench.rs`, run on the dev workstation against `/syncthing/World of Warcraft` with `target/release` and the resolution sqlite warm:

```
probe                                              CASC extract  disk cache  mem cache
---------------------------------------------------------------------------------------
UI-Panel-Button-Up      (128×32, includes init)      302.17 ms      1.16 ms    1.7 µs
UI-DialogBox-Background (64×64)                        1.84 ms    701.8 µs    2.4 µs
INV_Misc_QuestionMark   (64×64)                       11.15 ms      1.14 ms    1.9 µs
UI-Panel-MinimizeButton (32×32)                        1.62 ms    904.2 µs    1.9 µs
WorldMap/GEAR_64GREY    (64×64)                       13.40 ms      1.15 ms    2.3 µs
UI-Tooltip-Background   (64×64)                       13.23 ms      1.25 ms    2.4 µs
UI-Backpack-EmptySlot   (64×64)                       18.81 ms      1.01 ms    1.1 µs
UI-StatusBar            (64×8)                         8.77 ms      1.03 ms    1.8 µs
---------------------------------------------------------------------------------------
average (incl. init)                                   46.37 ms      1.04 ms    1.9 µs
average (excl. first)                                   9.83 ms      1.06 ms    2.0 µs
```

Breakdown by layer:

- **CASC init** (one-time per process, first extract only): ~300 ms. Loads 45 `.idx` files, opens 190 GB of `.data` archives via mmap, opens the resolution sqlite. Independent of the asset being requested.
- **CASC per-file extract** (steady state): ~10 ms typical. Sqlite SELECT (sub-µs) + index lookup + archive read + BLTE decompress + disk write.
- **BLP cache hit** (file present, fresh `TextureManager`): ~1 ms. Listfile lookup + `out_path.exists()` + read + image-blp decode.
- **In-memory hit**: ~2 µs. HashMap lookup.

The BLP byte cache is roughly **10× faster** than re-extracting steady-state, and ~300× faster than the very first asset of the process. Once populated (`~/.cache/wow-ui-sim/casc-extract/`, currently 672 MB / 1747 files), the simulator can avoid CASC init entirely for any startup that touches only assets already on disk.

## Blizzard UI source cache

`data/blizzard-ui-files/<profile>.txt` manifests list the Blizzard UI source files for each supported profile. The retail manifest mirrors the complete Gethe `live` AddOns tree, including `Classic/` and `Mainline/` family variants; this preserves source inventory even though retail runtime `[Family]` substitution selects `Mainline`. `wow-cli casc sync-blizzard-ui` resolves each active-profile manifest entry as `Interface/AddOns/<entry>`, extracts it from the active CASC product into `~/.cache/wow-ui-sim/blizzard-ui/<profile>/AddOns`, and preserves Blizzard's original addon/file casing on disk. The bundled limited listfile is generated from the union of those profile manifests plus tracked `data/listfile-overrides.csv` rows for paths missing from the upstream community listfile or requiring canonical display casing.

Retail/Classic sync tries local install archives first through `asset-resolver`. If active root/encoding metadata resolves an FDID but the local streaming install lacks that archive chunk, it downloads the authoritative CASC blob by encoding key through `Osso/casc-extract`.

PTR 12.1.5 is intentionally separate: no matching local `wowxptr` install exists. Its committed `data/blizzard-ui-builds/ptr.json` maps the exact `wowxptr` build/config and Gethe `ptr2` revision to CDN archive ranges. Sync validates each range's BLTE encoding key and decoded content key before writing it. This is a build-specific source-acquisition route, not generic TVFS support; it neither labels local `wowt` content as PTR nor falls back to Gethe file bytes.

The GUI startup path uses the cache only. If the completion marker is missing, startup syncs the manifest and rechecks the cache. PTR additionally requires exact pinned provenance, so a stale cache with required files is rejected. The old `Interface/BlizzardUI` symlink and `vendor/wow-ui-source` checkout are not part of runtime discovery.

### Bundled limited listfile

`tools/gen_limited_listfile.py` stores ordinary community rows with normalized lowercase paths. `data/listfile-overrides.csv` is authoritative: an override replaces the source display path for both normalized path and FDID resolution while preserving slash-normalized canonical casing. Generated rows sort by normalized path, keeping output deterministic when display casing differs.

### Retail-only isolation test

Run `scripts/test-retail-casc-isolation.py` to verify the retail manifest without allowing access to any sibling WoW flavor. The script builds `wow-cli`, discovers every `_..._` directory beside `_retail_`, masks the non-retail directories with empty Bubblewrap mounts, and syncs into a fresh writable HOME/cache. Failed runs preserve the cache and `retail-casc-isolation.log`, then print manifest entries that are absent or fail profile-specific content validation.

```bash
./scripts/test-retail-casc-isolation.py
./scripts/test-retail-casc-isolation.py --skip-build --keep-cache
```

To add a genuinely required retail file:

1. Confirm the path is present in the Gethe `live` AddOns tree and referenced by a retail `_Mainline.toc` or its transitive dependency. Do not add a path merely because it appears in Classic, PTR, or another profile manifest.
2. Confirm the path or FDID exists in the active retail CASC product. A file absent from retail CASC is a stale manifest candidate, not a file to fabricate locally.
3. Add the addon-relative path to `data/blizzard-ui-files/retail.txt` using Blizzard's original casing.
4. If the community listfile does not map the path yet, add a verified `FDID;interface/addons/...` row to `data/listfile-overrides.csv`.
5. Regenerate the bundled subset with `python3 tools/gen_limited_listfile.py`.
6. Run the manifest mapping test and the retail-only isolation script again.

When removing an unavailable entry, first prove it is not reachable from the retail TOC/dependency graph. Extraction failure alone is insufficient because a valid retail file may require an updated FDID or listfile override.

## Failure modes

- **Resolution sqlite missing or stale**: `asset-resolver` logs the open failure, extracts current `root.bin` / `encoding.bin`, rebuilds `resolution.sqlite`, then reopens it. If rebuild fails, CASC lookup fails with the underlying read/build error.
- **Wrong cache override**: an invalid `$ASSET_RESOLVER_CACHE_DIR` can put generated CASC metadata in an unexpected place or one without write permission. Unset it to use `~/.cache/asset-resolver`.
- **Case-variant duplicates**: `out_path` uses the case returned by the listfile lookup. If a caller passes a non-canonical case to `lookup_path`, the BLP can land under `interface/...` or `INTERFACE/...` instead of `Interface/...`. The cache currently has 4 such stragglers out of 1747 files; harmless but wasteful.
- **Listfile/CASC drift**: when the live archives have GC'd a file the encoding manifest still references, CASC extract fails with "missing resolution entry" or "missing archive location". The loader writes a `.missing` sentinel and falls back to `<wow>/_retail_/BlizzardInterfaceArt/` for the next request.
- **Partial Blizzard UI source sync**: the source cache is ignored unless `.wow-ui-sim-blizzard-ui-complete` exists. Re-run `wow-cli casc sync-blizzard-ui` after fixing CASC/listfile issues. Missing Blizzard UI source files are not filled from repo mirrors; if local archives lack chunks that the active product metadata still references, sync may fetch those chunks from Blizzard CDN and cache the archive indexes under `~/.cache/casc-extract/`.

## Sources

- [`docs/specs/casc-loading.md`](../../specs/casc-loading.md) — behavioral contract for CASC loading
- `src/texture/resolve.rs` — `casc_enabled`, `try_casc_resolve`, `casc_extract_dir`
- `src/blizzard_ui_sync.rs` — Blizzard UI source manifest sync into the user cache
- `scripts/test-retail-casc-isolation.py` — Bubblewrap test that masks non-retail WoW flavors
- `Osso/casc-extract` — public helper crate used for missing CASC CDN chunks by encoding key
- `data/blizzard-ui-files/` — per-profile manifests of Blizzard UI source files extractable from CASC
- `data/listfile-overrides.csv` — tracked authoritative path→FDID rows for missing or canonically cased paths
- `src/texture.rs` — `TextureManager::load` and the in-memory cache
- `asset-resolver/src/casc_resolver.rs` — `init_casc`, resolution metadata refresh, `extract_fdid_to_path`, `Installation::initialize`
- `asset-resolver/src/casc_cache.rs` — `CascResolutionCache::open`, freshness check, `build_resolution_cache`, `resolve_fdid`
- `asset-resolver/src/paths.rs` — source-data resolution, cache-root resolution, and `remap_to_shared_data_path`
- `examples/casc_bench.rs` — reproduction harness for the timing table

## See Also

- [[texture-atlas]] — `TextureManager` structure, BLP/PNG/WebP support, atlas database
- [[casc-root-v2-parsing-missing-textures]] — root v2 misparse that silently dropped 89% of fdids from the resolution cache; rebuild steps after parser changes
- [[rendering-pipeline]] — font system and downstream consumers of CASC assets
