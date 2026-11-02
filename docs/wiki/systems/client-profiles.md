# Client Profiles

The simulator targets seven WoW client profiles — retail, PTR, wrath (3.3.5a), mists (5.4 / MoP Classic), era (1.x / Vanilla), anniversary, and WoW Forever — selected at compile time via mutually-exclusive cargo features. Each profile uses a profile-scoped Blizzard UI cache and routes the loader through profile-aware suffix and gametype filters. Mainline API surface differences are gated by cumulative retail API epoch features so `client-retail` and `client-ptr` can point at different patch-note lifetimes without making PTR itself the API truth. Retail-family API availability is gated separately by cumulative patch epoch features so a profile/channel points at an API epoch instead of hard-coding every API delta to `client-ptr`.

## Active profile selection

`src/client_profile.rs` defines `enum ClientProfile { Retail, Ptr, Wrath, Mists, Era, Anniversary, WowForever }` and a single `pub const ACTIVE: ClientProfile` resolved by cfg-blocks against the enabled profile marker. Retail uses the internal `profile-retail` feature; public `client-retail` selects the Retail 12.1.5 bundle, while `client-ptr` selects cumulative 12.1.7 (Interface `120107`), including shared RegionalTextLocale support. A `compile_error!` block enforces exactly one client profile marker.

Feature ↔ profile ↔ vendor source ↔ TOC suffix:

| Feature              | Profile     | Cache subdir | Default API epoch | Primary TOC suffix |
|----------------------|-------------|--------------|-------------------|--------------------|
| `client-retail`      | Retail      | `retail`     | `retail-12-1-5` (`120105`) | `_Mainline` |
| `profile-retail`     | Retail      | `retail`     | selected by enabled epoch | `_Mainline` |
| `client-ptr`         | Ptr         | `ptr`        | `retail-12-1-5` (`120105`) | `_Mainline` |
| `client-wrath`       | Wrath       | `wrath`      | `38001`          | `_Wrath`           |
| `client-mists`       | Mists       | `mists`      | `50504`          | `_Mists`           |
| `client-era`         | Era         | `era`        | `11507`          | `_Vanilla`         |
| `client-anniversary` | Anniversary | `anniversary`| `11507`          | `_Vanilla`         |
| `client-wowforever`  | WowForever  | `wowforever` | `16001`          | `_Camelot` → generic → `_Mainline` |

Retail-family epoch features are cumulative: `retail-12-0-5` includes `retail-12-0-0`, `retail-12-0-7` includes earlier 12.0 epochs, `retail-12-1-0` includes `retail-12-0-7`, and `retail-12-1-5` includes `retail-12-1-0`. `client-retail` selects `retail-12-1-5`/`120105`; `client-ptr` selects `retail-12-1-5`/`120105`; `profile-retail` selects the retail cache without forcing an epoch. `RetailApiEpoch` and `ACTIVE_RETAIL_API_EPOCH` resolve the highest enabled cumulative retail epoch. API surfaces, CVars, enums, events, XML elements, and frame methods introduced by patch notes should gate on the epoch feature rather than on the channel feature. Strict removals follow the same rule unless source evidence proves a profile-specific retirement: retail 12.1 keeps `C_RecruitAFriend.IsEnabled`, while `src/ptr/strict_removals.lua` hides it from PTR addons after startup. Profile-specific runtime behavior gates use `profile-retail` so historical retail tests retain retail semantics. Channel/vendor behavior stays profile-gated: local PTR `_ptr_` install paths and `data/blizzard-ui-files/ptr.txt` remain `client-ptr` concerns. Runtime PTR UI source is independently pinned to official `wowt` 12.1.7.70493 CDN ranges; local install flavor is not source-build authority. [Source identity, regeneration and proof](../../ptr-cdn-content-index.md) owns exact build/config/install/Gethe keys and inventory. `GetBuildInfo()` reads that immutable PTR identity; Retail remains 12.1.5 / `120105`, while historical epochs retain their selected interfaces. Date/trailing slots and non-PTR build defaults remain distinct from installed-product identity. The [actual 70493 baseline](../../baselines/README.md#ptr-70493-authenticated-source-capture--2026-10-24) is failing compatibility inventory, not clean startup or panel proof.

Helper functions/constants in `src/client_profile.rs`:

- `ACTIVE_INTERFACE_VERSION` → active TOC/API interface selected by the profile's epoch
- `RETAIL_API_INTERFACE_VERSION` → retail-family interface selected by the highest enabled retail epoch
- `cache_subdir()` → profile cache directory name under `~/.cache/wow-ui-sim/blizzard-ui/`
- `interface_version()` → legacy profile default, not the preferred API-epoch selector
- `blizzard_ui_addons_dir()` → completed cache path for the active profile, or the profile-scoped default cache path
- `blizzard_ui_addons_dir_under(root)` — test fallback path anchored at `root`
- `blizzard_ui_framexml_toc()` → wrath-only `<cache>/FrameXML/FrameXML.toc`; retail/PTR/mists/era/anniversary collapsed FrameXML into `Blizzard_*` addons

## Forever gamepad Edit Mode — 2026-10-15

[Owning enum contract](../../specs/forever-edit-mode-enums.md#verified-70374-gamepad-scope--2026-10-15) records bounded executed consumer GREEN; [central receipts](../../baselines/README.md#verified-scoped-receipts--2026-10-15) bind it to runtime source136, before the unbuilt import change. Startup remains FAIL; native/cross-profile/full-parent acceptance remains open.

## Forever expansion identity

`374c2c6c7` gives `WowForever` one profile policy for both `LE_EXPANSION_LEVEL_CURRENT` and legacy `GetExpansionLevel()`: `0` / Classic. This is an explicitly **inferred compatibility policy**, not native Forever evidence. Cached Angleur `8932166` runs under `WOW_PROJECT_MAINLINE`, but its Camelot TOC omits retail producer files and its authored predicate chooses the Camelot branch only when the current expansion equals `LE_EXPANSION_CLASSIC`. The prior inherited retail values (`current=11`, legacy API=10) selected the retail branch and reached those absent producers.

The policy leaves all non-Forever current constants at `11` and legacy API defaults at `10`; previous/max/account/display expansion state is unchanged. Grouped tests and an unchanged Angleur replay remain pending, so this page does not claim native numeric parity or general expansion-threshold compatibility.

## Retail API epochs

Mainline API additions/removals use cumulative Cargo features named after the patch epoch. Current chain:

```toml
retail-12-0-0 = []
retail-12-0-5 = ["retail-12-0-0"]
retail-12-0-7 = ["retail-12-0-5"]
retail-12-1-0 = ["retail-12-0-7"]
retail-12-1-5 = ["retail-12-1-0"]
retail-12-1-7 = ["retail-12-1-5", "regional-text-locale"]

profile-retail = []
client-retail = ["profile-retail", "retail-12-1-5", "prefork-full-ui"]
client-ptr = ["retail-12-1-7"]
```

Rules:

- `profile-retail` and the other profile markers select runtime profile: cache subdir, CASC product, install flavor, and TOC profile. `client-retail` is the public Retail 12.1.5 bundle; it enables `profile-retail`, `retail-12-1-5` and the existing prefork capability. `client-ptr` remains a separate PTR profile/cache and selects `retail-12-1-7`.
- `retail-*` features select mainline API epoch: C_* additions, globals/removals, events, CVars, XML elements, frame methods, and patch-note compatibility bootstraps. `RetailApiEpoch` / `ACTIVE_RETAIL_API_EPOCH` select the highest enabled cumulative epoch.
- Historical retail 12.0.0 audit tests use `cargo test --no-default-features --features profile-retail,retail-12-0-0`; profile-specific runtime behavior must gate on `profile-retail`, not `client-retail`.
- Patch features are cumulative. Gate additions with the smallest applicable epoch, for example `#[cfg(feature = "retail-12-1-5")]`; gate removals/lifetimes with the corresponding lower/upper epoch boundary.
- Do not gate patch-note API deltas on `client-ptr` by default. Retail and PTR epochs are cumulative while retaining separate profile/cache behavior; historical tests should select `profile-retail` with the required epoch explicitly. Add a profile-gated strict removal only when the cached channel source proves that channel-specific contract, as with PTR-only removal of `C_RecruitAFriend.IsEnabled`.

## Runtime Cache

Wrath instead uses `data/blizzard-ui-builds/wrath.json` and a separate validated local snapshot; only snapshot absence permits downloading the exact pinned Gethe archive. Invalid existing snapshots fail unchanged. Its 382-entry manifest includes `FrameXML/…`, copied to the profile’s sibling `FrameXML` tree; other entries remain AddOns-relative. Provenance and completion publish only after successful copying. No CASC UI-source fallback or asset-product retargeting is allowed. See the [owning Wrath source contract](../../specs/client-profiles.md#separate-pinned-wrath-source-restoration) for component versus full-startup/native proof limits; the guard remains.

Runtime Blizzard UI files live under the user cache:

```
~/.cache/wow-ui-sim/blizzard-ui/<profile>/AddOns
```

Populate it with `wow-cli casc sync-blizzard-ui` or the compatibility wrapper `scripts/setup-blizzard-ui.sh`. Do not use `Interface/BlizzardUI/` or repo-local `vendor/wow-ui-source-*` checkouts for runtime loading.

Each profile uses its own committed manifest in `data/blizzard-ui-files/<profile>.txt`. Forever uses `wowforever.txt`, generated from the canonical versioned source cache at `~/.cache/wow-ui-sim/wow-ui-source/forever`; `forever` is the source branch, while `wowforever` is the runtime cache profile. It must never be runtime-aliased to Anniversary or Era. PTR uses the independently pinned `wowt` CDN source and `ptr.txt` manifest; retail uses the `wow` CASC product and `retail.txt`. The retail manifest mirrors the complete Gethe `live` AddOns tree, including both `Classic/` and `Mainline/` family variants where the live tree contains them. The manifest is a source inventory, not the retail runtime's final TOC selection: retail `[Family]` substitution resolves to `Mainline`, while profile-aware TOC and game-type filtering governs which discovered addons load. Other profile manifests remain profile-specific. PTR reads `data/blizzard-ui-builds/ptr.json`: a generated immutable CDN range index. The [70493 source contract](../../specs/ptr-blizzard-ui-source.md) requires encoded BLTE identity, decoded content size/hash and exact bounded HTTP ranges. Product CDN discovery reads every advertised Hosts/Servers row, preserves path/scheme, prefers HTTPS and tries subsequent advertised endpoints without weakening content pins or TLS validation. No Gethe runtime-byte fallback exists.

Forever pins [1.60.1.70707](../../../data/blizzard-ui-builds/wowforever.json) with a [4,493-path runtime manifest](../../../data/blizzard-ui-files/wowforever.txt). Its [Gethe recovery](../../../src/blizzard_ui_sync/gethe_missing.rs) checks raw bytes first. On a content-key mismatch it tries inserting CR before bare LF, preserving existing CRLF and the final-newline state. Recovery accepts bytes only when their MD5 exactly matches the selected CASC content key; other mismatches fail.

Cache preparation writes exact schema2 provenance before workers write source files; completion remains separate until every entry succeeds. The process-boundary 4cc regression demonstrated partial-file deletion before this fix. Real 70493 interruption left 306 verified files and no completion marker; supported resume retained all 306 unchanged and completed all 4,578 entries. Provenance mismatches still discard stale source. All hashes/receipts live in the [source proof ledger](../../ptr-cdn-content-index.md#70493-source-and-interrupted-resume-proof); the fixed startup baseline records 436 errors / 618 occurrences after master/toc integration, not source-integrity failure.

Local install discovery uses the active profile's WoW flavor directory. PTR reads addons, WTF, and BlizzardInterfaceArt from `_ptr_`; retail continues to use `_retail_` with optional `_beta_` addon fallback. The October 25, 2026 Forever target retained `_classic_beta_` install-path discovery and selected CASC product `wowf` for 1.60.1.70532; product selection did not prove addon-path parity. The [selected-target checkpoint](../../updating-blizzard-ui-to-a-new-patch.md#current-selected-target--october-25-2026) records the repaired local install gate and then-pending runtime acceptance. Product `wow_classic_beta` / build1.60.1.69913 is historical. Its 4,398-file manifest comes from the `forever` source branch in the versioned source cache; it is not an Anniversary alias. Interface `16001` is the user-confirmed published [1.60.1 TOC version](https://warcraft.wiki.gg/wiki/Patch_1.60.1/API_changes). `abbc1272f` adds the seventh profile. After commit `65139f909` refreshed the community listfile, its 592 missing mappings resolved and `wow-cli casc sync-blizzard-ui` extracted all 4,398 files. The first no-addons/no-saved-vars startup capture exited 1 with 422 distinct Lua-error records (514 occurrences), originally committed as `docs/baselines/wowforever-lua-errors.json`; it remains historical pre-`5e26960b6` failure evidence, not proof that the family correction achieved startup compatibility or native-client parity. The [baseline ledger](../../baselines/README.md#forever-lua-errors) retains historical captures; the [70374 checkpoint](../../updating-blizzard-ui-to-a-new-patch.md#historical-selected-target--october-15-2026) records that epoch's sync and validation scope. The [historical 70532 target](../../updating-blizzard-ui-to-a-new-patch.md#current-selected-target--october-25-2026) selected 4,486 source paths; its runtime sync, provenance and startup baseline were pending at that checkpoint.

### Retail 70448 source and producer update — October 16, 2026

[Selected Retail target and isolation workflow](../../updating-blizzard-ui-to-a-new-patch.md#retail-selected-target--october-16-2026) records installed `wow` 12.1.5.70448 and full build key, exact Gethe source, manifest/listfile counts and unchanged PTR. All 4,026 runtime files were locally extracted into a private XDG cache; their contents match the selected Gethe source after line-ending normalization. Provenance records the exact installed identity and manifest digest; the shared Retail cache is not replaced.

The original consumers exposed a stale 12.1.0 public bundle: `MathUtil.lua` now aliases native math producers and `TimerUtil.lua` creates a TimedSignalMap. The epoch bump exposes existing modeled implementations. Remaining producer gaps were the [Texture metatable](../../specs/texture-metatable.md), [pulse enum metadata](../../specs/forever-combat-audio-pulse-enums.md), finite age-verification event, [secret atlas input](../../specs/texture-atlas-application-result.md), and new Retail slash-command strings. Each has a concrete RED regression; no vendor Lua is edited. Retail uses `global_strings_retail.rs` only at 12.1.5+, leaving earlier epochs and other profiles unchanged. 

Bounded Retail acceptance after rebase onto `9c0251fca`: startup exits 0 with exact `[]`; approved-batch 132/132, quest 82/82 and label/widget 51/51 preservation pass. Existing newly enabled 12.1.5 tests pass on Retail: 89 unit/API cases and 50 integration cases, plus 23 atlas/aura preservation cases. Six historical Intl/macro absence assertions retain their old bodies but now run only before the 12.1.5 epoch; no consumer hashes were re-pinned. Default `cargo check --tests` and workspace `cargo fmt --check` pass. Seven pre-existing unused publication-sweep helper warnings remain, unchanged and unsuppressed.

Runtime/unit/preservation artifact source is `20398a9eb`; corrected integration source is `115761b56`. Production inputs and all 265 preservation test bodies are identical across the latter test-gate-only change and subsequent documentation commits. These are executed simulator receipts, not independent/native/GPU/all-profile/full-suite acceptance. The 65 encrypted GlobalStrings rows remain an explicit acquisition gap, not fabricated strings.

### Installed identity metadata — October 10, 2026

[Selected local `.build.info` projection](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/installed-identity-metadata/identity.json) records six active rows: `wow` 12.1.0.69933; `wow_classic` 5.5.4.70268; `wow_classic_era` 1.15.9.70003; `wowt` 12.1.0.69587; `wowxptr` 12.1.5.70077; `wow_classic_beta` 1.60.1.70338. Verified 2026-10-10; [retention hash](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/installed-identity-metadata/hash-manifest.json).

Installed metadata only: no product.db cross-check, native launch/readiness, simulator runtime selection or API acceptance. The observed `wow_classic` row is Mists-era 5.5.4, not Wrath; no 3.x row appears in this one file, which does not establish global Wrath absence. Beta 1.60.1 identifies installed metadata, not usable keys, catalog compatibility or native API acceptance. Earlier build69913 and startup failures above remain historical evidence.

## PTR stock startup — bounded capture (2026-10-10)

[Independent receipt report](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/ptr-stock-startup/independent-report.md): normal build at `b048251f38b2d53e33b3365590cd227d40fc07d0`, compile exit 0, Cargo/rustc 1.99.0, `--no-default-features` with `sound,gui,casc,client-ptr`; 4,027 PTR cache entries unchanged. Stock headless `dump-tree` with no third-party addons/SavedVariables and runtime sound disabled completed, exit 0, completion marker observed. Captured streams contain no emitted Lua-error records; this does **not** establish zero stored errors. Six manifest deprecation warnings remain.

The [independent stored-error CLI audit](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/ptr-stock-startup/lua-errors-independent-report.md) is **PASS**: separate `lua-errors` execution returned exit 0, exact stdout `[]`, 0 unique errors / 0 occurrences, using the unchanged sealed PTR artifact. This command mode uses `casc=false`; the preceding normal dump's same-in-memory stored-error state remains **UNPROVEN**. Neither receipt establishes whole-profile readiness, native parity, GUI correctness, cold startup, CASC/content-key readiness, or historical empty-baseline validity; historical baseline untouched.

## Forever stock startup — bounded capture (2026-10-10)

[Independent receipt report](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/forever-stock-startup/independent-report.md): recorded epoch `98f625741d01df768af6c3c97324a3cfbc544573`, compile exit 0; normal stock dump completion **PASS**, exit 0 and completion marker observed. Separate stored-error CLI **FAIL**, exit 1: **19 unique errors / 36 occurrences**. At that historical epoch, clean startup was **FAIL**; this is not the historical pre-`5e26960b6` 422-record/514-occurrence baseline above. No whole-profile, native, GUI, or CASC/content-key readiness claim. Recorded HEADs `b048251f38b2d53e33b3365590cd227d40fc07d0` (PTR) and `98f625741d01df768af6c3c97324a3cfbc544573` (Forever) differ, but both source-bound builds contain diagnostic code `0e4bf4161`. Later private-helper gate `72971773c` is excluded; source/artifact binding remains bounded, not hermetic attestation.

Later saved `192210Z` at `ef55bf873` closes only [bounded normal completion and distinct CLI zero](../investigations/integrated-source-and-factory-proof-2026-10-09.md#forever-post-event-repair-stock--bounded-normal-completion-and-distinct-cli-zero): normal stock dump marker/exit0 (`casc=true`); separate CLI exact `[]`, exit0, 0 unique/0 occurrences (`casc=false`), not the same in-memory pool. Cached normal artifact reused from GREEN integration compilation, six manifest warnings retained. Historical19/36 and422/514 remain unchanged; attribute/pool disappearance is not causally attributed. Native/GUI/all-profile/cold-state/third-party/key readiness remains unproven.

## Startup test applicability

The [startup warning applicability inventory](../../specs/client-profiles.md#startup-warning-test-applicability) owns consumer-specific eligibility and the retained FrameXML fixture boundary; profile selection alone is not runtime acceptance.

## Profile-aware loader paths

### TOC discovery (`src/loader/mod.rs`)

`find_toc_file()` matches an ordered list of supported filenames against sorted TOC candidates:

1. Explicit Mists compatibility variants, when applicable: GameMenu/Mainline and UIParentPanelManager/Classic.
2. Active flavor (`Mainline`, `Wrath`, `Mists`, `Vanilla`, or `Camelot`).
3. Plain `<addon>.toc`.
4. Retained compatible alias: Retail/PTR `Standard`, legacy profiles `Classic`, Forever `Mainline`.

Each flavor tier tries `<addon>_<flavor>.toc` before `<addon>-<flavor>.toc`, including case-insensitive matches before proceeding to the next tier. Underscore precedence preserves simulator behavior; native ordering when both separator variants exist is not established. Exact case wins within a spelling, followed by lexical order for case collisions. Unknown/foreign suffixes and renamed folders cannot enter through an unordered scan.

Cached Carbonite ships Retail `Carbonite.toc` alongside `Carbonite-Camelot.toc` (`Interface: 16001`, `X-Camelot-Toc: dash`). Previously generic selection caused the startup interface gate to omit Carbonite, leaving its Info/Notes/Warehouse dependents without their provider. The [profile contract](../../specs/client-profiles.md) covers the package-shaped discovery/filter/loading regression. Classic-family aliases are needed by existing Mists cache entries such as ChatFrameBase, GameTooltip, MoneyFrame, and ObjectAPI.

### TOC content (`src/toc/mod.rs`)

The table below is **simulator policy**, not native-confirmed vocabulary. See [[toc-game-type-tokens]] for the separately bounded native-source observations, filename/filter distinction, and recognition/grammar limits.

`is_allowed_game_type()` reads inline `[AllowLoadGameType <type>]` annotations and matches against the active profile's allow-list:

| Profile     | Accepted gametypes                                |
|-------------|---------------------------------------------------|
| Retail      | `mainline`, `standard`                            |
| Ptr         | `mainline`, `standard`                            |
| Wrath       | `wrath`, `wrath_classic`, `classic`               |
| Mists       | `mists`, `mists_classic`, `classic`               |
| Era         | `vanilla`, `classic_era`, `classic`               |
| Anniversary | `vanilla`, `classic_anniversary`, `classic`       |
| WowForever  | `camelot`, `mainline`                             |

`family_subdir()` substitutes the `[Family]` TOC token: retail/PTR/Forever → `Mainline`; wrath/mists/era/anniversary → `Classic`. Commit `5e26960b6` corrected Forever from `Classic` after its authenticated source showed that `Mainline/NineSliceLayouts.lua`, `Mainline/InputUtil.lua`, and `Mainline/SharedUIPanelTemplates.lua` are the base files before Camelot overrides. The `[Game]` token maps retail/PTR to `Standard`, wrath to `Wrath`, mists to `Mists`, era/anniversary to `Vanilla`, and Forever to `Camelot`. Both inline and header `ExcludeLoadGameType` filters reject matching game types. Forever accepts `camelot`/`mainline` annotations, as covered by the existing profile tests; `classic` is excluded. Its FrameXML selects `Camelot/StackSplitFrame.xml` instead of the excluded Mainline XML. Mainline-only TOCs, including WorldMap and its shared providers, use the general profile precedence rather than an addon-specific exception.

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

Forever loads no era/anniversary compatibility bootstrap. Its temporary `GetBuildInfo()` identity follows the [current profile contract](../../specs/client-profiles.md#what-it-must-do); date and trailing values remain defaults. Its event validator shares the finite known-event tables without enabling retail epoch deltas; unmodeled Forever-specific events remain a documented gap.

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

## Release ICU packaging

[Release packaging](../../release-icu.md) owns Retail ICU provisioning, the macOS
bundle closure, Docker runtime/source coherence and tag-push-only publication.
Mists remains outside the native ICU dependency. The [local acceptance matrix](../../release-icu.md#local-acceptance--2026-10-18) records Linux, Docker and native Windows release/link/startup proof, separate from helper/static checks. macOS is verified only by CI dispatch; that gate remains pending.

## Sources

- `Cargo.toml` — mutually-exclusive `client-*` profile features and cumulative `retail-*` API epoch features
- `src/client_profile.rs` — enum, `ACTIVE` const, profile path helpers, active API interface constants, inferred Forever expansion identity
- [Forever expansion identity](../../specs/forever-expansion-identity.md) — Angleur consumer basis and explicit native-evidence limit
- `src/asset_resolver_config.rs` — profile-to-CASC-product mapping
- `src/loader/mod.rs` — `find_toc_file`, ordered profile suffixes, deterministic separator/case matching
- `src/toc/mod.rs` — `is_allowed_game_type`, `family_subdir`, `TocFile::is_game_type_restricted`
- `src/lib.rs` — `pub mod wrath`/`mists`/`era` cfg gates
- `src/lua_api/env_init/mod.rs` — bootstrap call ordering
- `src/wrath/`, `src/mists/`, `src/era/` — per-profile modules
- `src/event/valid_events.rs` — strict vs permissive event validator dispatch
- `scripts/setup-blizzard-ui.sh`, `scripts/init-worktree.sh` — vendor pinning
- `.github/workflows/{test,addon-harness}.yml` — CI matrix
- [Release ICU packaging](../../release-icu.md) — release/Docker contracts and proof limits
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
