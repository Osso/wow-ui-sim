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
- [ ] SIMULATOR POLICY: generated XML chunks in secure mode read live shared-secure publications first. Names absent from `__secureenv` retain their existing private-snapshot result; the read fallback does not publish ordinary-only values into `__secureenv`.
### R88y verified secure publication

- [ ] VERIFIED — Retail 12.1.5.70688 R88y.y1: use only the 6,554 VERIFIED rows from the pinned 6,763-row CSV (`3a8922d84756171f666fe79b61068d3344ce2db866250dede1de384fa9878f82`). Apply flags 1/2/3/6/7 to existing simulator functions only; keep absent implementations absent and exclude 209 INFERRED rows. Secure membership must match the row's Game/Glue/both context for the active screen and a flag with a secure destination. Remove secure members when the context excludes the active screen. If the context includes the screen, ordinary `_G` cleanup must not remove an existing secure copy. Reconcile after bootstrap and screen registration. The additional actions for flags 6 and 7 remain UNKNOWN.
- [ ] VERIFIED — Retail 12.1.5.70688, R88 series decomp line 13165: each implemented flag-3 native C function has distinct ordinary and secure function objects, with the same handler and no captured values. **UNVERIFIED:** identity after later script replacement or deliberate aliasing.
- [ ] VERIFIED — Retail 12.1.5.70688 R88y.y2: install `string.concat`, `string.split`, and `string.join` from the existing `strconcat`, `strsplit`, and `strjoin` handlers before secure table-copy construction. Function values in the shallow-copied standard-library tables retain their shared identity. Keep `fastrandom` on the existing base `math.random` handler and out of the secure base; leave the simulator's current `math.random` handler unchanged and do not assert random outputs. Keep `table.join` absent.
- [ ] SIMULATOR POLICY — Retail 12.1.5: after bootstrap, the ordinary and secure `string` library tables preserve the verified alias identities. A simulator-only colon-split compatibility method must not replace Retail `string.split`; keep that helper on profiles without this R88y evidence.
- [ ] VERIFIED — Retail 12.1.5.70688 R88y.y3: on successful `SwapToGlobalEnvironment`, change the caller function environment to ordinary `_G` and return zero values. Preserve the conditional XML `addToSecureEnv` named-frame route. Do not add blanket global copying; `securecopy` behavior and frame-callback access remain UNKNOWN.
- [x] `C_AddOns.LoadAddOn` must attempt optional dependencies before required dependencies.
- [x] An optional dependency failure must not stop the requested addon from loading.

### R88t secure dependency and named publication routes

- [x] VERIFIED (Retail 12.1.5.70688, R88t): ordinary and secure addon loaded states are separate. A secure request passes its effective mode to dependencies, including dependencies without their own secure directive; optional dependencies load before required dependencies, and dependencies load before the current addon's files.
- [x] VERIFIED (Retail 12.1.5.70688, R88t): for a dependency with no secure directive and no ordinary loaded state, the loader requests ordinary mode and then secure mode. If a mode is already loaded, the request can return without running its files again.
- [ ] VERIFIED (Retail 12.1.5.70688, R88ac.ac4): a failed ordinary dependency request does not suppress the following secure request.
- [x] VERIFIED (Retail 12.1.5.70688, R88t): flag 3 publishes `AddSourceLocationExclude` and `table.count` into both environments with separate function objects; the ordinary and secure table libraries are separate.
- [x] VERIFIED (Retail 12.1.5.70688, R88v.v4; `secureenv_isolation::create_texture_source_location_skips_excluded_callsite`): calling the secure `AddSourceLocationExclude` object with a string returns zero values and changes the source selected by ordinary `CreateTexture`. The first template-free texture returns its saved file and line after the exclusion list changes; excluding the only file frame makes the next texture return an `UnknownFile:` location.
- [ ] VERIFIED (Retail 12.1.5.70688, R88v.v4): number arguments convert to text; input text stops at NUL; empty input adds nothing; duplicates are retained; secret denial and stack-overflow errors use the verified forms.
- [ ] VERIFIED (Retail 12.1.5.70688, R88v.v4): source matching is case-sensitive and uses a file-text suffix; positive lines wrap modulo 65,536 and non-positive lines become zero.
- [ ] VERIFIED (Retail 12.1.5.70688, R88y): publish the six implemented `C_ColorOverrides` members to both environments according to their VERIFIED flag-3 rows. Do not publish unimplemented members. Each ordinary/secure handler pair is distinct and callable against the shared override state.
- [x] VERIFIED (Retail 12.1.5.70688, R88t.t2): constant registration publishes separate `Constants` and `AccountStoreConsts` tables without a member flag. The four verified AccountStore values are 1, 3, 3139, and 2814 for their named members.
- [ ] VERIFIED (Retail 12.1.5.70688, R88t.t2): publish the native `Constants.InventoryConstants` table to secureenv without a per-member flag; the secure consumer must read `NumBagSlots = 4` and `NumReagentBagSlots = 1`. SIMULATOR POLICY: keep missing and compatibility constants ordinary-only unless native publication is established.
- [x] VERIFIED (Retail 12.1.5.70688, R88t): when ordinary and secure `UIParent` names are empty, both receive the same frame-table object through the Retail named-frame policy.
- [ ] VERIFIED (Retail 12.1.5.70688, R88t): the Retail UIParent bootstrap uses named-frame policy flag 32 and each destination preserves an existing non-nil name.

### R88ac GlobalStrings secure publication

- [ ] VERIFIED (Retail 12.1.5.70688, R88ac.ac2): each GlobalStrings record whose preparation succeeds and whose Flags contain all bits in the active mask writes its name and value to ordinary `_G` and the separately referenced shared secure table. Game uses mask 1; Glue uses mask 2.
- [ ] INFERRED simulator policy (R88ac.ac2): the committed Retail GlobalStrings PHF map stores no Flags, so Retail mirrors every map row that the existing registration writes to `_G`. Exact per-row Game/Glue filtering is not modeled; `RAID_TARGET_1` membership remains INFERRED.
- [ ] VERIFIED (Retail 12.1.5.70688, R88ac.ac3; supplement SHA-256 `107dad291c11c9a80949d7063e02d849bb394155663870db0631c96c54d9659b`): mirror the same existing simulator value to secureenv for `LE_GAME_*` numeric globals whose names occur in the Game/flag-3 CSV. Do not create missing constants.
- [ ] INFERRED simulator policy (R88ac.ac3): mirror other already-modeled Retail integer constants to secureenv. Native evidence identifies a 335-name shared numeric route, but the supplement has no per-name inventory for this set; do not infer membership or add constants.

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
- [Taint system](../wiki/systems/taint-system.md#dual-lua-environment) — secure environment publication and shallow library copies.

## Implementation inventory

- `crates/sim-toc/src/lib.rs` — parses `LoadManagers` metadata.
- `src/loader/addon_order.rs` — orders packaged startup addons and dependencies.
- `src/loader/mod.rs` — discovers startup roots and prerequisite addons.
- `src/loader/startup_addons.rs` — builds the ordered startup stream, registers follower declarations, and submits startup followers.
- `src/bin/wow_sim/addon_loading.rs` — registers packaged follower declarations before Blizzard startup, registers third-party declarations, and skips startup roots already loaded by followers.
- `src/c_api/c_addons.rs` — exposes startup follower processing through the runtime loader.
- `src/c_api/c_addons_runtime.rs` — resolves runtime dependencies and submits `LoadWith` followers.
- `src/c_api/helpers.rs`, `src/c_api/secure_publication.rs`, and generated `src/c_api/secure_publication_data.rs` — route implemented members through verified R88y flags; the source CSV and generator are `data/secure-publication/retail-12-1-5-70688.csv` and `tools/gen_r88y_secure_publication.py`.
- `src/loader/addon.rs` — orders ordinary and secure dependency requests.
- `src/loader/lua_file.rs`, `src/loader/xml_file.rs`, `src/lua_api/loader_env.rs`, `src/lua_api/globals/security/secure_env.rs` — execute secure files through the shared table and retain private replay only for selected replay paths and generated XML loader chunks.
- `src/lua_api/workarounds/temporary/secure_legacy_unit_globals.rs`, `src/lua_api/workarounds/mod.rs`, `src/lua_api/env_init/mod.rs` — apply the profile-tagged source-derived Unit-global compatibility copy at the former snapshot boundary.
- `src/lua_api/env_init/enums.rs` — registers `Enum` values and the Retail `Constants.*` secure-table publication path.
- `src/lua_api/globals/strings/mod.rs` — registers PHF GlobalStrings rows and integer constants; applies the inferred string and non-`LE_GAME_*` integer mirrors, plus the CSV-listed `LE_GAME_*` route.
- `src/lua_api/env_init/mod.rs`, `src/lua_api/env_runtime.rs` — install pre-copy library aliases and rerun verified publication after screen-specific registrations.
- `src/lua_api/globals/real/math_extensions.rs`, `src/lua_api/globals/real/string_extensions.rs`, and `src/lua_api/globals/real/table_extensions.rs` — register implemented library extensions; R88y publication routes their verified flags. The Retail secure `table.count` handler retains the separately verified R88t identity.
- `src/lua_api/workarounds/temporary/formatting_utility_defaults.rs` — applies the colon-split compatibility method outside Retail 12.1.5.
- `src/lua_api/globals/security/environment.rs`, `src/loader/tests/inline_script.rs` — implement and test the verified SwapToGlobalEnvironment and XML addToSecureEnv routes.
- `src/lua_api/globals/create_frame/mod.rs` — publishes the built-in `UIParent` name to secureenv.
- `src/lua_api/env_init/shared_bootstrap.lua` — retains the private `GetForbiddenObjectTable` closure while the verified flag-6 global is secure-only.
- `src/lua_api/globals/real/source_location.rs` — registers AddSourceLocationExclude and selects a caller file/line from existing Lua call frames.
- `src/lua_api/frame/methods/button_anchor_hierarchy/hierarchy.rs`, `src/lua_api/frame/methods/core_state/region.rs` — save and return texture source locations.
- `src/lua_api/state/sim_state.rs`, `src/widget/frame.rs`, `src/widget/frame_defaults.rs`, `src/widget/frame_size.rs` — hold shared exclusions and per-texture source metadata.
- `src/lua_api/workarounds/temporary/debug_environment_defaults.rs`, `src/lua_api/workarounds/temporary/color_defaults.rs` — publish only existing temporary handlers under the verified names.
- `src/lua_api/state/sim_state.rs` — holds ordered registered follower declarations and the resolved target-to-follower index.
- `src/lua_api/state.rs` — initializes the registration vector and follower index.
- `src/lua_api/state/ui_reload.rs` — clears the follower index on UI reload.

## Tests asserting this spec

- `src/loader/tests/screen_selection.rs`
- `tests/addon_runtime_dependencies.rs`
- `src/c_api/c_addons_runtime_tests.rs`
- `tests/secureenv_isolation.rs` — separate loaded states, dependency request order/failure continuation, verified Retail `LE_GAME_*` secure values, and the one-file source-location consumer.
- `tests/blizzard_frame_xml_loads.rs` — the original `UIErrorsFrame` consumes a modeled Retail `LE_GAME_*` numeric value.
- `tests/blizzard_communities_secure_loads.rs` — verifies `CommunitiesCreateDialog.DescriptionFrame` `InputScrollFrame` setup and that `debug` stays out of `__secureenv`.
- `tests/blizzard_private_auras_ui_loads.rs` — the original FrameFactory/Pools, AccountUtil, ColorManager, GameRulesUtil, RaidWarning XML, and PrivateAurasTooltip consumers.
- `src/c_api/secure_publication.rs` — verified-only R88y routing, absent implementations, and late flag-3 registration after screen changes.
- `src/c_api/helpers.rs` and `src/lua_api/globals/security/secure_env.rs` — separate flag-3 C-function objects; the latter also tests pre-copy library aliases and shallow-copy identity.
- `src/c_api/c_color_overrides.rs` and `tests/secureenv_isolation.rs` — six `C_ColorOverrides` handlers and `C_Timer.NewTimedSignalMap` publication.
- `src/lua_api/workarounds/temporary/formatting_utility_defaults.rs` — verifies defaults reapplication preserves Retail `string.split` identity.
- `src/lua_api/workarounds/temporary/secure_legacy_unit_globals.rs::tests` — checks profile-tagged Unit registrations in ordinary `_G`; it does not assert secureenv membership or native publication.
- `src/lua_api/globals/security/environment.rs` — successful `SwapToGlobalEnvironment` changes caller environment and returns zero values.
- `src/lua_api/globals/real/table_extensions.rs` — separate ordinary/secure `table.count` handlers.
- `src/loader/tests/inline_script.rs` — conditional XML `addToSecureEnv` named-frame publication.

## Known gaps (current cycle)

- [ ] UNKNOWN: PTR 12.1.7.70668 does not establish whether declaration parsing feeds the child-list resolver. The simulator applies the inferred resolver model; PTR tests do not assert parser linkage or duplicate suppression.
- [ ] VERIFIED (R88t, native) but untested in the simulator: an ordinary dependency-request failure must not suppress the following secure request. No evidence-backed input currently makes only the ordinary request fail; a Lua file diagnostic is not proof of that request failure.
- [ ] INFERRED (R88t): when neither mode was loaded and both default file paths complete, each default path executes in its requested environment. Do not claim this exact per-environment count as native parity.
- [ ] UNKNOWN: the native cleanup caller for the shared exclusion list is not established; the simulator retains entries for the lifetime of `SimState`. The two-file caller/wrapper comparison remains INFERRED and is not asserted.
- [ ] rilua exposes no per-argument secret mode and no fallible stack reservation; AddSourceLocationExclude therefore does not reproduce the verified mode-zero secret error or stack-overflow path.
- [ ] The source test invokes the secure function object from an ordinary add-on and observes the effect through ordinary `CreateTexture`; it does not invoke the ordinary function object from secure add-on code.
- [ ] SOURCE-DERIVED (VERIFIED simulator source at master `725fb6e3a`): the compatibility table records the 173-name union of Unit-prefixed global functions registered before the former snapshot across all seven client profiles. Each name records its registered profiles, compatibility-copy profiles, and registrar file:line. Copy only for the active compatibility profile, and only when `_G` holds a function; skip absent names. The six VERIFIED R88y Unit rows are excluded from compatibility copying on Retail, where that publisher runs, and retained on profiles without that publisher. The profile guard checks `_G` registration only; it does not assert secureenv membership. Native secure publication remains UNKNOWN (R88ac.ac1, R88af.1/.2).
- [ ] UNKNOWN: native transitive copying of other ordinary globals into secureenv. The simulator preserves only the named Unit compatibility set; the direct dependency helper does not add a blanket-copy route.
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
