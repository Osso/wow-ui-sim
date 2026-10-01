# DamageMeter explicit structure input

Bounded contract for `structures-DamageMeterCombatSource-639` (12.0.5). Cached Retail `Blizzard_APIDocumentationGenerated/DamageMeterDocumentation.lua` is the shape/security source; [audit](../wiki/investigations/patch-12-0-5-api-audit.md#damagemeter-input-only--pending-red) tracks proof. Parent observed actual behavioral RED at `97f7edd2d`. C API-owned queries now serialize explicit inputs outside combat; the seeded Lua producer is removed. Implementation is source-only pending parent compilation/GREEN.

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

- `src/c_api/c_damage_meter.rs`: input structs, explicit type/ID/meter/source lookup, availability/list/duration, reset and combat-publication block. Registrations use the normal C API utility bootstrap under `retail-12-0-5`.
- `src/c_api/c_damage_meter/snapshot.rs`: fresh nested tables rooted on the VM stack during serialization; aggregate/source-detail fields stay separate, optional fields omitted.
- `src/lua_api/state/sim_state.rs`, `src/lua_api/state.rs`: per-environment empty-default input field/constructor.
- Seeded `src/lua_api/workarounds/temporary/damage_meter_state.rs` and its module/bootstrap registrations are removed. No fallback or synthetic rows remain. Undocumented seed-only `GetDamageMeterEntries` is not retained; compatibility Current-ID helper is retained as specified. Earlier profiles receive no replacement seed; their runtime acceptance is unverified.

## Tests asserting this spec

All nine tests live in existing `tests/startup_targeted_regressions/damage_meter.rs`, under `retail-12-0-5`, no new Cargo target:

- `damage_meter_empty_input_has_no_fabricated_records`: empty input vs empty query snapshots/availability.
- `damage_meter_explicit_aggregate_and_detail_shapes_are_distinct`: concrete list/session/source/spell/detail fixture; both ID/type APIs and NeverSecret values.
- `damage_meter_missing_selectors_do_not_alias_fixture_data`: zero/unknown IDs, absent Expired binding, every unsupplied meter, GUID/creature matching and no-selector case.
- `damage_meter_explicit_meter_input_and_optional_fields_are_preserved`: independent HealingDone aggregate input, optional omissions, no derived detail.
- `damage_meter_reset_clears_inputs_without_changing_availability` and `damage_meter_queries_return_independent_nested_snapshots`: reset/repopulate, actual backing-state emptiness, nested consumer mutation, retained snapshots and environment isolation.
- Three additional unrun guard fixtures: `damage_meter_ambiguous_partial_selectors_return_empty_details`, `damage_meter_secret_selectors_reject_without_unwrapping`, `damage_meter_combat_publication_is_explicitly_blocked`. These assert bounded matching/rejection/deferred publication, not native secrecy parity or broad security acceptance.

Replacement coverage: old workaround `patch_12_0_0_damage_meter_fields_and_reset`/`installs_seeded_damage_meter_sessions`, three `tests/system_api_seeded.rs` DamageMeter tests, seeded startup detail test and standalone `tests/damage_meter_zero_id.rs` are consolidated here. Fabricated values, undocumented helper expectations and sensitive arithmetic/readability assertions are intentionally not retained. `tests/blizzard_damage_meter_loads.rs` remains unchanged. Earlier profile acceptance is not claimed.

Proof ledger: parent default-retail compile at `97f7edd2d` reported exit 0 (434s); `/tmp/patch-12.0.5-batch17-red-run.log` observes six failures, zero passes, finite test body 5.77s. JSON records process exit 101, 8.246s including overhead, exact revision and binary hash. Failures reach seed-vs-empty, missing explicit session and wrong-shape boundaries; later assertions were not all reached. This implementation runs only scoped rustfmt and commits; no builds, checks, test execution, delegation or push. Nine fixtures await parent GREEN compilation/execution. No audit row closure or native parity credit.

## Known gaps (current cycle)

- [ ] Parent compilation/GREEN of the current producer and nine fixtures is pending; only predecessor six-case RED is observed.
- [ ] Combat query secrecy, conditional name disclosure and secret argument access cannot safely be represented by these plain structs alone. Inspected `c_secrets.rs` models stat/cooldown/aura policies, not DamageMeter; `table_builder.rs` registration installs Rust closures without documentation annotations. No applicable automatic `SecretWhenInCombat` enforcement was found in the VM. Combat getters explicitly error until a DamageMeter field-level disclosure contract exists. Secret selectors remain rejected by typed extraction, not authenticated/unwrapped. No tainted-caller acceptance evidence here.
- [ ] Key presence alone does not prove sensitive values are correct or safely readable; secret-aware value proof remains pending.

## Out of scope

Combat ingestion/full simulation, sorting/ranking, derived totals/rates, event dispatch, persistence, UI/render acceptance, non-12.0.5 profile verification, invalid argument policy, native-client probing and whole DamageMeter API completion.
