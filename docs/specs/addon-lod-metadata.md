# Third-party LoadOnDemand metadata

Eligible third-party addons must be discoverable through `C_AddOns` before their code runs. `src/bin/wow_sim/addon_loading.rs` separates metadata registration from startup execution; see the [loading pipeline](../addon-loading-pipeline.md) and [bootstrap contract](addon-bootstrap-loading.md).

## What it must do

- [x] Register eligible LoadOnDemand addon names, titles, notes, dependency lists, metadata, and effective enabled states before executing third-party startup Lua or its eagerly required Blizzard dependencies.
- [x] Keep ordinary LoD Lua/XML and its Blizzard dependencies unexecuted at startup. Metadata queries report LoD status and `IsAddOnLoaded` reports false until an explicit load occurs.
- [x] Explicit `C_AddOns.LoadAddOn` loads the addon and required dependencies once through the existing runtime loader. Installed-addon count and metadata remain stable; repeated requests do not execute files again.
- [x] Keep disabled LoD metadata queryable; loading it returns `DISABLED` without executing files or treating deferred missing dependencies as startup errors.
- [x] Preserve game-type, screen, interface, enabled-state, dependency ordering, and bootstrap eligibility rules. Eager addons still execute at startup; summary counts describe startup candidates, not metadata-only entries.

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

## Verified proof

At `350f5444a`, isolated Forever builds compiled the integration target, library target, and `wow-sim` binary target. Direct tests passed: third-party LoD 3/3 and the broader `addon_loading::tests::` group 8/8. `cargo check --offline --no-default-features --features gui,client-wowforever` and `cargo fmt --check` also passed. Ledgers: `/tmp/forever-addon-audit/verify-0e23609d6-{integration-no-run,lib-no-run,wow-sim-no-run}-ledger.json`, `/tmp/forever-addon-audit/verify-350f5444-direct-tests-ledger.json`, and `/tmp/forever-addon-audit/verify-350f5444-cargo-check-ledger.json`.

The isolated real DBM-GUI replay at `55ee20d7` observed metadata and the LoD/dependency flags while `IsAddOnLoaded` was `(false, false)` and `DBM_GUI` was absent. Explicit `C_AddOns.LoadAddOn("DBM-GUI")` then returned true, marked it loaded, and created `DBM_GUI` and `DBM_GUI_OptionsFrame` with zero collected Lua errors. Evidence: `/tmp/forever-addon-runtime/dbm-lod-native-e4hvi488/{stdout,ledger.json}`.

## Known gaps (current cycle)

- This proves one explicit third-party load path and loader invariants, not DBM-GUI visual behavior, other deferred addons, dependency completeness outside the tested closure, or inventory-wide compatibility.
- The historical DBM/Ellesmere observations reproduce the prior false LoD metadata boundary: the old scan removed ordinary LoD TOCs before registry insertion. They do not replace the current proof.

## Out of scope

New interface/version policy, TOC suffix selection, eager loading to satisfy metadata queries, vendor edits, formatter changes, or new third-party dependency-promotion rules.
