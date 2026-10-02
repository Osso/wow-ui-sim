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

## Reconciled combined batch51/52 parent GREEN — 2026-10-02

This section is the saved-artifact proof SSOT, not independent acceptance. Batch51 inputs/17 fixtures: `61e002a3811adb2b1cd3cb27e23b6883ed3910be`; producer: `398fac2950f2fe36ac8a51177bbbeb4bf8a3fde3`; exact-five annotation docs: `3d357dd10` (existing batch52 provider unchanged); corrected producer: `6c3c3192621b916a27b967d9cdd231fc875d9229`. Every saved proof includes dirty-source diff SHA256 `6967f0b47312d926c2359bd29bc1c26d4d1d523522abf8d87e067104da170a1a`: compiled combined dirty sources, **not clean-revision proof**. Protected unowned duration source was not inspected or changed for this reconciliation.

### Historical RED and failed producer compilation

Full `/tmp/patch-12.0.5-batch51-red-{build-result,run}.json` and `red-run.stdout` retain compiled genuine **0PASS/17FAIL**, compile exit0/178.56892285693903s and runtime exit101/3.0491113850148395s. RED integration binary SHA256 `9d44aad1c4400516eb0b3c4d4b1d7f3c7b74f58d6eb88d70ce82e3501b9b3b2e`. This is batch51 RED only; batch52 had meaningful existing coverage, no fabricated RED or new producer.

`/tmp/patch-12.0.5-batch51-52-green-build-result.json` and full `green-build.jsonl` record producer compile exit101/10.111329689971171s, no executable artifacts: E0308 at producer lines44/64, expected `SimState`, found `Ref<'_, SimState>` in the borrowed-state expression. It is a compile failure, not runtime GREEN. Corrected revision above fixes the coercion; failed evidence remains historical.

### Corrected compiled commands and results

Saved compiler argv: `cargo test --test integration --no-run --message-format=json`. Full `/tmp/patch-12.0.5-batch51-52-green-fixed-build.jsonl` (738 records, build-finished success, no compiler diagnostics) and `.stderr` agree with `green-fixed-build-result.json`: exit0/**160.59031843405683s** (160.590s rounded).

Integration executable `/syncthing/Sync/Projects/wow/wow-ui-sim/target/debug/deps/integration-a11e89d240f9bd0c`, SHA256 `bfa0f09fef20f376ea292a837b6d0b55937932254e78468748297c352f8f913b`. Each saved argv is `timeout 90 <integration-executable> <filter> --test-threads=1`; `/tmp/patch-12.0.5-batch51-52-green-fixed-runs.json` owns exact argv/revision/dirty hash. Every row exits0. Full outputs: `/tmp/patch-12.0.5-batch51-52-green-fixed-run-<index>.stdout`.

| Index | Filter | Distinct PASS | Runtime seconds | Full stdout SHA256 |
|---|---|---:|---:|---|
| 0 | `action_spell_slot_identifiers::` | 17 | 2.808688770979643 | `bf4e912cc2d8e464da0beccda37e1297147c64c58acf903787d7df1e060f9746` |
| 1 | `c_action_bar_slot_mutation::` | 10 | 1.7368072390090674 | `4279d0db3c77b95bf8eab8520f82a80f2c8211067f585c77c20d5fc92ee047d1` |
| 2 | `admin_actionbar_api::` | 9 | 1.2835899300407618 | `4b58482298c69671045a57bcee7f4f7a820657ad734e3b4241cb80ff18c15c58` |
| 3 | `inventory_verbs::` | 25 | 3.4967042410280555 | `88baa75f9c5892664f695e4eb05dd9c59dac64c4d5f4c1033a4e65c738d82855` |
| 4 | `action_text::` | 5 | 1.057624691980891 | `e2e1082922745b2c893e8b71bb5355e66c0e03af7bedf78cd078eead848f2099` |
| 5 | `c_action_bar_input_probes::` | 11 | 1.7267762189731002 | `e592e9998b46bda28c510a3a4a5c9a574cf2f4bb3ed5d042a0f47115af7e685b` |
| 6 | `c_action_bar_state_globals::` | 11 | 1.6775870760902762 | `25f770a8190022d8b21e559e183f31372bafe50930a3bf1d5898c39334673248` |
| 7 | `cooldown_aura_spell_identifiers::` | 14 | 2.414613540051505 | `15ba1aa59418bd077be48a39021fba7b208d446250355fd5d6b39db266416a7e` |
| 8 | `aura_spell_classification_identifiers::` | 16 | 2.4545076290378347 | `6dffd9a43e912bdd756efe14f0266e704375353103a093b59bef8495b3201484` |
| 9 | `character_stats::stat_restriction::` | 4 | 0.9239758340409026 | `7d9fc10fcffdfb5e21d47f0f9916e7e10e757e6d51f113ad8cf18e1a74fb0bb3` |

Full ten outputs contain **17 new +105 controls =122 unique named PASS**, zero duplicate test names, zero failures. The four restriction controls exercise the unchanged existing provider, including the five exact batch52 player fixtures; they do not create five new tests or native stat models.

Actual ten-run runtime sum **19.5808751712320374s**. Startup argv: `timeout 90 /syncthing/Sync/Projects/wow/wow-ui-sim/target/debug/wow-sim --no-addons --no-saved-vars lua-errors`; executable SHA256 `f24d2b936ec9915bdfee94eb50a3ecf6901ef7ddcc98de092eeec71cb63f4032`. Saved `green-fixed-startup-run.json` and full `green-fixed-startup.stdout`: exit0, exact `[]` plus newline, **4.768463075044565s**. Runtime including startup **24.3493382462766024s**, below60s target; preserve actual short runtime, no padding/repeat, no final whole-goal/page acceptance.

### Capability boundaries and pending acceptance

| Exact scope | Observed saved proof | Still unknown/excluded |
|---|---|---|
| Batch51 rows229/243 | Meaningful existing direct assignments; positive slots, macro/outfit/zero exclusion; live changes, aliases, fresh table/bool, isolation, strict public boundary, real VM-secret rejection/GC and taint preservation | Native catalog/acquisition, base/override and special bars, native miss/zero-result conditions, order, result secrecy and secret permissions. Policies remain inferred. |
| Batch52 rows514/500/424/438/498 | [Exact-five annotation scope](unit-stat-output-restriction.md#batch52-saved-parent-observed-proof--2026-10-02): meaningful explicit player values and numeric output wrapper, plain/restricted/plain arity/order/value and tainted opacity | Decomposition, placeholders, nonplayer behavior, formulas, native activation/access/secrecy parity. Prior40-API coverage remains historical and unchanged. |

Independent verifier414 concurrent/pending; no report read or acceptance inferred. Independent acceptance, accounting, coverage and all unchecked requirements remain pending. No data/page-coverage/PLAN changes or whole-goal closure. Earlier GREEN-pending checkpoints above describe their historical point; this saved observation supersedes only the absence of parent GREEN, not acceptance.

### Artifact SHA256 ledger

All paths below are under `/tmp/`; hashes cover full artifacts, not excerpts.

| Artifact | SHA256 |
|---|---|
| `patch-12.0.5-batch51-red-build-result.json` | `8bdd0eee8c011f040dcf0efeef85ff0c11210cd6520f1a946f390e94bdd1169b` |
| `patch-12.0.5-batch51-red-run.json` | `a29265fd1ee15a40cf9aecb63cd898cc30db9bff80a9a7479ed9c4beb979121b` |
| `patch-12.0.5-batch51-red-run.stdout` | `065438fad30f80f2d1a36049ddc4c855c449ed79c80921b4dd9216208f57d184` |
| `patch-12.0.5-batch51-52-green-build-result.json` | `601493a457036601cd3b5c56692e20db4e739a20a54440467adcb360bd64e7f9` |
| `patch-12.0.5-batch51-52-green-build.jsonl` | `ebef8b82af07be8800d9513a8cda85be248f363560c8d8ba805218793d6d6ffa` |
| `patch-12.0.5-batch51-52-green-fixed-build-result.json` | `f2e866f7ea2c71b05906c428dbcebe9ed82ae4341dca55417427e567d806f8e7` |
| `patch-12.0.5-batch51-52-green-fixed-build.jsonl` | `12b1e920cde8ccedba0ddf30bf630685f283ebe22c026965d33801c57ddb9779` |
| `patch-12.0.5-batch51-52-green-fixed-build.stderr` | `0f3ba29488a472ec6d4151f2e4bd0c0dc5c7f306cbed21c6fc9b8b45477b44de` |
| `patch-12.0.5-batch51-52-green-fixed-runs.json` | `d9f3626b50ac4900b19f368e801cc3873ceab723281fe3ed2f8cc6458e0e0fa5` |
| `patch-12.0.5-batch51-52-green-fixed-startup-run.json` | `0429e0ef1fe6191325b6f47a80728014c9b111a83e9a8cf8e6e2858961489ae6` |
| `patch-12.0.5-batch51-52-green-fixed-startup.stdout` | `37517e5f3dc66819f61f5a7bb8ace1921282415f10551d2defa5c3eb0985b570` |
