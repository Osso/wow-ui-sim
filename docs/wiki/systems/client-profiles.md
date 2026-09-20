# Client Profiles

The simulator targets seven WoW client profiles — retail, PTR, wrath (3.3.5a), mists (5.4 / MoP Classic), era (1.x / Vanilla), anniversary, and WoW Forever — selected at compile time via mutually-exclusive cargo features. Each profile uses a profile-scoped Blizzard UI cache and routes the loader through profile-aware suffix and gametype filters. Mainline API surface differences are gated by cumulative retail API epoch features so `client-retail` and `client-ptr` can point at different patch-note lifetimes without making PTR itself the API truth. Retail-family API availability is gated separately by cumulative patch epoch features so a profile/channel points at an API epoch instead of hard-coding every API delta to `client-ptr`.

## Active profile selection

`src/client_profile.rs` defines `enum ClientProfile { Retail, Ptr, Wrath, Mists, Era, Anniversary, WowForever }` and a single `pub const ACTIVE: ClientProfile` resolved by cfg-blocks against the enabled profile marker. Retail uses the internal `profile-retail` feature; public `client-retail` remains the retail 12.1.0 bundle, while `client-ptr` selects the cumulative 12.1.5 epoch. A `compile_error!` block enforces exactly one client profile marker.

Feature ↔ profile ↔ vendor source ↔ TOC suffix:

| Feature              | Profile     | Cache subdir | Default API epoch | Primary TOC suffix |
|----------------------|-------------|--------------|-------------------|--------------------|
| `client-retail`      | Retail      | `retail`     | `retail-12-1-0` (`120100`) | `_Mainline` |
| `profile-retail`     | Retail      | `retail`     | selected by enabled epoch | `_Mainline` |
| `client-ptr`         | Ptr         | `ptr`        | `retail-12-1-5` (`120105`) | `_Mainline` |
| `client-wrath`       | Wrath       | `wrath`      | `38001`          | `_Wrath`           |
| `client-mists`       | Mists       | `mists`      | `50504`          | `_Mists`           |
| `client-era`         | Era         | `era`        | `11507`          | `_Vanilla`         |
| `client-anniversary` | Anniversary | `anniversary`| `11507`          | `_Vanilla`         |
| `client-wowforever`  | WowForever  | `wowforever` | `16001`          | `_Camelot` → generic → `_Mainline` |

Retail-family epoch features are cumulative: `retail-12-0-5` includes `retail-12-0-0`, `retail-12-0-7` includes earlier 12.0 epochs, `retail-12-1-0` includes `retail-12-0-7`, and `retail-12-1-5` includes `retail-12-1-0`. `client-retail` remains at `retail-12-1-0`/`120100`; `client-ptr` selects `retail-12-1-5`/`120105`; `profile-retail` selects the retail cache without forcing an epoch. `RetailApiEpoch` and `ACTIVE_RETAIL_API_EPOCH` resolve the highest enabled cumulative retail epoch. API surfaces, CVars, enums, events, XML elements, and frame methods introduced by patch notes should gate on the epoch feature rather than on the channel feature. Strict removals follow the same rule unless source evidence proves a profile-specific retirement: retail 12.1 keeps `C_RecruitAFriend.IsEnabled`, while `src/ptr/strict_removals.lua` hides it from PTR addons after startup. Profile-specific runtime behavior gates use `profile-retail` so historical retail tests retain retail semantics. Channel/vendor behavior stays profile-gated: PTR CASC product `wowxptr`, `_ptr_` install paths, and `data/blizzard-ui-files/ptr.txt` remain `client-ptr` concerns. PTR 12.1.5 now has a completed 4,025-file source cache from immutable `wowxptr` CDN ranges pinned by build/config and Gethe-revision provenance. Only `client-ptr` reads the pinned `GetBuildInfo()` version/build (`12.1.5` / `69594`) and returns interface `120105`; non-PTR builds retain temporary interface `120100` with `retail-12-1-0`, otherwise `120007`. Its date and trailing slots remain temporary defaults. The current startup baseline has six pixel-rounding records, so startup and panel compatibility remain unproven.

Helper functions/constants in `src/client_profile.rs`:

- `ACTIVE_INTERFACE_VERSION` → active TOC/API interface selected by the profile's epoch
- `RETAIL_API_INTERFACE_VERSION` → retail-family interface selected by the highest enabled retail epoch
- `cache_subdir()` → profile cache directory name under `~/.cache/wow-ui-sim/blizzard-ui/`
- `interface_version()` → legacy profile default, not the preferred API-epoch selector
- `blizzard_ui_addons_dir()` → completed cache path for the active profile, or the profile-scoped default cache path
- `blizzard_ui_addons_dir_under(root)` — test fallback path anchored at `root`
- `blizzard_ui_framexml_toc()` → wrath-only `<cache>/FrameXML/FrameXML.toc`; retail/PTR/mists/era/anniversary collapsed FrameXML into `Blizzard_*` addons

## Retail API epochs

Mainline API additions/removals use cumulative Cargo features named after the patch epoch. Current chain:

```toml
retail-12-0-0 = []
retail-12-0-5 = ["retail-12-0-0"]
retail-12-0-7 = ["retail-12-0-5"]
retail-12-1-0 = ["retail-12-0-7"]
retail-12-1-5 = ["retail-12-1-0"]

profile-retail = []
client-retail = ["profile-retail", "retail-12-1-0"]
client-ptr = ["retail-12-1-5"]
```

Rules:

- `profile-retail` and the other profile markers select runtime profile: cache subdir, CASC product, install flavor, and TOC profile. `client-retail` is the public retail 12.1.0 bundle; it enables `profile-retail` and `retail-12-1-0`. `client-ptr` remains a separate PTR profile/cache and selects `retail-12-1-5`.
- `retail-*` features select mainline API epoch: C_* additions, globals/removals, events, CVars, XML elements, frame methods, and patch-note compatibility bootstraps. `RetailApiEpoch` / `ACTIVE_RETAIL_API_EPOCH` select the highest enabled cumulative epoch.
- Historical retail 12.0.0 audit tests use `cargo test --no-default-features --features profile-retail,retail-12-0-0`; profile-specific runtime behavior must gate on `profile-retail`, not `client-retail`.
- Patch features are cumulative. Gate additions with the smallest applicable epoch, for example `#[cfg(feature = "retail-12-1-5")]`; gate removals/lifetimes with the corresponding lower/upper epoch boundary.
- Do not gate patch-note API deltas on `client-ptr` by default. Retail and PTR epochs are cumulative while retaining separate profile/cache behavior; historical tests should select `profile-retail` with the required epoch explicitly. Add a profile-gated strict removal only when the cached channel source proves that channel-specific contract, as with PTR-only removal of `C_RecruitAFriend.IsEnabled`.

## Runtime Cache

Runtime Blizzard UI files live under the user cache:

```
~/.cache/wow-ui-sim/blizzard-ui/<profile>/AddOns
```

Populate it with `wow-cli casc sync-blizzard-ui` or the compatibility wrapper `scripts/setup-blizzard-ui.sh`. Do not use `Interface/BlizzardUI/` or repo-local `vendor/wow-ui-source-*` checkouts for runtime loading.

Each profile uses its own committed manifest in `data/blizzard-ui-files/<profile>.txt`. Forever uses `wowforever.txt`, generated from the canonical versioned source cache at `/home/osso/.cache/wow-ui-sim/wow-ui-source/forever`; `forever` is the source branch, while `wowforever` is the runtime cache profile. It must never be runtime-aliased to Anniversary or Era. PTR is configured for the `wowxptr` CASC product and the `ptr.txt` manifest; retail uses the `wow` CASC product and `retail.txt`. The retail manifest mirrors the complete Gethe `live` AddOns tree, including both `Classic/` and `Mainline/` family variants where the live tree contains them. The manifest is a source inventory, not the retail runtime's final TOC selection: retail `[Family]` substitution resolves to `Mainline`, while profile-aware TOC and game-type filtering governs which discovered addons load. Other profile manifests remain profile-specific. PTR 12.1.5 reads `data/blizzard-ui-builds/ptr.json`: a generated, committed CDN range index that validates encoded BLTE and decoded content keys. It does not relabel local `wowt` data or fall back to Gethe bytes. The first full cache synchronization extracted 4,025 files. The remaining six-record pixel-rounding baseline is a compatibility gap, not a source-acquisition failure.

Local install discovery uses the active profile's WoW flavor directory. PTR reads addons, WTF, and BlizzardInterfaceArt from `_ptr_`; retail continues to use `_retail_` with optional `_beta_` addon fallback. Forever reads `_classic_beta_` and uses product `wow_classic_beta`, confirmed by local build `1.60.1.69913`. Its 4,398-file manifest comes from the `forever` source branch in the versioned source cache; it is not an Anniversary alias. Interface `16001` is the user-confirmed published [1.60.1 TOC version](https://warcraft.wiki.gg/wiki/Patch_1.60.1/API_changes). `abbc1272f` adds the seventh profile. After commit `65139f909` refreshed the community listfile, its 592 missing mappings resolved and `wow-cli casc sync-blizzard-ui` extracted all 4,398 files. The first no-addons/no-saved-vars startup capture exited 1 with 422 distinct Lua-error records (514 occurrences), committed as `docs/baselines/wowforever-lua-errors.json`; it remains the historical pre-`5e26960b6` failure baseline, not evidence that the family correction has achieved startup compatibility or native-client parity.

## Profile-aware loader paths

### TOC discovery (`src/loader/mod.rs`)

`find_toc_file()` picks the variant matching `ClientProfile::ACTIVE`:

1. `<addon><primary_suffix>.toc` (e.g. `Bartender4_Wrath.toc` under wrath)
2. Plain `<addon>.toc`
3. Any `.toc` whose name doesn't contain another profile's suffix — driven by helpers `active_profile_toc_suffix()` and `other_profile_toc_suffixes()` (`scan_for_compatible_flavor_toc()` is the fallback walker)

### TOC content (`src/toc/mod.rs`)

`is_allowed_game_type()` reads inline `[AllowLoadGameType <type>]` annotations and matches against the active profile's allow-list:

| Profile     | Accepted gametypes                                |
|-------------|---------------------------------------------------|
| Retail      | `mainline`, `standard`                            |
| Ptr         | `mainline`, `standard`                            |
| Wrath       | `wrath`, `wrath_classic`, `classic`               |
| Mists       | `mists`, `mists_classic`, `classic`               |
| Era         | `vanilla`, `classic_era`, `classic`               |
| Anniversary | `vanilla`, `classic_anniversary`, `classic`       |
| WowForever  | `camelot`, `classic`                              |

`family_subdir()` substitutes the `[Family]` TOC token: retail/PTR/Forever → `Mainline`; wrath/mists/era/anniversary → `Classic`. Commit `5e26960b6` corrected Forever from `Classic` after its authenticated source showed that `Mainline/NineSliceLayouts.lua`, `Mainline/InputUtil.lua`, and `Mainline/SharedUIPanelTemplates.lua` are the base files before Camelot overrides. The `[Game]` token maps retail/PTR to `Standard`, wrath to `Wrath`, mists to `Mists`, era/anniversary to `Vanilla`, and Forever to `Camelot`. Both inline and header `ExcludeLoadGameType` filters reject matching game types. Forever continues to accept only `camelot`/`classic` annotations: the family-directory correction does not admit `mainline`-annotated TOC entries. Forever's FrameXML selects `Camelot/StackSplitFrame.xml` instead of the excluded Classic XML. Its WorldMap TOC retains the `_Mainline` filename while containing explicit Camelot entries; that single addon is selected explicitly, not by accepting all mainline-flavored TOCs.

`TocFile::is_game_type_restricted()` evaluates the `## AllowLoadGameType` *header* line (separate from the inline annotation parser) using the same allow-list.

## Per-profile compatibility shims

Each non-retail profile loads its own Lua bootstrap after `runtime_surface_bootstrap.lua` and before secure-environment cloning. Wired in `src/lua_api/env_init/mod.rs`:

| Profile module        | Files                                                                | What it stubs |
|-----------------------|----------------------------------------------------------------------|---------------|
| `src/wrath/`          | `compat_bootstrap.{lua,rs}`, `compat_frame_proxies.lua`, `frame_methods.rs`, `post_load.{lua,rs}` | ~30 wrath-specific stubs + Lua-5.0 string/math aliases + `MiniMapTrackingIcon`/`PlayerArrowEffectFrame` proxies (wrath-only). Also registers `IgnoreDepth`, `SetBackdropColor`, `SetBackdropBorderColor`, `SetPlayerTextureWidth/Height`, `SetMaxBytes`, `GetTextHeight` directly on the frame metatable for code paths that call them as methods. |
| `src/mists/`          | `compat_bootstrap.{lua,rs}`                                          | ~46 mists-only globals (post-Cataclysm leftovers MoP kept that retail removed: `GetActionBarPage`, `GetComboPoints`, `GetQuestLog*` family, `GetRuneType`, etc.) |
| `src/era/`            | `compat_bootstrap.{lua,rs}` (shared by era + anniversary)            | ~30 vanilla-only globals: `IsInGlobalEnvironment`, `GetActionBarPage/Toggles`, `GetComboPoints`, `GetPVPYesterdayStats`, `MoneyFrame_OnLoad`, `MoneyInputFrame_*`, `SecureMixin`, `UIParent_OnLoad`, `IsKeyRingEnabled`, `HasKey`, `HasPetUI`, `SetSelectedSkill`, etc. |

Promotion rule: a stub starts in the per-addon shim (`tools/classic-addon-compat/<addon>/<shim>/<shim>.lua` with `## LoadFirst: 1`); if the same gap surfaces across multiple addons under one profile, it gets promoted to the matching profile-level bootstrap so per-addon shims stay narrow.

The wrath module is shared with mists/era/anniversary at the cfg level (`src/lib.rs`: `#[cfg(any(client-wrath, client-mists, client-era, client-anniversary))] pub mod wrath;`) because all four profiles need its `frame_methods::register_all` (no-op stubs for backdrop / depth / player-texture methods that vendor frames call directly). Only wrath actually loads `compat_bootstrap.lua` itself; mists has its own; era + anniversary share `src/era/compat_bootstrap.lua`. The wrath-only `compat_frame_proxies.lua` (real `Blizzard_SharedXML` would shadow it on mists) is gated tighter at `#[cfg(feature = "client-wrath")]`.

Forever loads no era/anniversary compatibility bootstrap. Its temporary `GetBuildInfo()` identity uses `1.60.1` / `69913` / `16001`; date and trailing values remain defaults. Its event validator shares the finite known-event tables without enabling retail epoch deltas; unmodeled Forever-specific events remain a documented gap.

`src/event/valid_events.rs` follows the same shape: retail/PTR use the strict generated event tables grouped under `src/event/known_events.rs`; wrath/mists/era/anniversary route through `crate::wrath::is_registerable_event(name)` which accepts any non-empty event name (the mainline `events.yaml` doesn't cover pre-Cataclysm or vanilla). Patch-specific event additions/removals inside the mainline table gate on the retail epoch feature, not on `client-ptr`.

## Synthetic FrameXML addon (wrath only)

Wrath ships its UI as a flat `Interface/FrameXML/` tree alongside `Interface/AddOns/`; retail/PTR/mists/era/anniversary collapsed FrameXML into a `Blizzard_FrameXML` addon. The loader detects this via `client_profile::blizzard_ui_framexml_toc()` and synthesizes a virtual addon called `FrameXML` that loads before the regular `Blizzard_*` discovery pass.

## CI matrix

`.github/workflows/test.yml` `client-profile-smoke` currently runs the Mists profile only. The job runs `setup-blizzard-ui.sh mists` → `cargo build --features client-mists` → `cargo check --tests` → `lua-errors > lua-errors.json`, then diffs against `docs/baselines/mists-lua-errors.json`. Retail stays on the dedicated `cargo-test` job because tests are written against the retail UI surface.

The addon harness is Mists-only and is driven locally by `scripts/test-classic-addons.sh` / `scripts/ci-mists-guard.sh` from `tools/classic-addon-manifest.tsv`. See `docs/baselines/classic-addon-test-targets.md` for the retained addon picks.

## Mists baselines

Captured in `docs/baselines/`:

- `mists-lua-errors.json` — clean boot-time error snapshot
- `mists-panels.md`, `mists-panel-interactions.md`, and `mists-panel-visuals.tsv` — panel parity artifacts
- `mists-test-coverage.md`, `mists-release-proof.md`, and `mists-lod-audit.md` — retained Mists proof notes
- `classic-addon-test-targets.md` — Mists addon harness target set

## Sources

- `Cargo.toml` — mutually-exclusive `client-*` profile features and cumulative `retail-*` API epoch features
- `src/client_profile.rs` — enum, `ACTIVE` const, profile path helpers, active API interface constants
- `src/asset_resolver_config.rs` — profile-to-CASC-product mapping
- `src/loader/mod.rs` — `find_toc_file`, `active_profile_toc_suffix`, `other_profile_toc_suffixes`
- `src/toc/mod.rs` — `is_allowed_game_type`, `family_subdir`, `TocFile::is_game_type_restricted`
- `src/lib.rs` — `pub mod wrath`/`mists`/`era` cfg gates
- `src/lua_api/env_init/mod.rs` — bootstrap call ordering
- `src/wrath/`, `src/mists/`, `src/era/` — per-profile modules
- `src/event/valid_events.rs` — strict vs permissive event validator dispatch
- `scripts/setup-blizzard-ui.sh`, `scripts/init-worktree.sh` — vendor pinning
- `.github/workflows/{test,addon-harness}.yml` — CI matrix
- `src/blizzard_ui_sync/{pinned,pinned_download}.rs` — immutable PTR CDN sync and validation
- `data/blizzard-ui-builds/ptr.json` — PTR 12.1.5 build/content identity
- [PTR source spec](../../specs/ptr-blizzard-ui-source.md) — cache and startup proof boundary

## See Also

- [[addon-loading]] — TOC discovery, addon load order, SavedVariables (now profile-aware)
- [[taint-system]] — `runtime_surface_bootstrap.lua` runs before each profile's compat bootstrap
- [[lua-api]] — frame methods registered globally vs profile-conditional
- [Client profile spec](../../specs/client-profiles.md) — supported bundle contract and current gaps
- [Forever 1.60.1 running report](../../wowforever-1.60.1.md) — committed changes, proof, and current limitations
- [PTR CDN content index](../../ptr-cdn-content-index.md) — offline regeneration contract
- [[event-system]] — strict-vs-permissive event validator gating
