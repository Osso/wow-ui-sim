# Tertiary stat inputs

Next55 covers exactly three pending 12.0.5 PlayerScript rows from the [retained GlobalAPI source](../../data/patch-api/sources/12.0.5-api-changes.txt) and [register](../../data/patch-api/sources/12.0.5-register.json). Existing typed character rating fields must supply meaningful current-snapshot outputs; [unit-stat output restriction](unit-stat-output-restriction.md) owns the shared secrecy contract. As of 2026-10-02: parent compiled genuine RED; minimal upstream producer fix implemented, parent GREEN/profile proof pending. No accounting changes.

| Exact retained ID | Literal added annotation | Existing input → getter expectation |
|---|---|---|
| `global api-PlayerScript GetAvoidance-420` | `PlayerScript GetAvoidance` / `+ SecretWhenUnitStatsRestricted` | `avoidance_rating = 2250` → `12.5` |
| `global api-PlayerScript GetLifesteal-442` | `PlayerScript GetLifesteal` / `+ SecretWhenUnitStatsRestricted` | `leech_rating = 585` → `3.25` |
| `global api-PlayerScript GetSpeed-480` | `PlayerScript GetSpeed` / `+ SecretWhenUnitStatsRestricted` | `speed_rating = 1395` → `7.75` |

These concrete host inputs are chosen fixtures, not native catalog/acquisition evidence. **GUESS / INFERRED assistant conversion:** 180 rating per percentage point, clamped nonnegative. This coefficient reuses the existing common conversion, but is neither native-verified nor a user-selected native coefficient. The requested scope permits this guess; absent native conversion evidence does not block bounded simulator development.

## What it must do

### Current snapshot and shared rating lookup

- [ ] Under `retail-12-0-5`, the three getters read existing `CharacterStats` fields at indices21 avoidance,17 lifesteal,13 speed. No duplicate percentage fields or input struct.
- [ ] **INFERRED:** percentage results equal `max(current_rating / 180, 0)`. These three cases are explicit in the ratio helper rather than classified as unknown indices.
- [ ] `GetCombatRating(21/17/13)` exposes raw ratings, including negatives; `GetCombatRatingBonus` exposes matching clamped percentages from the same snapshot. These supporting queries earn no additional row424/426 credit.
- [ ] Live host replacement of one typed field changes only its corresponding outputs. Default/explicit zero follows a demonstrated nonzero transition; independent environments do not share fields or restriction state.
- [ ] Queries leave the rating snapshot and restriction input unchanged. Existing crit9 and unknown999 controls retain current behavior, including the existing explicit-value helper's unknown-index180 conversion.

### Callback output and security

- [ ] Public callbacks succeed with exactly one result (`pcall` yields exactly `true, value`), preserving exact values through plain → restricted → plain. Each result position is checked for secrecy/access.
- [ ] Public stamped callers retain their stack taint and receive plain values when unrestricted. Restricted stamped callers receive authentic host getter results, cannot unwrap or perform arithmetic, and retain caller taint.
- [ ] Actual getter-produced secret NUMBER results remain rooted with secrecy and raw identity across forced GC and tainted calls; trusted recovery reveals exact values without changing typed inputs. Tests do not fabricate markers, replace APIs, or assume secret Boolean raw equality.

### Recompute and profile boundary

- [ ] Getters read the current computed snapshot, not sticky host overrides. Existing equipment recomputation may reset these unpopulated ratings to zero; subsequent host replacement is immediately visible.
- [ ] New field lookup/bonus cases are gated to `retail-12-0-5`; profiles without that epoch retain previous zero behavior. No other stat formula, state, gear computation, or restriction producer changes.

## How it works

- [Lua API architecture](../lua-api.md)
- [Shared restriction contract and evidence](unit-stat-output-restriction.md)

## Implementation inventory

- `src/lua_api/state_types/character_world.rs` — existing `CharacterStats.{avoidance_rating,leech_rating,speed_rating}` and snapshot computation; unchanged.
- `src/lua_api/globals/real/combat_stats.rs` — indices13/17/21 map to existing speed/leech/avoidance fields; raw bonus queries reuse that mapping and the explicit guessed180 ratio. One `cfg!(feature = "retail-12-0-5")` constant gates the new lookups/bonus branch. Existing getters and `push_stat_number`/taint behavior are unchanged; older profiles retain zero even with nonzero host fields.
- `src/lua_api/globals/admin_equipment.rs` — existing equipment callbacks replace stats with `CharacterStats::compute`; unchanged.
- `src/c_api/c_secrets.rs` — existing trusted number output wrapper; unchanged.

Root cause before this fix: `combat_rating_for` omitted indices13/17/21 despite those fields already existing; its unknown arm returned zero. `GetCombatRatingBonus` separately omitted them. The three getters already route through the shared rating lookup and ratio helper. Adding downstream percentage state would duplicate existing inputs and conceal the upstream omission; that earlier proposal in `/tmp/patch-12.0.5-zero-stat-input-boundary.md` is rejected.

`CharacterStats::compute` starts from defaults and does not populate the three ratings. No public `PlayerState::recompute_stats()` method exists in the inspected Lua state implementation. The test instead uses the actual existing `A_Admin.UnequipItem(1)` callback, which replaces the snapshot through `CharacterStats::compute`; it asserts item removal, zero reset and later host replacement. No new admin API or persistence contract is introduced.

## Tests asserting this spec

`tests/tertiary_stat_inputs.rs`: 14 focused tests, cfg `retail-12-0-5`, discovered by the existing grouped integration harness; no new Cargo target.

| Coverage | Tests | Proof |
|---|---:|---|
| Exact getters with nonzero raw/bonus lookup | 3 | Parent compiled RED; GREEN pending |
| Live replacement, default/reset, negative clamp, environment isolation, query immutability | 5 | Parent compiled RED; GREEN pending |
| Toggle arity/values, public/restricted stamped taint, authentic rooted GC/recovery | 4 | Parent compiled RED; GREEN pending |
| Actual equipment snapshot replacement; crit/unknown/explicit-ratio controls | 2 | Parent compiled RED; GREEN pending |

Saved RED proof: `/tmp/patch-12.0.5-batch55-red-build-result.json` and `/tmp/patch-12.0.5-batch55-red-run.json`, with parent full outputs. Artifact SHA256 `caf93dbd864b986aceec5e72cb94b6fcf45c62acda4488e3c1fd12fbad2e11ee`; dirty-source diff SHA256 `6967f0b47312d926c2359bd29bc1c26d4d1d523522abf8d87e067104da170a1a`. Dirty combined evidence includes preserved unowned `src/c_api/aura_duration.rs`, not clean-revision proof; this producer change invalidates RED as current-code proof. Formatting is not GREEN.

All14 batch55 tests and existing four restriction tests/zero fixtures remain unchanged. Source420/442/480 remain `audit-pending` and uncredited; no capability, page coverage, wiki, data, PLAN, or unrelated annotation promotion.

## Known gaps (current cycle)

- [x] Parent compiled and observed genuine RED before producer implementation: revision `4610f9a42bd5ae34045266536c76fe9988274e6e`, compile exit0/158.60273947101086s; run exit101/4.041617421084084s, 0PASS/14FAIL. All failures were exact current snapshot value assertions after startup/setup.
- [ ] Parent-owned GREEN, current-source check, profile-preservation static/bounded proof and independent acceptance remain pending. No current passing claims.
- [ ] Native conversion, acquisition/catalog, rating-by-level/profile, activation rules, permissions and secrecy parity are unknown. These tests establish only the inferred simulator contract.

## Out of scope

- Duplicate percentage inputs, new structs, admin APIs, catalog/acquisition, gear synthesis or preservation of host inputs across recomputation: neither required nor grounded.
- Changes to other formulas, state, automatic restriction producers or prior-profile ordinary values.
- Local build/test/check/readability/coverage/gates, delegation, model CLI, push and accounting changes: parent owns verification and acceptance.
