# Addon load phases

This spec defines startup phase order and dependency handling in `src/loader/` and `src/c_api/c_addons_runtime.rs`. The [addon loading pipeline](../addon-loading-pipeline.md) describes the current implementation.

## What it must do

### Startup phases

- [x] Phase one must select addons with `LoadFirst` or `UseSecureEnvironment`.
- [x] Phase two must select the remaining non-insecure packaged UI addons, and its top-level traversal must begin only after phase one returns.
- [x] Phase membership must not come from addon names or a synthetic foundational-addon list.
- [x] VERIFIED (PTR 12.1.7.70668): native startup performs three passes and checks phase flags separately from the active Glue/Game load mask.
- [ ] INFERRED simulator policy (PTR cached TOCs; native live-list membership UNKNOWN): model `Blizzard_CharacterCreate` as a phase-two Glue startup root and load its required LoadOnDemand dependency chain before its files. Do not assert its order relative to `Blizzard_PTRFeedbackGlue`.

### Screen selection

- [x] VERIFIED (R13, Retail 70688): insecure TOC parsing ignores `AllowLoad` and keeps the Game default.
- [x] INFERRED (R64 Q1): ordinary third-party AddOns use insecure parsing. The simulator retains Game-only selection for these addons, even with `AllowLoad: Glue` or `Both`; packaged addons retain explicit screen filtering. See [screen evidence](../wiki/systems/addon-loading.md#screen-qualified-dependency-declarations).

### Startup prerequisites

- [x] Startup must try each available `LoadManagers` addon before the addon that declares it.
- [x] Startup must attempt optional dependencies before required dependencies.
- [x] An optional dependency failure must not stop the parent addon from loading.

### Runtime loading

- [ ] Secure-mode addon and dependency files must resolve global reads through the shared `__secureenv` by default. Ordinary-only globals such as `debug` and `coroutine` stay unavailable there, and per-file environment annotations remain effective.
- [ ] Keep the temporary private snapshot only for explicit `SecureReplay` and generated XML loader paths in this cycle. After the original fallback-disabled PrivateAuras consumer passes, remove that bridge in its own commit; inherited-only values must not enter `__secureenv`.
- [ ] For the Retail 12.1.5.70688 R88n.11b entries, apply each verified member flag and preserve one Lua function value only where R88n.11b proves it. Keep values absent when the simulator has no implementation and flags UNKNOWN without evidence; R88t's separate-function routes are specified below.
- [x] `C_AddOns.LoadAddOn` must attempt optional dependencies before required dependencies.
- [x] An optional dependency failure must not stop the requested addon from loading.

### R88t secure dependency and named publication routes

- [ ] VERIFIED (Retail 12.1.5.70688, R88t): ordinary and secure addon loaded states are separate. A secure request passes its effective mode to dependencies, including dependencies without their own secure directive; optional dependencies load before required dependencies, and dependencies load before the current addon's files.
- [ ] VERIFIED (Retail 12.1.5.70688, R88t): for a dependency with no secure directive and no ordinary loaded state, the loader requests ordinary mode and then secure mode. If a mode is already loaded, the request can return without running its files again.
- [ ] VERIFIED (Retail 12.1.5.70688, R88t): flag 3 publishes `AddSourceLocationExclude` and `table.count` into both environments with separate function objects; the ordinary and secure table libraries are separate.
- [ ] VERIFIED (Retail 12.1.5.70688, R88v.v4): `AddSourceLocationExclude` accepts one string or number, adds a nonempty NUL-terminated copy to shared source-exclusion state, and returns zero values. Matching is a case-sensitive suffix check on file-source text. A template-free `CreateTexture` records the first non-excluded file source and line; `GetSourceLocation` returns the saved `file:line` text and does not rescan.
- [ ] VERIFIED (Retail 12.1.5.70688, R88t): flag 3 publishes the six named `C_ColorOverrides` members into separate namespace tables with separate function objects. The simulator may publish existing handlers only; it must not add placeholder methods without a backing model.
- [ ] VERIFIED (Retail 12.1.5.70688, R88t): constant registration publishes separate `Constants` and `AccountStoreConsts` tables without a member flag. The four verified values are 1, 3, 3139, and 2814 for their named members.
- [ ] VERIFIED (Retail 12.1.5.70688, R88t): the Retail UIParent bootstrap uses named-frame policy flag 32. If ordinary and secure `UIParent` names are empty, both receive the same frame-table object; each destination preserves an existing non-nil name.

### `LoadWith` followers

- [x] VERIFIED (Retail 12.1.5.70688; Forever 1.60.1.70707): each `LoadWith` declaration joins the declaring addon to each named target's follower vector; resolution does not load the target or filter followers by enabled state, phase, origin, or `INSECURE`.
- [x] VERIFIED (Retail 12.1.5.70688): the target completes its own files and fires `ADDON_LOADED` before it submits followers. The follower loop preserves its resolved vector order.
- [x] VERIFIED (Retail 12.1.5.70688): a failing follower does not stop later submissions and does not change the target's success result.
- [x] VERIFIED (Retail 12.1.5.70688; Forever 1.60.1.70707): follower submissions use the generic loader and its dependency order: optional dependencies, required dependencies, then the follower's own files.
- [x] VERIFIED (PTR 12.1.7.70668): the generic loader visits the target's child list after the target's own file-processing path; the resolver does not filter by enabled state, phase, origin, or `INSECURE`.
- [ ] INFERRED simulator policy (PTR 12.1.7.70668): apply the child-list resolver to registered `LoadWith` declarations and suppress duplicates. The PTR test seeds the child list directly; parser population and duplicate handling are not tested. Native parser-to-resolver linkage is UNKNOWN.
- [x] VERIFIED (Retail 12.1.5.70688): generic refusal order is `USER_ADDONS_DISABLED` (14), `WRONG_LOAD_PHASE` (13), `NO_ACTIVE_INTERFACE` (10), `WRONG_ACTIVE_INTERFACE` (11), `WRONG_GAME_TYPE` (12), `DISABLED` (2) / `BANNED` (3) / `NOT_AVAILABLE` (8), `INSECURE` (5), `INCOMPATIBLE` (15), `INTERFACE_VERSION` (7), then required dependencies. Missing and rejected required children use `DEP_MISSING` (1), `DEP_DISABLED` (2), and `DEP_WRONG_GAME_TYPE` (12), as applicable.
- [x] SIMULATOR POLICY: register packaged declarations privately before startup without changing public AddOn enumeration; register user declarations at the existing third-party registration step. Invalidate the follower index when declarations change.
- [x] SIMULATOR POLICY: Retail and Forever submit registered followers in registration order through the shared request path. A disabled registered follower reaches the refusal check and returns `DISABLED`; one failure does not block later followers.

## How it works

- [Addon loading pipeline](../addon-loading-pipeline.md)
- [Addon-loading system](../wiki/systems/addon-loading.md#blizzard-addon-load-order)

## Implementation inventory

- `crates/sim-toc/src/lib.rs` — parses `LoadManagers` metadata.
- `src/loader/addon_order.rs` — orders packaged startup addons and dependencies.
- `src/loader/mod.rs` — discovers startup roots and prerequisite addons.
- `src/loader/startup_addons.rs` — builds the ordered startup stream, registers follower declarations, and submits startup followers.
- `src/bin/wow_sim/addon_loading.rs` — registers packaged follower declarations before Blizzard startup, registers third-party declarations, and skips startup roots already loaded by followers.
- `src/c_api/c_addons.rs` — exposes startup follower processing through the runtime loader.
- `src/c_api/c_addons_runtime.rs` — resolves runtime dependencies and submits `LoadWith` followers.
- `src/c_api/helpers.rs`, `src/c_api/secure_publication.rs` — publish the R88n.11b namespace values implemented by the simulator.
- `src/loader/addon.rs` — orders ordinary and secure dependency requests.
- `src/loader/lua_file.rs`, `src/loader/xml_file.rs`, `src/lua_api/loader_env.rs`, `src/lua_api/globals/security/secure_env.rs` — execute secure files through the shared table and retain private replay only for selected replay paths and generated XML loader chunks.
- `src/lua_api/env_init/enums.rs` — publishes the four verified `Constants.AccountStoreConsts` values.
- `src/lua_api/globals/real/table_extensions.rs` — publishes Retail `table.count`.
- `src/lua_api/globals/create_frame/mod.rs` — publishes the built-in `UIParent` name to secureenv.
- `src/lua_api/workarounds/temporary/debug_environment_defaults.rs`, `src/lua_api/workarounds/temporary/color_defaults.rs` — publish only existing temporary handlers under the verified names.
- `src/lua_api/state/sim_state.rs` — holds ordered registered follower declarations and the resolved target-to-follower index.
- `src/lua_api/state.rs` — initializes the registration vector and follower index.
- `src/lua_api/state/ui_reload.rs` — clears the follower index on UI reload.

## Tests asserting this spec

- `src/loader/tests/screen_selection.rs`
- `tests/addon_runtime_dependencies.rs`
- `src/c_api/c_addons_runtime_tests.rs`
- `tests/secureenv_isolation.rs` — separate loaded states, dependency request order, already-loaded mode behavior, and the one-file source-location consumer.
- `tests/blizzard_private_auras_ui_loads.rs` — the original FrameFactory/Pools, AccountUtil, ColorManager, GameRulesUtil, and PrivateAurasTooltip consumers.

## Known gaps (current cycle)

- [ ] UNKNOWN: PTR 12.1.7.70668 does not establish whether declaration parsing feeds the child-list resolver. The simulator applies the inferred resolver model; PTR tests do not assert parser linkage or duplicate suppression.
- [ ] VERIFIED (R88t, native) but untested in the simulator: an ordinary dependency-request failure must not suppress the following secure request. No evidence-backed input currently makes only the ordinary request fail; a Lua file diagnostic is not proof of that request failure.
- [ ] INFERRED (R88t): when neither mode was loaded and both default file paths complete, each default path executes in its requested environment. Do not claim this exact per-environment count as native parity.
- [ ] VERIFIED (R88v.v4) but untested: the native AddSourceLocationExclude callback uses one shared list; the simulator must store exclusions once for both environments. The two-file caller/wrapper comparison is INFERRED and must not be asserted.
- [ ] Follow-up: the full R88v C_ColorOverrides model is deferred from this lane. Keep the four absent members (`ClearColorOverrides`, `GetDefaultColorForQuality`, `RemoveColorOverride`, and `SetColorOverride`) absent; do not add placeholders or partial handlers.
- [ ] UNKNOWN: transitive copying from ordinary globals into secureenv. The direct dependency helper does not add a blanket-copy route.
- [ ] The original `TooltipDataProcessor` identity and its published secure-table provenance remain deferred to `lt-635e`; this lane verifies that the PrivateAurasTooltip consumer runs without errors.
- [ ] UNKNOWN: PTR phase and demand fields are not mapped to cached TOC directives, and native live-list membership and sort direction are not established.
- [ ] UNKNOWN: Native `LoadWith` behavior on Mists, Era, Anniversary, and Wrath. The simulator keeps their previous dispatch path.
- [ ] INFERRED: a registered phase-three user addon may load early during an earlier target's follower loop. The resolver has no phase filter; tests do not assert this inference.
- [ ] INFERRED: a disabled addon that remains registered can enter the native follower vector and then receive `DISABLED` from the generic loader. Tests verify the simulator refusal, not native vector membership.
- [ ] The simulator does not model all verified refusal predicates, including user-add-on policy, load phase, interface availability, game type, banned/unavailable state, `INSECURE`, or version compatibility.
- [ ] UNKNOWN: Whether a LoadOnDemand `LoadWith` target receives a startup request through another path.
- [ ] UNKNOWN: The callers and observable effect of the internal mode-one load-on-demand dependency request; dependency file loading remains unchanged.

## Out of scope

- Modeling mode-one dependency-chain-only traversal.
- Asserting the inferred association between user AddOns-directory inputs and native phase three.
- Adding a blanket ordinary-to-secure copy; transitive publication remains UNKNOWN.
