# Action spell slot identifiers

Batch51 covers exactly 12.0.5 source rows **229 FindSpellActionButtons** and **243 HasSpellActionButtons**, each changing argument 1 from `number` to `SpellIdentifier`. This is an **INFERRED bounded simulator model**, not native parity. Producer: [`src/c_api/c_action_bar_spell_slots.rs`](../../src/c_api/c_action_bar_spell_slots.rs), using existing assignments and the shared public identifier boundary. Implementation is committed separately from parent-owned GREEN and acceptance; requirements remain unchecked. [Lua API architecture](../lua-api.md) describes the runtime surface.

## What it must do

### Existing effective assignment model — INFERRED

- [ ] Query only current direct spell assignments in `SimState.action_bars: HashMap<u32, u32>` (slot → spell ID), matching the exact resolved ID; require positive result slots. No new model or catalog.
- [ ] Respect current `GetActionInfo` effective-kind priority: outfit before macro before spell. A same-slot macro/outfit hides the underlying spell mapping; macro-only slot 8 creates no spell assignment or inferred macro spell.
- [ ] Return slot indexes, never spell IDs or registered UI frame IDs. Existing `action_ui_buttons: Vec<(u64, u32)>` stores frame/action associations; registration alone creates no assignment.
- [ ] Observe alias changes, `PutActionInSlot(source, target)`, and host assignment clear/replace immediately. Queries remain read-only over bars, macros, outfits, UI tuples and aliases, isolated across environments.
- [ ] Unknown spell 7003, known-but-unslotted spell 7004, unresolved strings and empty assignments miss without inventing catalog data.

### Identifier boundary — INFERRED

- [ ] Use existing `c_spell::read_public_spell_identifier_at`: required public UTF-8 STRING or finite integral NUMBER in inclusive `u32` bounds. Validate before consulting aliases, including potential fractional/range/lossy-UTF-8 coerced keys.
- [ ] Preserve shared alias-first resolution: explicit case-normalized name, full colored link alias overriding embedded ID, numeric alias overriding numeric identity, seeded numeric string only. No generic name/link parsing, numeric-string coercion or recursive alias inference.
- [ ] Reject missing/nil, booleans, tables, functions, threads, actual Frame objects, nonfinite/fractional/negative/out-of-range numbers and invalid UTF-8. Recover with a subsequent valid public call without input mutation.
- [ ] Reject authentic host-created VM secret NUMBER (known/miss), STRING (name/link/miss) and secret-wrapped actual Frame before inspecting payloads, even in secure callers. Preserve globally/list/stack-rooted GC identity and secrecy, input snapshots and caller taint; public calls recover in secure and tainted contexts.
- [ ] Ordinary tainted public queries work without clearing/changing stack taint. This does **not** establish native `AllowedWhenTainted` secret permissions.

### Result contract — bounded policy

- [ ] Find returns exactly one fresh public table per call, a dense Lua array of exact matching positive current slot indexes, without duplicates or metadata. Tests compare members, not undocumented native order; ascending internal determinism is permissible, not required.
- [ ] A miss returns exactly one fresh empty table under the chosen existing simulator policy. Cached `MayReturnNothing=true` permits native zero-result cases whose conditions remain unknown; this contract does not claim all native misses return tables.
- [ ] Has returns exactly one public boolean agreeing with whether the same resolved identifier has any effective matching slot.
- [ ] Mutating returned tables or caller-owned identifier/expectation containers never changes backing inputs or other returned tables, including fresh empty results.

## How it works

- [Lua API architecture](../lua-api.md)
- [Action text contract](action-text.md)
- [Client profiles](client-profiles.md)

## Implementation inventory

- `src/lua_api/state/sim_state.rs`: existing bars/macros/outfits/UI tuples/aliases and known spell membership; no new state planned.
- `src/lua_api/globals/inventory_verbs.rs`: existing `GetActionInfo` outfit → macro → spell effective-kind priority.
- [`src/lua_api/globals/action_bar_api.rs`](../../src/lua_api/globals/action_bar_api.rs): existing public slot move and UI registration; old empty Find provider compiled only without `retail-12-0-5`.
- [`src/lua_api/globals/action_bar_api/registration.rs`](../../src/lua_api/globals/action_bar_api/registration.rs): creates the namespace, then publishes the pair through the C API producer after existing registrations under cumulative `retail-12-0-5`; old Find entry gated out for that epoch.
- `src/c_api/c_spell.rs`: existing strict public identifier boundary and alias-first resolver.
- [`src/c_api/c_action_bar_spell_slots.rs`](../../src/c_api/c_action_bar_spell_slots.rs): sole epoch-owned pair producer, shared effective-slot iterator excluding zero/macro/outfit slots; strict shared identifier boundary, fresh dense ascending table (empty on miss) and boolean. Read-only, no new state/catalog.
- [`src/c_api/mod.rs`](../../src/c_api/mod.rs): producer module compiled only under cumulative `retail-12-0-5`; earlier profiles retain their previous publication.

Cached evidence (declarations/consumer, not native execution):

- `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/ActionBarFrameDocumentation.lua`: Find “Returns the list of action bar slots that contain a specified spell.” Declares `slots: table<luaIndex>`, `MayReturnNothing=true`, and base-spell expectation. Has declares one non-nil bool. Both declare `SpellIdentifier` and `AllowedWhenTainted`; neither establishes the chosen result secrecy/validation/miss policy.
- `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_ActionBar/Shared/ActionButtonUtil.lua`: passes Find results to `AddPlayerActionBarsContainingSlots`; separately checks special bars. Establishes slot representation, not exhaustive native coverage or ordering.
- `data/patch-api/sources/12.0.5-api-changes.txt`: rows229/243 establish only argument-type delta, not this bounded model or security policy.

## Tests asserting this spec

`tests/action_spell_slot_identifiers.rs`: **17 focused tests**, grouped by `retail-12-0-5` feature; no new Cargo target. Fixtures unchanged. Parent compiled RED at `61e002a3811adb2b1cd3cb27e23b6883ed3910be`: build exit0/178.56892285693903s; runtime exit101/3.0491113850148395s, 0PASS/17FAIL. Proof: `/tmp/patch-12.0.5-batch51-red-build-result.json`, `/tmp/patch-12.0.5-batch51-red-run.json` and `.stdout`. Artifacts record preserved unowned dirty-source provenance, not clean-revision proof. Implementation has no local compilation/test/check/lint/readability/coverage evidence; parent GREEN pending.

Fixtures clear/replace the default Protection Paladin bar before all query assertions. Seed slots 3/101 → 7001, 5 → 7002, macro-only 8 → 77, plus explicit name and full-link aliases. `GetActionInfo` verifies seeded kinds/IDs before paired calls. Shadow fixtures verify effective macro/outfit kinds before querying; real UI Frame verifies `GetObjectType() == 'Frame'` and actual registered tuple before the non-assignment query. Public move follows actual source/target order 101 → 12. Invalid/secret fixtures use real runtime values, never Lua marker secrets, API replacements or presumed userdata variants.

| Capability | Fixture assertion | Proof level |
|---|---|---|
| Pair shapes, two/single/miss/empty matches | Exact arity, dense members, public table/bool, freshness | Parent compiled RED 0PASS/17FAIL; producer implemented, GREEN pending |
| Shared aliases and strict boundary | Name/link/numeric override, mutation, endpoints, invalid values before aliases | Tests written; policy INFERRED |
| Effective live assignments | Macro/outfit shadow preconditions, UI non-assignment, public move/host changes | Tests written; model INFERRED |
| Isolation and security | Input snapshots, caller/result mutation, taint, authentic secrets across GC | Tests written; native permissions unknown |

## Known gaps (current cycle)

- [x] Parent compile and genuine RED for these exact fixtures, recorded above; no local build/test/check/lint/readability/coverage/gate execution authorized.
- [x] Implement bounded producer and single-owner epoch publication after parent compiled RED; fixtures and backing state unchanged.
- [ ] Parent-owned GREEN and acceptance before checking any contract bullet; no source-row coverage credit claimed.

## Out of scope

- Row245 and special-bar semantics; active page/visibility, vehicle, possess, pet, stance, bonus/override or temporary bars.
- Base/override normalization despite cached base-spell expectation; full native catalog/grammar and assignment acquisition.
- Macro spell inference, invented item representation, new models/catalogs or UI-frame-derived assignments.
- Native validation/errors, alias rules, output order, result secrecy, `MayReturnNothing` conditions and exact native secret/taint access permissions. Conservative local rejection is not native permission parity.
