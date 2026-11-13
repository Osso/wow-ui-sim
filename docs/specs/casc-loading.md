# CASC asset loading

The simulator reads textures and fonts directly from a live WoW install via the [`asset-resolver`](/home/osso/Projects/world-of-osso/asset-resolver) crate (a thin wrapper over `cascette-rs` + the community listfile). The install location is discovered through `asset_resolver::wow_install_path()`: `WOW_INSTALL_PATH` env, then `WOW_DATA_PATH` env, then a built-in candidate list (Linux/Wine/Lutris/WSL/macOS); the default is `/syncthing/World of Warcraft`. This spec describes the contract the loader must satisfy. For implementation details see the wiki.

## What it must do

### Feature gating

- [ ] `casc` Cargo feature is on by default and pulls in `asset-resolver`.
- [ ] Building with `--no-default-features` (or `--features ""`) compiles cleanly with no CASC symbols and the loader skips the CASC tier entirely.
- [ ] `WOW_SIM_CASC=0` at runtime disables the CASC tier without rebuilding (loader behaves as if the feature were off).
- [ ] When `asset_resolver::wow_install_path()` returns `None` (no env override and no candidate path on disk), the loader does not panic — it just reports a CASC miss and continues.
- [ ] `WOW_INSTALL_PATH` (install root) and `WOW_DATA_PATH` (`Data/` dir) override discovery; both are validated by checking that `<root>/Data/data` exists before they win. A stored per-product path is the next source; see [install path settings](wow-install-path-settings.md).
- [ ] wow-ui-sim creates `asset_resolver::CascListfileResolver` with an explicit cache/shared-data location; runtime loading does not set or require `GAME_ENGINE_SHARED_ROOT`.

### Installed product identity

- [x] Non-pinned UI-source provenance and runtime CASC resolution read the selected product from the installation's `.product.db` through one shared parser.
- [x] Select `activeBuildKey` (field 14), not completed or incomplete build keys; retain `currentVersionStr` and optional `activeInstallKey` for provenance. This is simulator selection policy, not a guarantee of native launcher behavior or complete local content.
- [x] An absent `.build.info` product row does not prevent reading the selected product's available CASC content. No installation metadata is rewritten, and no other product is selected as a fallback.
- [ ] Missing, malformed, duplicate, or invalid selected-product metadata fails explicitly. A missing `.product.db` is not rescued with `.build.info`.
- [ ] PTR's pinned UI-source provenance remains independent of installed-product metadata.

### Blizzard UI locale selection

- [ ] Local FDID, CDN FDID, and path-based Blizzard UI reads explicitly select `enUS` root variants with unconstrained content flags (`ContentFlags::NONE`); no cross-locale fallback.
- [ ] Preserve root version and full-width content flags, including high bits, when parsing/selecting variants; do not flatten root records to a last-variant map.
- [ ] Record `locale=enUS` and `content_filter=none` in UI-cache provenance. Sync invalidates locale-blind caches even when build identity and manifest match, before reusing files.

### Blizzard UI source bytes

- [x] Lua cache usability requires a readable file, not UTF-8 encoding. Sync must preserve authenticated source bytes without normalization or transcoding.
- [x] Profile-specific source content checks match byte sequences without requiring the whole Lua file to decode as UTF-8.
- [x] The runtime loads Lua source bytes into rilua; string literals retain their original bytes through source compilation and bytecode replay.
- [x] XML, TOC and XSD cache validation still requires UTF-8. TOC and XML runtime text parsing is unchanged.

### Unit-test startup cache isolation

- [x] The `wow-sim` unit-test executable, including its reload-test child processes, must not automatically synchronize Blizzard UI sources. A missing or stale cache fails startup and identifies the expected Retail/PTR pin from the committed content index.
- [x] Tests require an explicitly prepared matching cache. Invalid-cache startup must leave cache bytes unchanged. Normal non-test `wow-sim` startup retains automatic synchronization.

### Source sync transport

- [ ] A sync operation reuses one CDN session and archive index set across file downloads instead of re-querying product metadata per file, then releases them before GUI startup.
- [ ] Initialization checks every archive index in the selected CDN configuration, reusing individual cached index files; an interrupted index download is not accepted as a complete set.
- [ ] Encoding-key downloads retry typed transport failures at most three times with exponential backoff and jitter. HTTP status and decode failures remain explicit failures.
- [x] Linux Zenity startup errors render as plain text, including literal `<profile>` paths.

### Forever mapped, non-resident UI sources

- [ ] For Forever only, sync selects Gethe bytes only when the active enUS root maps the manifest FDID to a CKey, the encoding table maps that CKey to an EKey, and the selected local indexes contain no entry for that EKey. Resident entries keep the existing CASC route; absent root/encoding mappings keep the existing failure path. Read/decode errors for resident data do not select Gethe.
- [ ] The committed `data/blizzard-ui-builds/wowforever.json` pins the exact Gethe revision and build version, following the existing pinned source convention. Its version must equal the installed profile build. Read blobs from that immutable revision in the existing `wow-ui-sim/wow-ui-source-git/forever` cache, not from a moving branch or working-tree export. Read that revision's `version.txt` and require the same version. Sync does not fetch or change the source checkout.
- [ ] Hash the raw Gethe bytes with MD5 and require equality with the root/encoding CKey before any destination write. Do not normalize newlines or text. A mismatch fails with the manifest path, FDID, expected CKey and actual Gethe MD5. Missing pinned blobs and mismatched versions fail explicitly.
- [ ] Cache provenance identifies the pinned revision/version and records every substituted file as a `gethe_file=` JSON record with `path`, `fdid`, `ckey`, `gethe_revision`, and `source: "gethe"`. Reused substituted files are reverified and recorded on sync; resident files produce no Gethe record. Completion is published only after all entries succeed.
- [ ] This rule exists because the selected Forever streaming installation has authenticated root/encoding mappings for UI content absent from local indexes (observed FDID 8379893). It is not a general fallback or CDN repair. Retire it when a supported local acquisition path reliably supplies the full selected UI manifest; remove the pin and route together and recapture the stock startup baseline.

Static implementation and fixture tests are staged; execution remains pending. The fixture selector is `blizzard_ui_sync::gethe_missing::tests::`.

### GUI resolution-cache preparation

- [ ] Before `run_iced_ui`, an enabled CASC installation prepares the selected product/build resolution cache at `ASSET_RESOLVER_CACHE_DIR` when set, otherwise the resolver default cache root. `WOW_PRODUCT` overrides the client-profile product selection.
- [x] A fresh resolution cache is built before GUI rendering; a warm cache is reused without rebuilding. Resolution-cache preparation failure stops GUI startup with an explicit error.
- [ ] The community lookup catalog initializes before rendering too; catalog-load errors retain the resolver's explicit diagnostic. This moves initialization to startup, not out of total startup time.
- [x] `WOW_SIM_CASC=0`, a build without `casc`, or an undiscoverable install does not prepare a cache and preserves existing startup behavior.
- [ ] Preparation does not initialize the lazy runtime archive reader or extract textures before the first render.
- [x] The required library regression builds a test-owned local install and checks cold creation, warm reuse, concrete FDID mappings, disabled CASC, missing bootstrap index entries, and corrupt indexed bootstrap content; it does not depend on the mutable installed game.
- [ ] The live-install smoke test is gated by `optional-tests`. It reports the selected product and install, skips only when no install is discoverable, and fails when cache preparation fails.

### Texture resolution

- [ ] WoW paths with backslashes (`Interface\Buttons\UI-Panel-Button-Up`) are normalised to forward slashes and extension-stripped before lookup.
- [ ] Lookup probes the bundled simulator listfile first, then the external/community listfile, under each of `<path>`, `<path>.blp`, `<path>.BLP`, `<path>.tga`, `<path>.TGA`, `<path>.ttf`, `<path>.TTF`, `<path>.otf`, `<path>.OTF` and stops at the first hit.
- [ ] On hit, `asset_resolver::ensure_cached(fdid, out_path)` materialises the BLP under `~/.cache/wow-ui-sim/casc-extract/<listfile/path>` (forward-slash form).
- [ ] A second resolution of the same path returns the cached file without re-extracting from CASC.
- [ ] Addon-shipped textures under `Interface/AddOns/<Addon>/...` resolve before CASC is consulted.
- [ ] Listfile lookup is case-insensitive (`UI-Panel-Button-Up.blp` and `ui-panel-button-up.blp` both resolve to the same fileDataID).
- [ ] The generated bundled subset keeps ordinary community rows normalized lowercase; `data/listfile-overrides.csv` is authoritative for its normalized path and FDID, preserving slash-normalized canonical casing; generated rows sort by normalized path.
- [ ] When CASC misses (e.g. live archives have been GC'd by a partial patch but the encoding manifest still references the entry), a third-tier fallback resolves the path under `<install-root>/_retail_/BlizzardInterfaceArt/Interface/...` (install root from `asset_resolver::wow_install_path()`) with the same case-insensitive matching as the addon tier. CPU miss reuse, filesystem freshness and root/reload invalidation are owned by the [source-cache contract](texture-source-resolution-cache.md).

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

- [Texture source cache](../wiki/systems/texture-source-resolution-cache.md) — shared setter/renderer resolver and CPU-negative freshness.

- [Blizzard UI patch/sync workflow](../updating-blizzard-ui-to-a-new-patch.md) — selected-build acquisition and baseline gates.

- → `docs/wiki/systems/casc-asset-cache.md` — three cache layers, lookup flow, measured per-layer costs
- → `docs/wiki/systems/texture-atlas.md` — TextureManager structure, path resolution, format support
- → `docs/rendering-pipeline.md` — Font System section
- → `docs/texture-atlas-system.md` — full texture/atlas walkthrough

## Implementation inventory

- `src/blizzard_ui_sync.rs` — UI extraction routes and locale/content cache provenance.
- `src/blizzard_ui_sync/locale.rs` — explicit enUS root selection for CDN FDID and path reads; local FDID extraction retains asset-resolver's TACT-aware decoder.
- Asset-resolver is pinned to published `a626003dd4d3dbd1f157181f017694c648c84ce8` for its enUS root-block filter and resolution-schema migration. Its transitive cascette revision is `027a1ac34c81b4ec23a0bfa36ea298f6642abe19`; direct simulator cascette pins remain unchanged. That dependency also carries keyring loading and missing-key chunk zero-fill behavior; no full encrypted-content acceptance is claimed. Unrelated lockfile package identities and dependency edges are preserved.

- `src/asset_resolver_config.rs` — GUI resolution-cache and community-catalog preparation using the active install and product.
- `src/bin/wow_sim/gui_commands.rs` — GUI startup preparation before `run_iced_ui`
- `src/texture/resolve.rs` — CASC tier for textures (`casc_enabled`, `casc_extract_dir`, `try_casc_resolve`)
- `src/render/font.rs` — CASC tier for fonts (`casc_enabled`, `try_casc_font_bytes`) and known core-font encoding-key fallback for CASC cache misses
- `examples/casc_smoke.rs` — verification harness for textures + fonts
- `Cargo.toml` — `casc` feature gate (`dep:asset-resolver`, `dep:cascette-client-storage`, `dep:cascette-crypto`), default-on
- `Cargo.lock` — pins transitive `binrw`/`binrw_derive` 0.15.2 for the CASC dependency graph; the update fixes Rust E0365 private macro re-export warnings without changing the pinned CASC source revisions

## Tests asserting this spec

- `src/blizzard_ui_sync/locale/tests.rs` — five selector cases: enUS before last ptPT, absent-enUS rejection, path/FDID agreement, explicit content filtering, V4/high-bit preservation.
- `src/blizzard_ui_sync/tests.rs::locale_blind_provenance_invalidates_cached_ptpt_before_sync` — same-build stale-locale cache invalidation; focused integrated PASS, broader gates open.

- `src/blizzard_ui_sync/tests.rs::build_identity_reads_forever_when_build_info_omits_the_product` — file-backed missing-row regression without metadata writes.
- `src/blizzard_ui_sync/tests.rs::build_identity_uses_selected_product_not_stale_build_info` — selected-product identity, optional install key, and absent-product failure.
- Pinned cascette `758d8dbf` `product_db` — 8/8, zero warnings; parser selection/validation proof against the actual install.
- `src/asset_resolver_config.rs::tests::gui_resolution_cache_preparation` — isolated synthetic cold build, warm reuse, two FDID mappings, disabled CASC, missing index entry, corrupt content and invalid install; targeted Era test passes.
- `src/asset_resolver_config.rs::tests::gui_resolution_cache_preparation_ignores_ambient_install` — verifies that an unindexed ambient install does not affect the synthetic regression; targeted Era test passes.
- `src/asset_resolver_config.rs::tests::gui_resolution_cache_preparation_uses_live_install` — `optional-tests`-gated live smoke; its Era run reported the missing selected encoding index as a test failure. A valid-install pass remains unverified.
- `tests/casc_loading.rs` — integration tests for the CASC tier (skip when `/syncthing/World of Warcraft/Data` is missing or `WOW_SIM_CASC=0`)
- `examples/casc_smoke.rs` — manual verifier for ad-hoc probes (not run by `cargo test`)
- `src/render/font.rs::tests::resolves_friz_quadrata` — asserts FRIZQT__ resolves with or without CASC
- `src/render/font.rs::tests::unknown_font_falls_back_to_default` — asserts unknown path → FRIZQT family
- `src/render/font.rs::tests::resolves_case_insensitive` — asserts WoW-path normalisation
- `/tmp/wow-character-bug/product-db-cold-gui/{setup.json,result.json,observer.json,stderr,metadata-before.json,metadata-after.json,character-backpack.png}` — default-install-selected GUI: no install/data/product override; fresh isolated resolver/catalog state; 1,460 linked UI extracts; metadata non-mutation and inspected panel readiness. This is not fully cold all-assets proof.
- `/tmp/wow-character-bug/product-db-error-dialog/{result.json,observer.json,stderr,error-dialog.png,metadata-bracket.json}` — actual Linux Zenity missing-`.product.db` dialog; inspected literal `<profile>` rendering, expected timeout cleanup, and metadata unchanged from before the default GUI through the dialog/parser checks

## Known gaps (current cycle)

### Locale checkpoint — October 15, 2026

Reported handoff evidence, not rerun for this docs task:

| Scope | Proof | Remaining boundary |
|---|---|---|
| Actual selected-root metadata, FDID `5612369` | 12 variants; last is ptPT | Locale-blind selection chose the wrong variant |
| Matched native local enUS probe | PASS: indexed, 138 bytes, enUS content-hash match, `hasenUS=true`, `hasptPT=false` | Earlier missing result used the wrong variant; not evidence of absent local enUS content |
| Candidate selector / historical pinned resolver | 5/5 GREEN / 0/5 RED | Historical RED retained; not a result for the updated dependency |
| Integrated source136 | Selector and stale-locale provenance tests executed PASS; [central receipt](../baselines/README.md#verified-scoped-receipts--2026-10-15) | Full simulator/native/cross-profile and encrypted-archive gates remain open |
| Normal binaries | Compiled `9a6ea924e` plus dirty 70374 manifest/listfile/BuildInfo metadata | Compilation is not full Rust-test proof |
| Supported corrected-enUS sync | 4,434 extracted, 0 already present | Selected Forever build only |
| Exact selected-root content-key validation | 4,434/4,434 matched; no missing/mismatched files | Old cache: 4,433 matched, one wrong-ptPT LoadLocale file; not all encrypted content/all profiles |
| Corrected-cache headless startup | [Actual failing baseline](../baselines/README.md#forever-lua-errors) captured | Startup FAIL; independent saved locale/runtime receipt verification PASS |

Saved receipt root: `/home/osso/.local/state/wow-ui-sim/verification/widget-discrete-progress-current/feedback-enus-forever-bins-retry1-20261015T100055523974Z`; `source-basis.json` binds the dirty source, `corrected-enus-sync/` and `corrected-enus-content-green/` record acquisition/validation, and `corrected-70374-lua-errors/baseline-summary.json` records startup failure.

- [x] Focused integrated selector/cache-invalidation model gate passed at runtime source136; counts live in the central receipt.
- [ ] Full simulator, default-check, native/cross-profile and encrypted-content acceptance remain open. Acquisition and failing startup capture are bounded separate proofs, not clean-startup acceptance.

- [ ] No assertion that the texture `out_path.exists()` short-circuit in `try_casc_resolve` is faster than a cold extract; this is distinct from verified resolution-cache warm reuse.
- [ ] `asset_resolver::lookup_path` is not backslash-tolerant; the loader normalises but consumers calling the resolver directly hit `None` on `Fonts\\FRIZQT__.TTF`. Consider lifting normalisation into `asset-resolver` upstream.
- [ ] The inspected error-dialog proof covers Linux Zenity only, not KDE or Windows dialogs.

## Out of scope

- Full community listfile maintenance — upstream concern of `asset-resolver`; the simulator ships a generated limited subset in `data/wow-ui-sim-listfile.csv` for first-run cache population.
- 3D model assets (M2/WMO/ADT) — see `docs/wiki/index.md` "Intentional Gaps"; the simulator is 2D only.
- Audio assets — gated by the separate `sound` feature; CASC loading for sounds is not implemented.
- Live filesystem watching of the WoW install — re-launching the simulator picks up new patches.
- Bundled WebP/PNG mirror of textures — replaced by CASC; reintroducing it would defeat the whole migration.
