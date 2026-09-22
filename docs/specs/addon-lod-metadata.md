# Third-party LoadOnDemand metadata

Eligible third-party addons must be discoverable through `C_AddOns` before their code runs. `src/bin/wow_sim/addon_loading.rs` separates metadata registration from startup execution; see the [loading pipeline](../addon-loading-pipeline.md) and [bootstrap contract](addon-bootstrap-loading.md).

## What it must do

- [ ] Register eligible LoadOnDemand addon names, titles, notes, dependency lists, metadata, and effective enabled states before executing third-party startup Lua or its eagerly required Blizzard dependencies.
- [ ] Keep ordinary LoD Lua/XML and its Blizzard dependencies unexecuted at startup. Metadata queries report LoD status and `IsAddOnLoaded` reports false until an explicit load occurs.
- [ ] Explicit `C_AddOns.LoadAddOn` loads the addon and required dependencies once through the existing runtime loader. Installed-addon count and metadata remain stable; repeated requests do not execute files again.
- [ ] Keep disabled LoD metadata queryable; loading it returns `DISABLED` without executing files or treating deferred missing dependencies as startup errors.
- [ ] Preserve game-type, screen, interface, enabled-state, dependency ordering, and bootstrap eligibility rules. Eager addons still execute at startup; summary counts describe startup candidates, not metadata-only entries.

## How it works

- [Addon loading pipeline](../addon-loading-pipeline.md)
- [Addon bootstrap loading](addon-bootstrap-loading.md)
- [Lua API architecture](../lua-api.md)

## Implementation inventory

- `src/bin/wow_sim/addon_loading.rs` — eligible TOC discovery, complete metadata registration, then separate startup selection and dependency preload.
- `src/c_api/c_addons.rs` — existing metadata and loaded-state queries; unchanged.
- `src/c_api/c_addons_runtime.rs` — existing explicit load/dependency lifecycle; unchanged.

## Tests asserting this spec

- `src/bin/wow_sim/addon_loading/tests.rs::third_party_lod_metadata_precedes_startup_and_explicit_load_runs_once` — real temporary eager/LoD addons, deferred Lua/XML and Blizzard dependency counters, metadata queries from the first startup script, explicit load, and repeated-load/count stability.
- `src/bin/wow_sim/addon_loading/tests.rs::third_party_disabled_lod_metadata_retains_enable_state_and_dependencies` — effective disabled metadata and deferred missing dependency.
- `src/bin/wow_sim/addon_loading/tests.rs::third_party_lod_discovery_preserves_screen_game_and_interface_filters` — LoD discovery still honors existing screen, game-type, and interface restrictions.
- Existing tests in that same binary target cover bootstrap ordering, enabled filtering, and startup failure summaries.

## Known gaps (current cycle)

- [ ] Parent compilation and focused GREEN remain pending; no Cargo or runtime execution was performed in this implementation slice.
- The isolated DBM/Ellesmere batch observations reproduce false LoD metadata before loading. The old scan removed non-bootstrap LoD TOCs before registry insertion; the observer was reading that incomplete registry correctly.

## Out of scope

New interface/version policy, TOC suffix selection, eager loading to satisfy metadata queries, vendor edits, formatter changes, or new third-party dependency-promotion rules.
