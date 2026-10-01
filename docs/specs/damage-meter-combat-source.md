# DamageMeter explicit structure input

Bounded contract for `structures-DamageMeterCombatSource-639` (12.0.5). Cached Retail `Blizzard_APIDocumentationGenerated/DamageMeterDocumentation.lua` is the shape/security source; [audit](../wiki/investigations/patch-12-0-5-api-audit.md#damagemeter-input-only--pending-red) tracks proof. Parent observed actual behavioral RED at `97f7edd2d`. C API-owned queries now serialize explicit inputs outside combat; the seeded Lua producer is removed. Bounded independent proof now confirms nine fixtures on default Retail and historical Retail 12.0.0; the source row remains PARTIAL, not closed.

## What it must do

### Source-grounded structures

- [ ] `GetAvailableCombatSessions` returns a non-nil sequence of `DamageMeterAvailableCombatSession`: required numeric `sessionID`, required cstring `name`, optional numeric `durationSeconds`.
- [ ] `GetCombatSessionFromID/FromType` returns non-nil `DamageMeterCombatSession`: required `combatSources` sequence of `DamageMeterCombatSource`, required numeric `maxAmount`/`totalAmount` (documented defaults 0), optional numeric `durationSeconds`. Session ID/name are list fields, not fields of this structure.
- [ ] Each aggregate source has optional WOWGUID `sourceGUID`, optional numeric `sourceCreatureID`, required cstring `name`/`classFilename`/`classification`, required fileID `specIconID`, required numeric `totalAmount`/`amountPerSecond`/`deathRecapID`/`deathTimeSeconds`, required bool `isLocalPlayer`, required enum `sourceDisplayType`, optional cstring `factionGroup`. No `combatSpells` or `maxAmount` field belongs to the aggregate structure.
- [ ] `GetCombatSessionSourceFromID/FromType` returns non-nil `DamageMeterCombatSessionSource`: required `combatSpells` sequence, required numeric `maxAmount`/`totalAmount` (documented defaults 0). No aggregate identity/classification fields belong to this result.
- [ ] Each spell has required numeric `spellID`/`totalAmount`/`amountPerSecond`/`overkillAmount`, cstring `creatureName`, bool `isAvoidable`/`isDeadly`, and one `combatSpellDetails` structure (not a target sequence). That structure has required cstring `unitName`/`unitClassFilename`/`classification`, bool `isPet`/`isMob`, numeric `amount`, fileID `specIconID`.

### Explicit simulator policy — inferred, not native-verified

- [ ] Input defaults disabled with empty failure-reason string, no available sessions, no type bindings, no aggregate or detail records. Availability is independent of combat data; `IsDamageMeterAvailable` returns bool/string, matching both non-nilable documented returns. Native initial availability/reason is unknown.
- [ ] Session/meter aggregates are keyed by explicit `(session ID, meter enum value)`; detail inputs are separately keyed by session/meter and optional source GUID/creature ID. No derivation from player identity, combat simulation, or enum validity. All fixture numbers/names are deliberate test input, not native facts.
- [ ] Session types map to IDs only through explicit input. `Overall=0`, `Current=1`, `Expired=2` are repository enum ordering, not implicit aliases. Compatibility `GetCurrentCombatSessionID` reads only the Current binding and returns nil when absent; this helper is registered today but absent from inspected documentation.
- [ ] Missing IDs (including zero), type bindings, meter inputs, or source matches return fresh empty result wrappers with zero documented totals and empty sequences; session duration is omitted. Wrappers satisfy documented non-nilability without creating stored/listed sessions or sources. This missing-selector policy is an inference, not a claim that every valid enum denotes a session. Invalid enum/error behavior is outside this slice.
- [ ] Source selectors constrain every non-nil argument. GUID-only and creature-only lookup require a unique matching detail input; no selector or ambiguous match yields an empty detail wrapper. No aggregate row is used as a detail fallback.
- [ ] `GetSessionDurationSeconds(sessionType)` uses the bound available-session row's optional duration; absent row/duration yields nil, permitted by documentation. No undocumented second-argument ID lookup.
- [ ] Reset clears list entries, type bindings, aggregates and details; preserves availability/reason. Returns no values, repeat reset harmless, explicit input can repopulate afterward. Clearing all session data is documented; preservation/idempotence are simulator policy. Events are not asserted here.
- [ ] Every query materializes an independent nested snapshot. Consumer mutation and reset cannot alter already returned snapshots or retained Rust input; separate environments do not share input. Cached `DamageMeterSessionWindowMixin:BuildDataProvider` decorates aggregate rows (`maxAmount`, `sessionTotalAmount`, `index`), grounding the need for independent mutable snapshots. Exact snapshot policy is inferred.

### Security boundary

- [ ] Four session/detail getters carry `SecretWhenInCombat = true` and `SecretArguments = "AllowedWhenUntainted"`. Duration has `AllowedWhenUntainted`. Do not clear caller taint, unwrap secret selectors, or declassify output to satisfy fixture assertions. Current bounded implementation rejects all four getters while `player.in_combat`, including missing-selector queries, with an explicit unmodeled-secrecy error. This deliberately defers combat publication; it is not native return/error behavior. Typed `FromStack` extraction rejects secret selectors even for untainted callers; implementing documented secure access remains deferred.
- [ ] Aggregate `name` is `ConditionalSecret`; `classFilename`, `specIconID`, `isLocalPlayer`, `deathRecapID`, `classification` are `NeverSecret`. Spell-detail `unitClassFilename`, `classification`, `specIconID` are `NeverSecret`. Other aggregate/spell/detail fields lack a disclosure exemption in this file; absence is not permission to expose them.
- [ ] Plain Rust strings/numbers are input storage, **not a security representation or Lua disclosure contract**. Tests run outside combat, enumerate field keys without comparing sensitive values, and compare populated aggregate/detail values only for `NeverSecret` fields. List name/ID/duration assertions concern the separately documented session-list structure. No blanket readable/untainted result assertion.

## How it works

- [Bounded audit/proof](../wiki/investigations/patch-12-0-5-api-audit.md#damagemeter-input-only--pending-red).
- [C API architecture](../lua-api.md).

## Implementation inventory

- `src/c_api/c_damage_meter.rs`: input structs, explicit type/ID/meter/source lookup, availability/list/duration, reset and combat-publication block. Registrations use the normal C API utility bootstrap unconditionally, preserving the removed bootstrap's all-profile scope.
- `src/c_api/c_damage_meter/snapshot.rs`: fresh nested tables rooted on the VM stack during serialization; aggregate/source-detail fields stay separate, optional fields omitted.
- `src/lua_api/state/sim_state.rs`, `src/lua_api/state.rs`: per-environment empty-default input field/constructor.
- Seeded `src/lua_api/workarounds/temporary/damage_meter_state.rs` and its module/bootstrap registrations are removed. No fallback or synthetic rows remain. Undocumented seed-only `GetDamageMeterEntries` is not retained; compatibility Current-ID helper is retained as specified. All profiles receive the same empty-backed model, including historical Retail 12.0.0 and WowForever; no seed is restored. This preserves registration scope, not native cross-profile semantics or runtime acceptance.

## Tests asserting this spec

All nine tests live in existing `tests/startup_targeted_regressions/damage_meter.rs`, available without an epoch/profile gate in the grouped integration target; no new Cargo target:

- `damage_meter_empty_input_has_no_fabricated_records`: empty input vs empty query snapshots/availability.
- `damage_meter_explicit_aggregate_and_detail_shapes_are_distinct`: concrete list/session/source/spell/detail fixture; both ID/type APIs and NeverSecret values.
- `damage_meter_missing_selectors_do_not_alias_fixture_data`: zero/unknown IDs, absent Expired binding, every unsupplied meter, GUID/creature matching and no-selector case.
- `damage_meter_explicit_meter_input_and_optional_fields_are_preserved`: independent HealingDone aggregate input, optional omissions, no derived detail.
- `damage_meter_reset_clears_inputs_without_changing_availability` and `damage_meter_queries_return_independent_nested_snapshots`: reset/repopulate, actual backing-state emptiness, nested consumer mutation, retained snapshots and environment isolation.
- Three additional guard fixtures: `damage_meter_ambiguous_partial_selectors_return_empty_details`, `damage_meter_secret_selectors_reject_without_unwrapping`, `damage_meter_combat_publication_is_explicitly_blocked`. These assert bounded matching/rejection/deferred publication, not native secrecy parity or broad security acceptance.

Replacement coverage: old workaround `patch_12_0_0_damage_meter_fields_and_reset`/`installs_seeded_damage_meter_sessions`, three `tests/system_api_seeded.rs` DamageMeter tests, seeded startup detail test and standalone `tests/damage_meter_zero_id.rs` are consolidated here. Fabricated values, undocumented helper expectations and sensitive arithmetic/readability assertions are intentionally not retained. `tests/blizzard_damage_meter_loads.rs` remains unchanged. Historical Retail 12.0.0 has bounded fixture acceptance only; no other-profile execution acceptance.

### Reconciled bounded proof — 2026-10-01

Producer `1a9fcd1ec`, unconditional registration repair `e38d98a89`, prerequisite `LuaApiMut` import repair `c1dce16c3`, rooted host-secret fixture `5b0644b33`. Independent reports: `/tmp/patch-12.0.5-damage-meter-independent-proof.md` and `/tmp/patch-12.0.5-damage-meter-followup-proof.md`; parent chronology: `/tmp/patch-12.0.5-proof-ledger.md`.

| Capability | Proof | Remaining limit |
|---|---|---|
| Empty disabled input; separate list/aggregate/detail/spell/unit-detail shapes, optional omissions and explicit HealingDone input | Default and historical 12.0.0 each 9 PASS | Sensitive values not blanket-compared; empty zero totals source-reviewed |
| ID/type/meter/GUID/creature routes, missing/ambiguous selectors | Both nine-case runs PASS | Invalid argument/native matching policies unclaimed |
| Reset/repopulate, nested mutation, retained snapshots and environment isolation | Both runs PASS | General GC snapshot stress absent |
| Five host secrets survive two full GCs; six selectors reject; four combat queries explicitly error, including after reset | Corrected fixture PASS on both builds | Rejection/deferred publication, not AllowedWhenUntainted or native secrecy parity |
| Unconditional registration/state | Source review plus default/historical runtime | WowForever/classic/PTR/all-profile execution absent |

Actual prerequisite RED `97f7edd2d`: 0/6, exit 101, body 5.77s; downstream assertions not all reached. Historical repair build first failed E0599; import-fixed run was 8/9 because historical `secretwrap` is an identity alias. Corrected host fixtures supply actual secret wrappers, root before allocations and verify secrecy after GC. Historical-host-secrets and default-final builds/runs at `5b0644b33` exit 0, each 9/9. Saved run/build JSON, revisions and executable hashes inspected independently; no test rerun for this reconciliation.

Prior 17 controls remain narrow original evidence: three discovery/TOC/template and fourteen seeded-system assertions PASS at `1a9fcd1ec`, unchanged producer/serializer/control files. Old binary overwritten; not current whole-runtime acceptance. Discovery log contains MacroFrame `string.find` diagnostics, so not zero-error full-addon UI proof.

Follow-up default `cargo fmt --check` and `cargo check` each exit 0, no warnings, captured unchanged-source snapshot only; `/tmp/patch-12.0.5-damage-meter-followup-gates.json` owns exact scope. Later unrelated housing edits excluded. Parent default-final startup reports exit 0, `[]`, 4.173s; saved `/tmp/patch-12.0.5-batch17-default-final-startup-run.json`, not independent startup execution. Later intersecting changes invalidate only affected proof scope.

`structures-DamageMeterCombatSource-639` remains **PARTIAL**: shapes and explicit snapshot behavior covered; sensitive value correctness/disclosure, combat secrecy and native semantics unresolved. No combat/native/all-profile/full-addon UI or whole-row/page closure claim.

### Registration-scope regression after `1a9fcd1ec`

Source inspection confirms predecessor `src/lua_api/workarounds/temporary/mod.rs` declared the seed module without a cfg and `src/lua_api/workarounds/mod.rs:195` invoked it unconditionally. The replacement module, state field/default and registration incorrectly required `retail-12-0-5`. Historical `--no-default-features --features profile-retail,retail-12-0-0` and `--no-default-features --features client-wowforever` do not enable that epoch. Those gates and the fixture gate are removed, without changing Cargo features, query policies or restoring synthetic data/fallbacks. Parent can select `startup_targeted_regressions::damage_meter::damage_meter_` in `--test integration` for either feature set; nine tests now have default and historical 12.0.0 runtime proof as recorded above. Other profiles remain source-only. Combat publication, conditional disclosure and AllowedWhenUntainted access remain unresolved exactly as above.

## Known gaps (current cycle)

- [x] Bounded default and historical 12.0.0 nine-fixture proof reconciled; no broader profile acceptance.
- [ ] Combat query secrecy, conditional name disclosure and secret argument access cannot safely be represented by these plain structs alone. Inspected `c_secrets.rs` models stat/cooldown/aura policies, not DamageMeter; `table_builder.rs` registration installs Rust closures without documentation annotations. No applicable automatic `SecretWhenInCombat` enforcement was found in the VM. Combat getters explicitly error until a DamageMeter field-level disclosure contract exists. Secret selectors remain rejected by typed extraction, not authenticated/unwrapped. No tainted-caller acceptance evidence here.
- [ ] Key presence alone does not prove sensitive values are correct or safely readable; secret-aware value proof remains pending.

## Out of scope

Combat ingestion/full simulation, sorting/ranking, derived totals/rates, event dispatch, persistence, UI/render acceptance, native cross-profile parity, invalid argument policy, native-client probing and whole DamageMeter API completion.
