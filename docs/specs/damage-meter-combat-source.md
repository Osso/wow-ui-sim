# DamageMeter explicit structure input

Bounded contract for `structures-DamageMeterCombatSource-639` (12.0.5). Cached Retail `Blizzard_APIDocumentationGenerated/DamageMeterDocumentation.lua` is the shape/security source; [audit](../wiki/investigations/patch-12-0-5-api-audit.md#damagemeter-input-only--pending-red) tracks proof. This change supplies input structs and tests, not a query producer. The existing Lua workaround still publishes fabricated Player/Companion data and must remain unchanged until parent observes actual behavioral RED.

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

- [ ] Four session/detail getters carry `SecretWhenInCombat = true` and `SecretArguments = "AllowedWhenUntainted"`. Duration has `AllowedWhenUntainted`. Do not clear caller taint, unwrap secret selectors, or declassify output to satisfy fixture assertions.
- [ ] Aggregate `name` is `ConditionalSecret`; `classFilename`, `specIconID`, `isLocalPlayer`, `deathRecapID`, `classification` are `NeverSecret`. Spell-detail `unitClassFilename`, `classification`, `specIconID` are `NeverSecret`. Other aggregate/spell/detail fields lack a disclosure exemption in this file; absence is not permission to expose them.
- [ ] Plain Rust strings/numbers are input storage, **not a security representation or Lua disclosure contract**. Tests run outside combat, enumerate field keys without comparing sensitive values, and compare populated aggregate/detail values only for `NeverSecret` fields. List name/ID/duration assertions concern the separately documented session-list structure. No blanket readable/untainted result assertion.

## How it works

- [Bounded audit/proof](../wiki/investigations/patch-12-0-5-api-audit.md#damagemeter-input-only--pending-red).
- [C API architecture](../lua-api.md).

## Implementation inventory

- `src/c_api/c_damage_meter.rs`: plain typed input structs; no registration, lookup, conversion, reset, or output logic.
- `src/c_api/mod.rs`: exposes input module under `retail-12-0-5`.
- `src/lua_api/state/sim_state.rs`, `src/lua_api/state.rs`: per-environment empty-default input field/constructor.
- `src/lua_api/workarounds/temporary/damage_meter_state.rs`: unchanged seeded production bootstrap; obsolete tests migrated only. Parent must replace producer after actual RED, with no retained fallback.

## Tests asserting this spec

All six new tests live in existing `tests/startup_targeted_regressions/damage_meter.rs`, under `retail-12-0-5`, no new Cargo target:

- `damage_meter_empty_input_has_no_fabricated_records`: empty input vs empty query snapshots/availability.
- `damage_meter_explicit_aggregate_and_detail_shapes_are_distinct`: concrete list/session/source/spell/detail fixture; both ID/type APIs and NeverSecret values.
- `damage_meter_missing_selectors_do_not_alias_fixture_data`: zero/unknown IDs, absent Expired binding, every unsupplied meter, GUID/creature matching and no-selector case.
- `damage_meter_explicit_meter_input_and_optional_fields_are_preserved`: independent HealingDone aggregate input, optional omissions, no derived detail.
- `damage_meter_reset_clears_inputs_without_changing_availability` and `damage_meter_queries_return_independent_nested_snapshots`: reset/repopulate, actual backing-state emptiness, nested consumer mutation, retained snapshots and environment isolation.

Replacement coverage: old workaround `patch_12_0_0_damage_meter_fields_and_reset`/`installs_seeded_damage_meter_sessions`, three `tests/system_api_seeded.rs` DamageMeter tests, seeded startup detail test and standalone `tests/damage_meter_zero_id.rs` are consolidated here. Fabricated values, undocumented helper expectations and sensitive arithmetic/readability assertions are intentionally not retained. `tests/blizzard_damage_meter_loads.rs` remains unchanged. Earlier profile acceptance is not claimed.

Proof ledger: **unrun**. No compilation, RED, GREEN, checks, delegation, or push in this slice; only scoped rustfmt and commit. Parent must compile fixtures, run actual RED against current producer, identify behavioral failures (not merely compile errors), then implement production publication. No audit row closure or native parity credit.

## Known gaps (current cycle)

- [ ] Parent compilation and real RED required before producer work.
- [ ] C API producer/registration and removal of seeded Lua implementation pending.
- [ ] Combat query secrecy, conditional name disclosure and secret argument access cannot safely be represented by these plain structs alone. Existing applicable VM/C_Secrets enforcement must be established before combat publication; unresolved security must block/defer that path, never declassify it. No secret/tainted caller acceptance evidence here.
- [ ] Key presence alone does not prove sensitive values are correct or safely readable; secret-aware value proof remains pending.

## Out of scope

Combat ingestion/full simulation, sorting/ranking, derived totals/rates, event dispatch, persistence, UI/render acceptance, non-12.0.5 profile verification, invalid argument policy, native-client probing and whole DamageMeter API completion.
