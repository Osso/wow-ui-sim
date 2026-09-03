# Addon Loading

The addon loading pipeline discovers addon directories, parses TOC files, loads Lua and XML files in declared order, applies templates, fires startup events, and initializes SavedVariables.

## Blizzard UI cache & per-profile setup

The Blizzard UI source isn't checked into this repo. Runtime loading reads the active profile from a user-cache tree:

```
~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns
~/.cache/wow-ui-sim/blizzard-ui/ptr/AddOns
~/.cache/wow-ui-sim/blizzard-ui/wrath/AddOns
~/.cache/wow-ui-sim/blizzard-ui/mists/AddOns
~/.cache/wow-ui-sim/blizzard-ui/era/AddOns
~/.cache/wow-ui-sim/blizzard-ui/anniversary/AddOns
```

The active profile (selected by the `client-*` cargo feature; see [[client-profiles]]) is the only one the loader reads from at runtime. Multiple profile caches can coexist on disk so a developer can flip features without replacing another profile's files.

`wow-cli casc sync-blizzard-ui` populates the active profile cache from the matching committed manifest in `data/blizzard-ui-files/<profile>.txt`. It writes `.wow-ui-sim-blizzard-ui-complete` plus `.wow-ui-sim-blizzard-ui-provenance` after the manifest is synced. Provenance records the profile, CASC product, active `.build.info` version/build key, optional install key, and compiled manifest hash; a mismatch removes only the active profile cache before extraction, so rebuilt patch syncs refresh automatically. Each profile manifest is paired with the active CASC product selected by the `client-*` feature (`wow` for retail, `wowt` for PTR). PTR currently carries a real 12.1 delta (`Blizzard_AuraContainer/*`). Retail `retail.txt` mirrors the complete Gethe `live` AddOns tree, including `Classic/` and `Mainline/` family variants. That source completeness does not make both variants load: retail TOC `[Family]` substitution selects `Mainline`, and profile-aware TOC/game-type filtering governs addon discovery.

`scripts/setup-blizzard-ui.sh` and `scripts/setup-blizzard-ui.ps1` are compatibility wrappers around the same sync command. Their optional profile argument is accepted for old workflows, but the Cargo feature still selects the active profile.

```bash
wow-cli casc sync-blizzard-ui
./scripts/setup-blizzard-ui.sh
./scripts/init-worktree.sh
```

`scripts/init-worktree.sh` is the bootstrap for a fresh worktree: it syncs the active profile cache so startup, benchmarks, and tests can find Blizzard addon sources. Without the completed cache, or when required profile files are missing from an otherwise completed cache, startup re-runs the sync/reports the stale `~/.cache/wow-ui-sim/blizzard-ui/<profile>/AddOns` tree instead of falling back to repo-local symlinks.

There is no repo-source fallback for Blizzard UI cache population. Missing manifest entries fail the sync so the CASC/listfile/install problem stays visible. If active-product CASC metadata resolves a file but the local streaming install lacks the archive chunk, sync may fetch that missing authoritative chunk from Blizzard CDN by encoding key; this is still CASC-backed, not a source mirror.

## Addon Discovery and TOC Parsing

`find_toc_file()` (`src/loader/mod.rs`) prefers `{AddonName}<suffix>.toc` (suffix is profile-dependent: `_Mainline` retail, `_Wrath` wrath, `_Mists` mists, `_Vanilla` era + anniversary) over `{AddonName}.toc` over the first `.toc` whose name doesn't carry another profile's suffix. The suffix table and exclude list live in `active_profile_toc_suffix()` / `other_profile_toc_suffixes()`. See [[client-profiles]].

`TocFile` fields: `addon_dir`, `name`, `metadata: HashMap<String, String>` (Interface version, Title, Dependencies, RequiredDeps, OptionalDeps, LoadOnDemand, SavedVariables, SavedVariablesPerCharacter), `files: Vec<PathBuf>` for TOC-order addon loading, and `file_is_bootstrap: Vec<bool>` marking inline `[Bootstrap]` entries without moving them.

TOC parsing strips `#` comments, skips `[AllowLoadTextLocale]` lines for non-enUS, and skips `[AllowLoadGameType]` lines unless any inline gametype matches the active profile's allow-list (retail: mainline/standard; wrath: wrath/wrath_classic/classic; mists: mists/mists_classic/classic; era: vanilla/classic_era/classic; anniversary: vanilla/classic_anniversary/classic). Inline game types may be comma- or whitespace-delimited, so `[AllowLoadGameType vanilla tbc mainline]` admits the current retail `mainline` file. It substitutes `[Family]` for the profile's family-subdir (Mainline retail, Classic everywhere else), substitutes `[Game]` for the active profile's game subdir, and normalizes backslashes. Path resolution is case-insensitive for Windows/macOS compatibility. See [[client-profiles]] for the full profile-aware tables.

`[Bootstrap]` does not reorder TOC files and does not create a separate startup/bootstrap pass. A line such as `Bootstrap.lua [Bootstrap]` stays in `files` at that exact position and executes inline when the addon loads. A LoadOnDemand addon executes at startup only when discovery explicitly selects it as a startup root or dependency; its annotation alone does not select it. A self `C_AddOns.LoadAddOn(thisAddon)` call from an executing `[Bootstrap]` file is a benign reentrancy no-op and does not recursively load later normal files early. Bootstrap files referenced by an active profile's manifest or TOC must be represented in that profile's required-entry set so stale completed caches are rejected when the file is missing. Retail and PTR both require `Blizzard_CooldownBroadcaster_Bootstrap.lua` for the current 12.1 manifest; stale PTR-only filtering previously let retail accept an incomplete cache.

## Load Flow (`src/loader/addon.rs`)

During startup, discovery loads non-LoadOnDemand roots plus explicit LoadOnDemand startup roots and their hard dependencies. A `[Bootstrap]` annotation alone never selects a LoadOnDemand addon. Selected LoadOnDemand addons, including `Blizzard_CombatLog`, run their complete TOCs in declared order; later `C_AddOns.LoadAddOn` calls are idempotent. Other LoadOnDemand addons remain metadata-only until explicitly loaded.

`AddonContext` holds `name`, private Lua `table`, and `addon_root`. Per-file process:
1. Check local overlay at `./Interface/AddOns/{addon}/{file}` first, fall back to addon root
2. `.lua` → `load_lua_file()`: strip BOM, transform path to `@Interface/AddOns/...` for debugstack, execute with `(addonName, addonTable)` varargs
3. `.xml` → `load_xml_file()`: parse with quick_xml, dispatch elements (Script/Include → load file; Font/FontFamily → create font object; ScopedModifier → recurse; frames → `create_frame_from_xml()`)
4. After each `.lua` file: inject C++ mixin stubs (empty `ModelSceneControlButtonMixin.OnLoad`, etc.)

`LoadResult` includes per-addon timing breakdown: `io_time`, `xml_parse_time`, `lua_exec_time`, `saved_vars_time`.

### Idempotent Loaded-Addon Loads

`load_addon_internal()` checks the `SimState` addon record before executing files. If the same addon is already marked loaded, the call returns an empty `LoadResult` without re-running Lua/XML files or post-load patches. This preserves mutable registries created by earlier files, including `StaticPopupDialogs`, and prevents repeated dependency encounters from replacing state. The separate loading transaction still handles in-progress re-entry: an addon is not marked loaded until its files and post-load work complete.

Retail 12.1 removes public `GetInventorySlotInfo`, while current `Blizzard_TransmogShared` uses `C_PaperDollInfo.GetInventorySlotInfo` directly. It has no loader-scoped legacy-global exception. See [[transmog-inventory-slot-scope]] for the retired stale-source workaround.

## Secure Replay Allowlist

The secure Lua environment is separate from `_G`; it has no generic `_G` fallback. After normal loading, the loader explicitly replays only selected Blizzard library addons into `__secureenv` through `is_secure_replay_library_addon()` (`src/loader/addon.rs`). The current allowlist includes `Blizzard_SharedXMLBase`, `Blizzard_SharedXML`, `Blizzard_FrameXMLUtil`, `Blizzard_CombatLogBase`, `Blizzard_CatalogShopSharedTemplates`, `Blizzard_CatalogShopSharedUtil`, `Blizzard_AsyncRequest`, `Blizzard_GameTooltip`, and `Blizzard_RaidWarning`.

`Blizzard_FrameXMLUtil` is replayed because secure `Blizzard_AuraContainer` resolves `AuraUtil.DefaultAuraCompare` and `AuraUtil.UnitFrameDebuffComparator`. Public-only loading left `__secureenv` with stale `AuraUtil`, aborting TargetFrame aura initialization before `FocusFrame` creation. `Blizzard_RaidWarning` is replayed because secure `Blizzard_PrivateAurasUI` resolves its declared dependency's `RaidWarningUtil` through `__secureenv`; its secure XML uses `RaidWarningUtil.MessageType`. This is an evidence-backed library list, not generic mirroring of `_G`; additions require a secure consumer and focused replay proof. Coverage: `loader::tests::lua_loading::blizzard_frame_xml_util_replays_aura_comparators_into_secure_environment`, `tests/blizzard_combat_log_processor_loads.rs::blizzard_combat_log_base_replays_util_into_secure_environment`, `tests/blizzard_catalog_shop_shared_util_loads.rs::blizzard_catalog_shop_shared_util_replays_helpers_into_secure_environment`, and `tests/blizzard_private_auras_ui_loads.rs::blizzard_private_auras_ui_reads_raid_warning_util_and_publishes_mixins_into_secure_env`.

## Blizzard Addon Load Order

Retail and PTR Blizzard startup discovery is root-and-closure based. Eligible non-LoadOnDemand cache directories named `Blizzard_*` are roots after screen/profile filtering. The candidate pool also contains eligible LoadOnDemand Blizzard TOCs and eligible non-Blizzard TOCs; discovery retains the transitive hard `## Dependencies:` closure plus explicit LoD startup roots, then topologically sorts it. `Blizzard_Game` depends on `Blizzard_TimeManager`, `Blizzard_CooldownBroadcaster`, `Blizzard_BoostTutorial`, and `Blizzard_CombatLog`; CombatLog's declared `Blizzard_CombatLogBase` and `Blizzard_CombatLogProcessor` dependencies load before it, publishing `CombatLog_LoadUI` before `PLAYER_LOGIN`. `Blizzard_MacroUI` and `Blizzard_TrainerUI` are standalone Game-only LoD roots, publishing `MacroFrame_LoadUI` and `ClassTrainerFrame_LoadUI` while retaining their LoD metadata and glue-screen exclusion. `Blizzard_AchievementUI` is another standalone Game-only LoD root: its complete TOC publishes `AchievementFrame_LoadUI` before `AlertFrame` handles `ACHIEVEMENT_EARNED`, restoring the real achievement-toast queue path while retaining its LoD metadata and glue-screen exclusion. `Blizzard_Transmog` is also an explicit Game-only LoD root; its complete TOC publishes `Transmog_LoadUI` from `Blizzard_Transmog_Bootstrap.lua` and registers the Transmogrifier interaction without introducing a bootstrap-only pass. `Blizzard_RaidUI` is an LoD startup root that depends on `Blizzard_RaidFrame`, so RaidFrame loads first and RaidUI is ready before startup events. An LoD key in the implicit startup-dependency map is selected as a root. Thus retail `Blizzard_TutorialManager` pulls `middleclass` before itself, while unrelated `Deprecated_PaperDoll` remains excluded. `[Bootstrap]` marks an inline TOC entry only: it neither selects every LoD addon nor reorders files. Foundational SharedXML addons are promoted to `LoadFirst` so templates exist before other Blizzard addons instantiate frames. Older docs referred to a 27-addon hardcoded retail list; current runtime loading is discovery-based.

Third-party addons load after the Blizzard startup pass. Third-party LoadOnDemand addons are registered during discovery but do not execute `[Bootstrap]` entries at startup. Non-LoadOnDemand third-party addons execute `[Bootstrap]` inline with normal files.

The non-retail profiles discover different addon sets:

- **Wrath** ships its UI as a flat `Interface/FrameXML/` tree alongside `Interface/AddOns/`, with no `Blizzard_FrameXML` addon. The loader detects this via `client_profile::blizzard_ui_framexml_toc()` (Some → wrath layout, None → addon layout) and synthesizes a virtual `FrameXML` addon that loads before the regular `Blizzard_*` discovery pass. Wrath ships 24 `Blizzard_*` addons + the synthetic FrameXML.
- **Mists** ships ~112 `Blizzard_*` addons (most retail addons exist with `_Mists.toc` variants).
- **Era / Anniversary** ship the vanilla multi-flavor addon set (Era uses ~35 `Blizzard_*_Vanilla.toc` variants; Anniversary uses the same vanilla TOCs against a 2.5.5 build).

Counts above are the discovered set after filtering by `is_allowed_game_type` and `default_enabled`; the on-disk addon directory may carry many more `_Cata.toc` / `_TBC.toc` / etc. variants the active profile skips.

## Third-Party Addon Enable State

Third-party addon state starts from the real character `WTF/Account/{account}/{realm}/{character}/AddOns.txt` when WTF import is enabled. When the `ServerSnapshot` addon has written `ServerSnapshotDB`, the captured `characters[...].addons.entries[*].enabled` values overlay the file state before third-party addon loading. This is the preferred live-client source because the AddOn List UI can expose per-character/dependency state that is not reliably represented by `AddOns.txt` rows alone.

The simulator-local `~/.local/share/wow-sim/AddOns.txt` remains a compatibility overlay after the real WTF file. Avoid treating it as the primary source for real-client state; it is a simulator UI save artifact, not a live-client snapshot.

The effective state pass is dependency-aware. Required dependencies can be enabled for explicitly enabled addons, but addons whose required dependency is explicitly disabled are disabled even when the dependent addon's own TOC default is enabled. This prevents data-shard addons such as RaiderIO DB modules from loading without their required base addon.

Third-party addon metadata is registered in `C_AddOns` for the full discovered, dependency-sorted addon list before any eager third-party Lua executes. The loader then runs only enabled, non-`LoadOnDemand` addons in order. This matches addons that inspect TOC metadata during file load: for example, `!BugGrabber` scans `C_AddOns.GetNumAddOns()` and `GetAddOnMetadata(i, "X-BugGrabber-Display")` before `BugSack` Lua runs, so `BugSack` must already be visible as an enabled addon with its display metadata even though its files load later.

## XML Element Handlers

| Category | Elements |
|----------|----------|
| File refs | Script, Include (case-insensitive variants) |
| Frames | Frame, Button, CheckButton, EditBox, ScrollFrame, Slider, StatusBar, GameTooltip, Model, ModelScene, MessageFrame, Minimap |
| Regions | Texture, FontString |
| Containers | ScopedModifier |
| Fonts | Font, FontFamily |

Frame creation generates and executes a Lua string: `CreateFrame(...)` + configuration calls + `SetScript(...)`. Template application runs automatically when `inherits` is specified.

## SavedVariables (`src/saved_variables.rs`)

Priority: WTF loading (`WTF/Account/{account}/SavedVariables/{addon}.lua` and per-character variant) then JSON fallback stored in `~/.local/share/wow-sim/SavedVariables/`. `SavedVariablesManager` tracks registered variables per addon, loads before Lua execution, and persists after shutdown.

## Startup Sequence

1. Create `WowLuaEnv`, set paths, configure SavedVariables, import account `bindings-cache.wtf`, and import `ServerSnapshotDB` action bars/keybindings when present
2. Load Blizzard non-LoD roots, explicit LoD startup roots, and their dependencies in order; every selected addon loads its full TOC, with no bootstrap-only pass
3. Load enabled third-party startup addons alphabetically/dependency-sorted, overlaying `ServerSnapshotDB` addon states when present; third-party LoD addons remain unloaded unless explicitly requested
4. Apply post-load workarounds, including idempotent reconciliation of replacement `_G.SettingsPanel`/`Settings` surfaces before later category registration/opening (plus UpdateMicroButtons and WorldMapFrame scroll initialization)
5. Fire startup events: `ADDON_LOADED("WoWUISim")`, `VARIABLES_LOADED`, `PLAYER_LOGIN`, `PLAYER_ENTERING_WORLD(true, false)`, `UPDATE_BINDINGS`, `DISPLAY_SIZE_CHANGED`, `UI_SCALE_CHANGED`
6. Launch GUI, dump frame tree, or render screenshot

## Error Handling

`LoadError` wraps Io, Toc, Xml, Lua variants. Recoverable loader/XML/Lua failures accumulate in `LoadResult.warnings`; typed nil-symbol observations and missing `C_*` requirements use separate fields. Path resolution tries four strategies (case-sensitive/insensitive × relative-to-xml/addon-root) before failing.

### Nil-Symbol Diagnostic Reconciliation

Implementation: [nil-symbol reports](../../../src/loader/addon/nil_symbol_reports.rs), [publication callbacks](../../../src/lua_api/globals/compat_overrides.rs), and [named-frame publication](../../../src/lua_api/globals/create_frame/helpers_shared.rs).

Nil-symbol diagnostics remain strict and typed. Direct syntactic regular-global misses become `NilSymbolObservation` values in `LoadResult.nil_symbol_observations`; missing `C_*` namespaces and members become `MissingRequirement` values in `LoadResult.missing_requirements`. Each value retains addon, source, line, and public/secure environment attribution and is deduplicated by environment and symbol kind. Loader tracing and `WOW_SIM_DEBUG_NIL_GLOBALS` expose both channels deterministically. Normal startup health gates only actual failures in `LoadResult.warnings`; unsupported or unmodeled API requirements remain visible without placeholder implementations.

Explicit `_G.name` and `_G[name]` reads of missing regular globals are ordinary optional probes: they do not create a nil-symbol record or enter the `__wow_logged_nil_symbols` dedup cache. Explicit `_G` access does not relax `C_*` diagnostics. A non-`C_*` global read as nil is reconciled only when that same addon explicitly publishes the name later into the same environment through ordinary Lua assignment or a named XML frame, and the final value is non-nil. Public and secure publications use separate ledgers and final-table checks: secure Lua assignments and Rust secure frame exports record the stable addon index in the secure ledger, so a secure publication cannot resolve a public lookup or vice versa. A nested `C_AddOns.LoadAddOn` publication belongs to the nested addon and does not resolve the outer addon's observation. Globals later cleared remain observations. Both publication ledgers are cleaned up with the `LoadingAddonGuard` transaction lifecycle.

rilua tracks lookup origin in VM execution state. `debug.isglobalindex()` is a read-only query that returns true only while `_G.__index` handles a syntactic global load and false for explicit table reads or calls outside that lookup; the state restores correctly across nested lookups, errors, and coroutine swaps.

Generated lifecycle helpers avoid synthetic observations: precompiled OnLoad/OnShow dispatch snapshots and restores raw `_G.self`, while post-cleanup runtime-surface restoration reads raw `_G.C_StoreSecure` before merging the namespace. This keeps helper state restoration and simulator-owned namespace repair out of client nil-symbol attribution.

Nested runtime-addon loads finalize failures, nil observations, and missing requirements under the nested addon, then forward all three channels exactly once to the immediate parent `LoadResult`; forwarding is transitive for nested-nested loads and does not reprocess raw nil-access records. A top-level runtime `C_AddOns.LoadAddOn` has no parent result, so its typed diagnostics enter the SimState-owned runtime ledger instead. Startup and test collectors drain that ledger exactly once alongside handler errors. The publication recorder used by the global assignment hook is captured in a bootstrap-local upvalue and removed from `_G` before addon code runs, so addon Lua cannot forge publication-ledger entries; ordinary Lua assignments and named XML frame publications remain tracked.

## Sources

- [addon-loading-pipeline.md](../../addon-loading-pipeline.md) — TOC parsing, load flow, XML handlers, SavedVariables, load order
- [addon_loading.rs](../../../src/bin/wow_sim/addon_loading.rs) — Blizzard eager load, third-party addon discovery, metadata pre-registration, enable-state application, and load loop
- [toc/mod.rs](../../../src/toc/mod.rs) — TOC metadata, normal file list, `[Bootstrap]` parsing, path resolution
- [loader/addon.rs](../../../src/loader/addon.rs) — normal addon file execution, inline `[Bootstrap]` entries, load transactions, warning finalization, and the secure replay allowlist
- [c_addons_runtime.rs](../../../src/c_api/c_addons_runtime.rs) — runtime addon loading, typed top-level diagnostic retention, exactly-once nested forwarding, and the scoped `Blizzard_TransmogShared` compatibility window
- [runtime.rs](../../../src/lua_api/state_types/runtime.rs) — typed diagnostic records and public/secure attribution fields
- [runtime surface bootstrap](../../../src/lua_api/env_init/runtime_surface_bootstrap.lua) — `_G.__index` diagnostic gate, strict `C_*` namespace fallback, and raw namespace restoration
- [secure environment](../../../src/lua_api/globals/security/secure_env.rs) — separate secure environment and secure publication recording
- [precompiled lifecycle helpers](../../../src/loader/precompiled.rs) — raw `_G.self` receiver snapshot/restore
- [environment cleanup restore](../../../src/lua_api/workarounds/temporary/environment_cleanup_restore.rs) — post-cleanup namespace restoration regression proof
- [settings surface defaults](../../../src/lua_api/workarounds/temporary/settings_surface_defaults.rs) — post-load replacement SettingsPanel/Settings reconciliation
- [startup warning tests](../../../tests/startup_warnings.rs) and [nil-symbol tests](../../../tests/addon_nil_symbol_report.rs) — zero-failure startup health, typed runtime diagnostic draining, attribution/deduplication, and direct-global versus explicit-`_G` behavior
- rilua commits `1a7c9de` and `3630419` — VM-scoped lookup-origin tracking and the read-only `debug.isglobalindex()` query
- [blizzard-ui-files](../../../data/blizzard-ui-files) — committed per-profile Blizzard UI cache manifests, including bootstrap files needed by each active profile cache
- [blizzard_ui_sync.rs](../../../src/blizzard_ui_sync.rs) and [profile_cache.rs](../../../src/blizzard_ui_sync/profile_cache.rs) — active-profile cache sync and profile-specific cache usability checks
- [secure replay tests](../../../tests/blizzard_combat_log_processor_loads.rs) and [CatalogShop replay tests](../../../tests/blizzard_catalog_shop_shared_util_loads.rs) — proof that selected Blizzard library globals are available in `__secureenv`
- [AccountStore popup regression](../../../tests/blizzard_ui/blizzard_accountstore/behavior_transaction_error.rs) — repeated StaticPopup loading must preserve the existing registry and popup definition

## See Also

- [[client-profiles]] — cargo-feature selection, vendor pinning, profile-aware loader paths and gametypes
- [[xml-template-system]] — XML parsing and template registry populated during loading
- [[event-system]] — startup events fired after all addons load
- [[lua-api]] — WowLuaEnv that executes all Lua files
