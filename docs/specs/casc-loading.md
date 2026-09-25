# CASC asset loading

The simulator reads textures and fonts directly from a live WoW install via the [`asset-resolver`](/home/osso/Projects/world-of-osso/asset-resolver) crate (a thin wrapper over `cascette-rs` + the community listfile). The install location is discovered through `asset_resolver::wow_install_path()`: `WOW_INSTALL_PATH` env, then `WOW_DATA_PATH` env, then a built-in candidate list (Linux/Wine/Lutris/WSL/macOS); the default is `/syncthing/World of Warcraft`. This spec describes the contract the loader must satisfy. For implementation details see the wiki.

## What it must do

### Feature gating

- [ ] `casc` Cargo feature is on by default and pulls in `asset-resolver`.
- [ ] Building with `--no-default-features` (or `--features ""`) compiles cleanly with no CASC symbols and the loader skips the CASC tier entirely.
- [ ] `WOW_SIM_CASC=0` at runtime disables the CASC tier without rebuilding (loader behaves as if the feature were off).
- [ ] When `asset_resolver::wow_install_path()` returns `None` (no env override and no candidate path on disk), the loader does not panic — it just reports a CASC miss and continues.
- [ ] `WOW_INSTALL_PATH` (install root) and `WOW_DATA_PATH` (`Data/` dir) override discovery; both are validated by checking that `<root>/Data/data` exists before they win.
- [ ] wow-ui-sim creates `asset_resolver::CascListfileResolver` with an explicit cache/shared-data location; runtime loading does not set or require `GAME_ENGINE_SHARED_ROOT`.

### Installed product identity

- [x] Non-pinned UI-source provenance and runtime CASC resolution read the selected product from the installation's `.product.db` through one shared parser.
- [x] Select `activeBuildKey` (field 14), not completed or incomplete build keys; retain `currentVersionStr` and optional `activeInstallKey` for provenance. This is simulator selection policy, not a guarantee of native launcher behavior or complete local content.
- [x] An absent `.build.info` product row does not prevent reading the selected product's available CASC content. No installation metadata is rewritten, and no other product is selected as a fallback.
- [ ] Missing, malformed, duplicate, or invalid selected-product metadata fails explicitly. A missing `.product.db` is not rescued with `.build.info`.
- [ ] PTR's pinned UI-source provenance remains independent of installed-product metadata.

### Source sync transport

- [ ] A sync operation reuses one CDN session and archive index set across file downloads instead of re-querying product metadata per file, then releases them before GUI startup.
- [ ] Initialization checks every archive index in the selected CDN configuration, reusing individual cached index files; an interrupted index download is not accepted as a complete set.
- [ ] Encoding-key downloads retry typed transport failures at most three times with exponential backoff and jitter. HTTP status and decode failures remain explicit failures.
- [x] Linux Zenity startup errors render as plain text, including literal `<profile>` paths.

### GUI resolution-cache preparation

- [ ] Before `run_iced_ui`, an enabled CASC installation prepares the selected product/build resolution cache at `ASSET_RESOLVER_CACHE_DIR` when set, otherwise the resolver default cache root. `WOW_PRODUCT` overrides the client-profile product selection.
- [x] A fresh resolution cache is built before GUI rendering; a warm cache is reused without rebuilding. Resolution-cache preparation failure stops GUI startup with an explicit error.
- [ ] The community lookup catalog initializes before rendering too; catalog-load errors retain the resolver's explicit diagnostic. This moves initialization to startup, not out of total startup time.
- [x] `WOW_SIM_CASC=0`, a build without `casc`, or an undiscoverable install does not prepare a cache and preserves existing startup behavior.
- [ ] Preparation does not initialize the lazy runtime archive reader or extract textures before the first render.

### Texture resolution

- [ ] WoW paths with backslashes (`Interface\Buttons\UI-Panel-Button-Up`) are normalised to forward slashes and extension-stripped before lookup.
- [ ] Lookup probes the bundled simulator listfile first, then the external/community listfile, under each of `<path>`, `<path>.blp`, `<path>.BLP`, `<path>.tga`, `<path>.TGA`, `<path>.ttf`, `<path>.TTF`, `<path>.otf`, `<path>.OTF` and stops at the first hit.
- [ ] On hit, `asset_resolver::ensure_cached(fdid, out_path)` materialises the BLP under `~/.cache/wow-ui-sim/casc-extract/<listfile/path>` (forward-slash form).
- [ ] A second resolution of the same path returns the cached file without re-extracting from CASC.
- [ ] Addon-shipped textures under `Interface/AddOns/<Addon>/...` resolve before CASC is consulted.
- [ ] Listfile lookup is case-insensitive (`UI-Panel-Button-Up.blp` and `ui-panel-button-up.blp` both resolve to the same fileDataID).
- [ ] The generated bundled subset keeps ordinary community rows normalized lowercase; `data/listfile-overrides.csv` is authoritative for its normalized path and FDID, preserving slash-normalized canonical casing; generated rows sort by normalized path.
- [ ] When CASC misses (e.g. live archives have been GC'd by a partial patch but the encoding manifest still references the entry), a third-tier fallback resolves the path under `<install-root>/_retail_/BlizzardInterfaceArt/Interface/...` (install root from `asset_resolver::wow_install_path()`) with the same case-insensitive matching as the addon tier. Repeated misses on the same path use the existing `.missing` sentinel to short-circuit CASC fast (~40ms warm path).

### Font resolution

- [ ] `WowFontSystem::new()` takes no arguments and returns a font system populated with at least the FRIZQT__ family.
- [ ] When CASC is available, `Fonts\FRIZQT__.TTF`, `Fonts\ARIALN.TTF`, and `Fonts\frizqt___cyr.ttf` all resolve to a real CASC-loaded face.
- [ ] When CASC path/FDID resolution misses a known core WoW font, the font loader falls back to the known CASC encoding key instead of shipping embedded font bytes.
- [ ] When CASC is unavailable (feature off, `WOW_SIM_CASC=0`, or the WoW install missing), `Fonts\FRIZQT__.TTF` may fall back to a system font family so text remains shapeable. Tests that require real WoW font metrics must require CASC.
- [ ] An unknown font path (`Fonts\NONEXISTENT.TTF`) falls back to the FRIZQT__ family rather than returning `None`.

### Smoke verification

- [x] Three baseline textures (`Interface\Buttons\UI-Panel-Button-Up`, `Interface\DialogFrame\UI-DialogBox-Background`, `Interface\Icons\INV_Misc_QuestionMark`) resolve end-to-end through `TextureManager.load` with the expected dimensions (`tests/casc_loading.rs::casc_resolves_baseline_textures`).
- [x] `Fonts\FRIZQT__.TTF`, `Fonts\ARIALN.TTF`, and `Fonts\FRIZQT___CYR.TTF` resolve through `WowFontSystem` to distinct real WoW faces, including the known encoding-key fallback when raw FDID resolution misses (`tests/casc_loading.rs::casc_resolves_baseline_fonts`).

## How it works

- → `docs/wiki/systems/casc-asset-cache.md` — three cache layers, lookup flow, measured per-layer costs
- → `docs/wiki/systems/texture-atlas.md` — TextureManager structure, path resolution, format support
- → `docs/rendering-pipeline.md` — Font System section
- → `docs/texture-atlas-system.md` — full texture/atlas walkthrough

## Implementation inventory

- `src/asset_resolver_config.rs` — GUI resolution-cache and community-catalog preparation using the active install and product.
- `src/bin/wow_sim/gui_commands.rs` — GUI startup preparation before `run_iced_ui`
- `src/texture/resolve.rs` — CASC tier for textures (`casc_enabled`, `casc_extract_dir`, `try_casc_resolve`)
- `src/render/font.rs` — CASC tier for fonts (`casc_enabled`, `try_casc_font_bytes`) and known core-font encoding-key fallback for CASC cache misses
- `examples/casc_smoke.rs` — verification harness for textures + fonts
- `Cargo.toml` — `casc` feature gate (`dep:asset-resolver`, `dep:cascette-client-storage`, `dep:cascette-crypto`), default-on
- `Cargo.lock` — pins transitive `binrw`/`binrw_derive` 0.15.2 for the CASC dependency graph; the update fixes Rust E0365 private macro re-export warnings without changing the pinned CASC source revisions

## Tests asserting this spec

- `src/blizzard_ui_sync/tests.rs::build_identity_reads_forever_when_build_info_omits_the_product` — file-backed missing-row regression without metadata writes.
- `src/blizzard_ui_sync/tests.rs::build_identity_uses_selected_product_not_stale_build_info` — selected-product identity, optional install key, and absent-product failure.
- Pinned cascette `758d8dbf` `product_db` — 8/8, zero warnings; parser selection/validation proof against the actual install.
- `src/asset_resolver_config.rs::tests::gui_resolution_cache_preparation` — 1/1 at `4eb32befa`: isolated-process cold build, warm reuse, disabled, and error boundaries; cold/warm skip without an install
- `tests/casc_loading.rs` — integration tests for the CASC tier (skip when `/syncthing/World of Warcraft/Data` is missing or `WOW_SIM_CASC=0`)
- `examples/casc_smoke.rs` — manual verifier for ad-hoc probes (not run by `cargo test`)
- `src/render/font.rs::tests::resolves_friz_quadrata` — asserts FRIZQT__ resolves with or without CASC
- `src/render/font.rs::tests::unknown_font_falls_back_to_default` — asserts unknown path → FRIZQT family
- `src/render/font.rs::tests::resolves_case_insensitive` — asserts WoW-path normalisation
- `/tmp/wow-character-bug/product-db-error-dialog/{result.json,observer.json,stderr,error-dialog.png,metadata-bracket.json}` — actual Linux Zenity missing-`.product.db` dialog; inspected literal `<profile>` rendering, expected timeout cleanup, and metadata unchanged from before the default GUI through the dialog/parser checks

## Known gaps (current cycle)

- [ ] No assertion that the texture `out_path.exists()` short-circuit in `try_casc_resolve` is faster than a cold extract; this is distinct from verified resolution-cache warm reuse.
- [ ] `asset_resolver::lookup_path` is not backslash-tolerant; the loader normalises but consumers calling the resolver directly hit `None` on `Fonts\\FRIZQT__.TTF`. Consider lifting normalisation into `asset-resolver` upstream.
- [ ] The inspected error-dialog proof covers Linux Zenity only, not KDE or Windows dialogs.

## Out of scope

- Full community listfile maintenance — upstream concern of `asset-resolver`; the simulator ships a generated limited subset in `data/wow-ui-sim-listfile.csv` for first-run cache population.
- 3D model assets (M2/WMO/ADT) — see `docs/wiki/index.md` "Intentional Gaps"; the simulator is 2D only.
- Audio assets — gated by the separate `sound` feature; CASC loading for sounds is not implemented.
- Live filesystem watching of the WoW install — re-launching the simulator picks up new patches.
- Bundled WebP/PNG mirror of textures — replaced by CASC; reintroducing it would defeat the whole migration.
