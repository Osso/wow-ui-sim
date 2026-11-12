# Addon load phases

This spec defines startup phase order and dependency handling in `src/loader/` and `src/c_api/c_addons_runtime.rs`. The [addon loading pipeline](../addon-loading-pipeline.md) describes the current implementation.

## What it must do

### Startup phases

- [x] Phase one must select addons with `LoadFirst` or `UseSecureEnvironment`.
- [x] Phase two must select the remaining non-insecure packaged UI addons, and its top-level traversal must begin only after phase one returns.
- [x] Phase membership must not come from addon names or a synthetic foundational-addon list.

### Startup prerequisites

- [x] Startup must try each available `LoadManagers` addon before the addon that declares it.
- [x] Startup must attempt optional dependencies before required dependencies.
- [x] An optional dependency failure must not stop the parent addon from loading.

### Runtime loading

- [x] `C_AddOns.LoadAddOn` must attempt optional dependencies before required dependencies.
- [x] An optional dependency failure must not stop the requested addon from loading.

### `LoadWith` followers

- [x] VERIFIED (Retail 12.1.5.70688): an addon X that names Y in `LoadWith` joins Y's follower list; resolving the relation does not load Y.
- [x] VERIFIED (Retail 12.1.5.70688): when Y reaches follower processing, it submits X to the general loader after Y's own files. This applies to startup and runtime `C_AddOns.LoadAddOn(Y)`.
- [x] VERIFIED (Retail 12.1.5.70688): each follower uses the general loader's checks and dependency order: optional dependencies, required dependencies, then its own files.
- [x] VERIFIED (Retail 12.1.5.70688): an early-exit Y submits no followers.

## How it works

- [Addon loading pipeline](../addon-loading-pipeline.md)
- [Addon-loading system](../wiki/systems/addon-loading.md#blizzard-addon-load-order)

## Implementation inventory

- `crates/sim-toc/src/lib.rs` — parses `LoadManagers` metadata.
- `src/loader/addon_order.rs` — orders packaged startup addons and dependencies.
- `src/loader/mod.rs` — discovers startup roots and prerequisite addons.
- `src/loader/startup_addons.rs` — builds the ordered startup stream and submits startup followers.
- `src/bin/wow_sim/addon_loading.rs` — skips startup roots that the follower loader already loaded.
- `src/c_api/c_addons.rs` — exposes startup follower processing through the runtime loader.
- `src/c_api/c_addons_runtime.rs` — resolves runtime dependencies and submits `LoadWith` followers.
- `src/lua_api/state/sim_state.rs` — holds the resolved target-to-follower index.
- `src/lua_api/state.rs` — initializes the follower index.
- `src/lua_api/state/ui_reload.rs` — clears the follower index on UI reload.

## Tests asserting this spec

- `src/loader/tests/screen_selection.rs`
- `tests/addon_runtime_dependencies.rs`
- `src/c_api/c_addons_runtime_tests.rs`

## Known gaps (current cycle)

- [ ] UNKNOWN: PTR 12.1.5.70668 phase predicate and parser rules are not established by R64.
- [ ] INFERRED: user AddOns-directory inputs correspond to native phase three. This inference is not asserted by tests.
- [ ] UNKNOWN: Whether a LoadOnDemand `LoadWith` target receives a startup request through another path.
- [ ] UNKNOWN: `LoadWith` behavior on PTR and Forever; R64 verifies it only on Retail 12.1.5.70688.
- [ ] UNKNOWN: Whether follower load failures propagate to the triggering addon. The simulator keeps the trigger result based on the trigger's own load and records follower failures separately.
- [ ] UNKNOWN: The callers and observable effect of the internal mode-one load-on-demand dependency request; dependency file loading remains unchanged.

## Out of scope

- Modeling mode-one dependency-chain-only traversal.
- Asserting the inferred association between user AddOns-directory inputs and native phase three.
