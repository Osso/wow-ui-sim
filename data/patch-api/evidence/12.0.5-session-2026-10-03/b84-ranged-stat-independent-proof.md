# B84 independent read-only verification — 2026-10-03

## Scope and evidence class

Verified only rows 474 (`GetRangedCritChance`) and 476 (`GetRangedHaste`) against B84 in `docs/specs/unit-stat-output-restriction.md:197–218`. Followed `verify/SKILL.md` as the assigned verifier: direct artifact inspection, no delegation. Files exist and contain substantive providers, assertions and accounting records; registration wiring is shown below. No implementation was added by this supplement, so a new-code anti-pattern gate is not applicable.

**Evidence validated: current source/test inspection, read-only Git comparisons, cached declaration inspection, and committed historical acceptance/accounting records. Not fresh behavioral execution or independently inspected historical raw logs.** No Cargo, builds, tests, formatters, simulator, agents or model CLIs ran. Repository files were not changed; only this report was written.

Observed HEAD: `092b6efab9fa238b9f3443c9e7a8c4cc6c5b2880`. `git status --porcelain=v1` returned only ` M src/c_api/aura_duration.rs`. Both commands exited 0, with empty stderr.

## Results by item

| Item | Row 474 | Row 476 |
|---|---|---|
| 1. Literal source/register delta | PASS | PASS |
| 2. Cached declarations | PASS | PASS |
| 3. Providers, registration, wrapping and features | PASS | PASS |
| 4. Seeded values and assertion coverage | PASS | PASS |
| 5. Reuse-scope diff and unrelated dirty file | PASS | PASS |
| 6. Committed historical proof accurately reused | PASS, recorded evidence only | PASS, recorded evidence only |
| 7. Limits and unsupported assertions | PASS with evidence qualifications | FAIL for literal “only” smoke-test assertion; annotation boundary remains supported |
| 8. Pending rows and current accounting | PASS | PASS |

### 1. Literal source and register — PASS / PASS

`data/patch-api/sources/12.0.5-api-changes.txt:473–476` reads:

```text
473: PlayerScript GetRangedCritChance
474:   + SecretWhenUnitStatsRestricted
475: PlayerScript GetRangedHaste
476:   + SecretWhenUnitStatsRestricted
```

Inspection of the entire retained file found those subject names only at lines 473 and 475. Neither subject has another delta anywhere in that file.

`data/patch-api/sources/12.0.5-register.json:3109–3120` records `"id": "global api-PlayerScript GetRangedCritChance-474"`, `"kind": "delta"`, `"subject": "PlayerScript GetRangedCritChance"`, `"change": "+ SecretWhenUnitStatsRestricted"`, and `"source_lines": [474]` (array formatted across lines 3115–3117). Lines 3121–3132 record the analogous haste ID, kind and change, with `source_lines` `[476]` at 3127–3129. Both say `"chronology": "consolidated-final"` and `"status": "consolidated-delta"`.

This is output-annotation credit only: no input, signature, formula or activation delta is present.

### 2. Cached declarations — PASS / PASS

Inspected `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/PlayerScriptDocumentation.lua`.

- Crit block: lines 744–753, name at 745, `Type = "Function"` at 746, `SecretWhenUnitStatsRestricted = true` at 747, return at 751: `{ Name = "result", Type = "number", Nilable = false }`.
- Haste block: lines 754–763, name at 755, function type at 756, the same annotation at 757, identical return declaration at 761.

Neither complete block contains `Arguments` or `SecretArguments`. B84’s named ranges 745–753 and 755–763 accurately identify the declarations, omitting only each opening brace. These are cached contract declarations, not native execution or independently authenticated cache provenance.

### 3. Provider chains, wiring and feature boundaries — PASS / PASS

`src/lua_api/globals/real/combat_stats.rs:109–122`:

```rust
let crit = borrow_state(state)?.player.stats.crit_pct() + 5.0;
push_stat_number(state, crit)?;
Ok(1)
```

`get_ranged_crit_chance` at 120–122 calls `get_crit_chance(state)` directly.

The same file at 143–147 computes `player.stats.haste_pct()`, calls `push_stat_number(state, haste)?`, and returns `Ok(1)`. `get_ranged_haste` at 153–166 delegates to `get_haste(state)` under `#[cfg(not(feature = "client-wowforever"))]` at 164–165. Its Forever branch at 154–162 instead pushes `Val::Num(haste)` and `Val::Num(quiver)` and returns `Ok(2)`; it never calls the stat wrapper.

Registration entries are `("GetRangedCritChance", get_ranged_crit_chance)` at 357 and `("GetRangedHaste", get_ranged_haste)` at 369. `register_all` at 413–419 loops over `COMBAT_STAT_GLOBALS` and calls `LuaApiMut::register_function(lua, name, function)?`. External caller `src/lua_api/globals/register.rs:283` invokes `super::real::combat_stats::register_all(lua)?`.

`src/c_api/c_secrets.rs:65–79` implements `push_stat_number`: with `retail-12-0-5`, read `unit_stats_restricted` and choose `rilua::table_security::wrap_host_secret_number(state, number)` or `Val::Num(number)`; without that feature, use plain `Val::Num(number)`. Line 78 performs exactly one `state.push(value)`.

Therefore each successful default-retail call returns one value through this wrapper boundary. The value is secret only when the explicit flag is true, not unconditionally.

Feature evidence:

- `Cargo.toml:108`: `default = ["sound", "gui", "casc", "client-retail"]`.
- Lines 119–121 and 149: `client-retail` enables `profile-retail` and `retail-12-1-0`, which transitively enables `retail-12-0-7` and `retail-12-0-5`. Default builds compile the restriction-aware helper and non-Forever, one-result haste branch. Default is the 12.1.0 epoch, not an exactly-12.0.5 epoch.
- An exact historical retail configuration using `profile-retail,retail-12-0-5` without defaults compiles those same branches. Enabling `retail-12-0-5` alongside defaults does not downgrade the default epoch. The epoch feature alone, without a profile, is invalid.
- `Cargo.toml:155` lists Forever’s features without a retail epoch. `src/client_profile.rs:238–252` rejects Forever combined with another profile. Lines 270–280 reject retail epoch features unless `profile-retail` or `client-ptr` is present: `compile_error!("retail API epoch features require profile-retail or client-ptr")`.

Thus `client-wowforever` cannot validly combine with `retail-12-0-5`: it needs a retail/PTR profile for the epoch, but Forever cannot coexist with either. B84’s Forever exclusion is substantive and correctly describes its two plain outputs. Feature conclusions are source-derived, not compiled in this verification.

### 4. Seeded values and tests — PASS / PASS

`tests/character_stats/stat_restriction_fixtures.lua:25–26` contains exactly:

```lua
{'GetRangedCritChance', {}, {7}},
{'GetRangedHaste', {}, {3}},
```

`tests/character_stats/stat_restriction.rs:7–36` creates a new environment, seeds player state and loads the shared fixtures; lines 19–20 set `player.stats.crit_rating = 360` and `player.stats.haste_rating = 510`.

Real implementations in `src/lua_api/state_types/character_world.rs:108–113` are:

```rust
pub fn crit_pct(&self) -> f64 { self.crit_rating as f64 / 180.0 }
pub fn haste_pct(&self) -> f64 { self.haste_rating as f64 / 170.0 }
```

The provider values follow from state: `360 / 180 + 5 = 7` and `510 / 170 = 3`. These getters do not return literal 7 or 3. Their divisors and crit base are simulator choices, not independently verified native formulas.

`tests/character_stats.rs:9–11` includes the restriction test module under `#[cfg(feature = "retail-12-0-5")]`.

- `stat_restriction_all_supported_outputs_preserve_values_arity_and_toggle`, restriction Rust file 39–65: calls `assertStatOutputs(false)` at 41, sets restriction true at 42, calls `assertStatOutputs(true)` at 46, restores false at 62, then calls the plain assertion again at 63. Shared Lua lines 44–57 pack the actual return count, assert `actual.n == #expected`, secrecy/access for each ordered result, trusted unwrap when restricted, and `math.abs(result - value) < 1e-10`. Both ranged fixtures participate. “Exact value” here means that stated numerical tolerance, not bitwise equality.
- `stat_restriction_tainted_callers_receive_opaque_host_results`, Rust file 68–100: iterates every fixture at 77–90; checks arity at 81, secret/inaccessible at 84, denied unwrap at 85, denied arithmetic at 86, and unchanged `StatRestrictionProbe` stack taint at 90. Lines 94–95 stamp and invoke the tainted function. Line 97 additionally calls the trusted `assertStatOutputs(true)`. Exact numeric values are not unwrapped by the tainted caller.
- The absent-unit test at 104–137 calls only missing-unit stats/armor/attack-power/speed plus block/attack-power-for-stat controls. Although it loads the fixtures, it does not invoke the ranged functions. The predicate test at 140–158 invokes the predicate, rating and block controls, not either ranged function.

The two named tests really contain the claimed behavioral assertions; the other two are correctly described as controls. This inspection did not execute any test.

### 5. Unchanged reuse scope and dirty file — PASS / PASS

Fresh read-only command:

```text
git diff --no-ext-diff d21d4a208 HEAD -- src tests Cargo.toml Cargo.lock
exit 0; stdout 0 bytes; stderr empty
```

This is the requested comparison with external diff disabled. It proves equality of committed files in that scope, not clean whole-tree identity or preservation of a historical runtime environment.

`git diff --no-ext-diff -- src/c_api/aura_duration.rs` exited 0 and showed only hunks `@@ -44,6 +44,7 @@` and `@@ -52,6 +53,16 @@`: registration of `"DoesAuraHaveExpirationTime"` and its function, testing `aura.expiration_time != 0.0` before pushing a boolean. No change touches either provider, `push_stat_number`, the player-stat model or restriction tests. The dirty file was not edited, formatted, staged or reverted.

B84 line 210’s earlier empty comparison at `559d47720` is now corroborated through current HEAD; it is not a fresh historical execution result.

### 6. Reused proof record — PASS / PASS, recorded-evidence qualification

`data/patch-api/sources/12.0.5-page-coverage.json:1409–1423` records capability `"spell-bonus-stat-security"`, implementation and compiled revision `d21d4a208da4b911f003181e5ca33163f42b92be`, proof `"bounded-independent-pass-qualified-check"`, and ledger `/tmp/patch-12.0.5-spell-bonus-stat-independent-proof.md`. Its `gate_scope` says `"12 ALL refreshed PASS=7new+4stat+1proxy"` and explicitly says successful check JSON had synchronous execution/direct exit not retained, without exit0/async-check clearance.

`docs/specs/spell-bonus-stat-security.md:66–74` corroborates this:

- Line 68: “12 distinct refreshed PASS =7 new+4 existing restriction controls+1 proxy control,” producer revision above and compile exit0.
- Line 72: “direct exit was not retained”; “Dirty-combined provenance/guarded source equality is not clean whole-tree identity”.
- Line 74 limits annotation credit and retains native/formula/activation/older-profile/broad-suite gaps.

The historical four-test aggregate includes the fixture-bearing tests just inspected; B84 does not invent two new executions or claim twelve ranged-specific cases. This is an inference from recorded aggregate acceptance plus unchanged test/provider source, not inspection of individual historical ranged results.

Python enumeration of `/tmp/patch-12.0.5-*` returned `[]`: no matching artifacts exist on this host. Raw historical reports/logs, hashes, exit codes and runtime assertions could not be independently authenticated. B84 line 210 accurately discloses unavailable artifacts and carries the qualified check protocol rather than clearing it.

### 7. Limits and one literal overstatement

**Row 474: PASS with qualifications.** B84 lines 214–216 honestly exclude ranged-specific modeling, native formulas, automatic activation, GUI/native parity and full suites. Its single seeded fixture and tainted/trusted boundaries are supported. No uncovered annotation boundary was established by source inspection; that is not proof that no possible boundary gap exists. Native access/secrecy policy, cache provenance and live execution remain unverified. Public/restricted numerical checks use tolerance `1e-10`.

**Row 476: FAIL for one literal documentation assertion, not its bounded retail annotation behavior.** Line 216 says: “No-argument plain calls elsewhere appear only in a `tests/character_stats.rs` smoke check (number, nonnegative).” Taken repository-wide, this is false for haste:

`tests/forever_character_remaining_stats.rs:3` gates the module under `client-wowforever`. Line 34 calls `count(GetRangedHaste())`, with lines 37–39 asserting `(ranged_arity, defense_arity) == (2, 2)`. Line 68 calls `local haste, quiver = GetRangedHaste()`; lines 74–78 assert `(haste, quiver, melee) == (baseline_haste, 3.5, baseline_haste)`. Line 80 calls it in a fresh environment, with lines 82–85 checking zero quiver and default modifiers.

These are additional no-argument plain-output tests in the excluded Forever profile. They do not expand 12.0.5 coverage and were not executed here. The “only” statement is valid only if explicitly narrowed to the non-Forever/default-retail test scope. B84’s actual Forever provider description at 215 is correct; “older-profile behavior is unverified here” remains honest about this audit’s execution evidence.

For comparison, `tests/character_stats.rs:166–213` is the described smoke test: calls crit at 175 and haste at 186, then checks `type(value) ~= "number" or value < 0` at 201. No other direct occurrences of the ranged global names were found in Rust/Lua under `src` and `tests` beyond providers, shared fixtures, that smoke test and the Forever tests. This search does not establish exhaustive coverage of dynamically constructed names or external addons.

### 8. Current accounting — PASS / PASS

`data/patch-api/sources/12.0.5-page-coverage.json:3144–3155`:

- Row 474: `"source_id": "global api-PlayerScript GetRangedCritChance-474"`, `"capabilities": []`, `"status": "audit-pending"`, note “Source retained; behavioral applicability audit not completed.”
- Row 476: analogous haste ID, empty capabilities, same status and note.

Parsed the whole current JSON, not historical prose. Across `source_rows` (lines 1443–3974): **362 rows** = **180 bounded-coverage**, **164 audit-pending**, **11 partial-development-green**, **7 metadata-only**. The current `capabilities` array contains **87 capabilities**. B82’s older 86/165/179 figures are historical, not current counts. Neither ranged row is already accounted as accepted.

## Final verdicts

**Row 474 — ACCEPT WITH QUALIFICATIONS.** Accept the single seeded default-retail output-annotation claim, based on substantive existing tests, reachable state-backed provider and unchanged reuse scope. Qualification: current proof is source plus committed historical acceptance, not freshly executed or raw-log-authenticated behavior; native formula/access/secrecy/activation and cache provenance remain unverified; numerical preservation uses `1e-10` tolerance.

**Row 476 — ACCEPT WITH QUALIFICATIONS.** Same bounded evidence and execution qualifications. Additional qualification: B84’s “only” smoke-test assertion must be read as non-Forever/default-retail only; literal repository-wide wording is falsified by existing Forever ranged-haste tests. Those tests supply no additional 12.0.5 or native credit. Valid retail epoch configurations cannot combine with Forever, whose provider returns two plain values.

No builds, tests, runtime/native/GUI probes, historical raw-artifact authentication, artifact hash verification or clean historical environment verification were performed. No row status was changed. These verdicts concern only rows 474 and 476.
