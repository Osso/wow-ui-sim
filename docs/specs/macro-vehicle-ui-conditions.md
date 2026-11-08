# Macro vehicle-UI conditions

`SecureCmdOptionParse` must distinguish the player's `vehicleui` fact from the selected unit snapshot's `unithasvehicleui` fact. State lives in `src/lua_api/game_data.rs`; condition evaluation lives in `src/lua_api/globals/security/cmd_option.rs`. See the [historical 3.3.0 audit](../wiki/investigations/patch-3-3-0-api-audit.md).

Historical wording distinguishes player and macro-target predicates; it does not establish native defaults, absent-target behavior or negation grammar. Existing NPC/admin/party snapshots initialize the new fact to false as explicit simulator policy, not native evidence. A snapshot of the actual player copies the player's existing fact; arbitrary targets do not inherit player state or hostility.

## What it must do

- [x] Player true, explicit target false: `[vehicleui]yes;no` selects `yes`; `[@target,unithasvehicleui]yes;no` selects `no`.
- [x] Player false, explicit target true: those clauses select `no` and `yes`.
- [x] Both explicit facts true: both clauses select `yes`.
- [x] Both explicit facts false: both clauses select `no`.
- [x] Explicit player/focus and implicit-target clauses use their independently supplied facts; a focus selector does not change the player-only `vehicleui` predicate.

## How it works

- [Historical contract and coverage](../wiki/investigations/patch-3-3-0-api-audit.md)
- [Lua API architecture](../lua-api.md)

## Implementation inventory

- `src/lua_api/game_data.rs`: explicit `TargetInfo.has_vehicle_ui` snapshot input; clones retain it.
- `src/lua_api/globals/admin_party_target_helpers.rs` and `admin_api/units.rs`: current and legacy admin target constructors initialize the simulator fact.
- `src/lua_api/globals/targeting_verbs.rs`: player/NPC/party target snapshot initialization.
- `src/lua_api/globals/security/cmd_option.rs`: `vehicleui` reads player state; `unithasvehicleui` reads selected player, target or focus facts through the existing selector path. Grammar and separate `UnitHasVehicleUI` API are unchanged.

## Tests asserting this spec

- `tests/cmd_option_vehicle_ui.rs`: four player/target cases plus one selected-player/focus/implicit-target case. Constant target identity/hostility; five common-producer cases are now eligible under every supported client profile; newly enabled profiles are not yet compiled/executed proof. This removes a test-only Retail gate; it adds no production behavior or native/older-profile acceptance. Executed Retail RED at52fc18f4e: five unique cases, one false/false control PASS and four predicate FAILs (exit101). Selected-unit case fails on its first supplied row; later three rows were unreached, not sixteen RED outcomes. Initial E0603/private `game_data` import compile failure is retained separately; corrected public `state::TargetInfo` export compiled. Receipt-directory setup failure was zero-launch, corrected by whole-text metadata copies without rehashing.
- `tests/cmd_option_selected_unit.rs`: existing selected-clause/unit and unchanged cached slash-handler preservation scope; not newly executed here.

## Current bounded runtime follow-up — 2026-10-14

[Public historical Store/macro packet](../../data/patch-api/evidence/store-macro-20261014/README.md) retains preproducer five-case 1 PASS/4 FAIL (later selected rows unreached), producer five-case PASS with all four fifth-case rows reached, and separate 2/2 selected-unit preservation. Compile/setup/admission failures remain separate. Accepted 170/170 copy and bounded known-literal privacy gate adds no executable/source authentication, affected-profile/PTR, native/render or full-parent acceptance; current item/display/UI proof is excluded.

The full Retail macro / Store artifact verification report records source `a6b57273246430131060b64d2b02e6d50baea9a6`: `cmd_option_vehicle_ui::` **5/5 PASS**, exit0, and `cmd_option_selected_unit::` **2/2 PASS**, exit0. All four authored rows in the fifth macro case reached their explicit-player, focus, implicit-target and player-only `vehicleui` assertions; eight completed environment startups corroborate four named cases plus four loop rows. Macro and selected-unit stderr emit no Lua error records. This checks the five literal requirements above, not broader token or native semantics.

The report associates selected archived test text and reused receipt/seal STAT metadata with the runtime; it supplies no new source-to-binary byte authentication. Historical RED remains five unique cases, 1 PASS/4 FAIL, with only the first selected-unit row reached. Source/readability already have their separate source gate. Producer compilation remains 741 source JSON records, successful build, zero compiler-message records and six manifest deprecation warnings; neither compilation nor this runtime report establishes final producer formatting/checks or affected-profile acceptance.

## Known gaps (historical pre-GREEN checkpoint)

The pending items below preserve the earlier checkpoint, not current runtime status; the follow-up above supersedes only targeted Retail GREEN and selected-unit preservation.

- [x] Compile passive state/fixture and execute five-case behavioral RED before adding the condition producer. Two unchanged selected-clause/cached slash-handler preservation cases passed at the same preproducer revision.
- [ ] Verify the two-arm producer with targeted GREEN and fresh selected-unit preservation, full diagnostics, formatting/checks and independent gate.
- [ ] Pet/vehicle/party/raid/self/softinteract/enemy tokens have no newly established vehicle-fact resolution in this bounded source seam; broader selected-token coverage remains open.
- [ ] Verify affected profiles, formatting/checks, readability and parent acceptance separately.

## Remaining current gates

- [x] Independent PTR gate at compiled `bc5cecffb`: vehicle5/5 and selected-unit2/2, all four fifth-case rows reached; Store2 unique/item4/corrected bag1 also pass. Separate normal public Store lifecycle passes visibility/name/containment and CLEAN0errors. Store cleanup retains one AuraContainerUtil failure reported twice; six manifest warnings remain. This is bounded PTR proof, not all-startup-clean, native/render or full-parent acceptance.
- [ ] [Current revision/gate matrix](store-product-group-shape.md#seven-profile-saved-runtime-matrix--2026-10-14): vehicle5 runtime PASS on all seven profiles. Selected-unit2 preservation is Retail/PTR only. Mists overall/authentication remains PARTIAL; test eligibility or zero selected tests is not preservation. Final HEAD formatting/default check and synthetic sealer proof are source-only, not runtime/native/full-parent acceptance.
- [ ] Broader selected-token resolution; native, negation, default and absent-target semantics remain unproved. Existing selected-unit preservation is not new vehicle-fact proof for these boundaries.

## Out of scope

This bounded slice does not establish absent-target/negation/native/default semantics, full macro grammar, native UI, or a change to the separate `UnitHasVehicleUI` API. No vendor edits, getter overrides or inferred vehicle state from hostility.
