# B85 independent read-only verification — 2026-10-03

## Scope and observed revision

Reviewed `docs/specs/unit-stat-output-restriction.md:230–255`, independently tracing retained deltas, cached function declarations, registered producers, fixture assertions and committed proof/accounting records. Followed the verify skill as the assigned verifier; no delegation, builds, tests, formatters, simulator execution or repository writes.

Initial `git rev-parse HEAD` returned `68e76142a23b6f98aec68e40e7b6b5bdd95236a1`; `git status --porcelain` returned empty, both exit 0. During verification another actor advanced HEAD to `a342c6eec1fa758487082eb8a0653353d80441df` (`Add B86 aura expiration-time query inputs`). Tree remained clean. Its scoped change from `68e76142a` is solely `tests/aura_expiration_time.rs`, 298 insertions. B85 findings below are pinned to the requested `68e76142a`; the relevant stat sources/tests/accounting remained unchanged. An unqualified statement about the final moving HEAD having no test changes would now be false.

## Per-item results

| Item | 502 | 508 | 512 |
|---|---|---|---|
| 1. Literal delta and register identity | PASS | PASS | PASS |
| 2. Cached function declaration | PASS | PASS | PASS |
| 3. Registered provider and wrapped positions | PASS | PASS | PASS |
| 4. Seeded fixture and actual assertions | PASS | PASS | PASS |
| 5. Scoped proof-reuse boundary at requested revision | PASS, qualified | PASS, qualified | PASS, qualified |
| 6. Honest committed-record evidence class | PASS, qualified | PASS, qualified | PASS, qualified |
| 7. Bounded limits and precedent | PASS, qualified | PASS, qualified | PASS, qualified |
| 8. No-credit deferrals | PASS, shared finding | PASS, shared finding | PASS, shared finding |
| 9. Pending accounting | PASS | PASS | PASS |

PASS here means the inspected documentation/source claim is substantiated, not freshly executed behavioral PASS.

### 1. Source rows and register

`data/patch-api/sources/12.0.5-api-changes.txt` contains:

- 501–502: `Unit UnitAttackPower` / `+ SecretWhenUnitStatsRestricted`.
- 507–508: `Unit UnitRangedAttackPower` / `+ SecretWhenUnitStatsRestricted`.
- 511–512: `Unit UnitSpellHaste` / `+ SecretWhenUnitStatsRestricted`.

Full-file subject inspection found no other occurrence/delta for these three subjects. These are output-only annotation additions, not new signatures, formulas, argument policies or activation rules.

Matching `12.0.5-register.json` records are at 3277–3288, 3313–3324 and 3337–3348, respectively. Each states `"kind": "delta"`, the matching subject, `"change": "+ SecretWhenUnitStatsRestricted"`, singleton `source_lines` 502/508/512, `"chronology": "consolidated-final"` and `"status": "consolidated-delta"`.

### 2. Cached declarations, not events

Cache prefix: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/`.

`UnitDocumentation.lua` function blocks are exactly 654–670, 2855–2871 and 3023–3037. Each has `Type = "Function"`, `SecretWhenUnitStatsRestricted = true`, `SecretArguments = "AllowedWhenUntainted"`, and one argument `{ Name = "unit", Type = "UnitToken", Nilable = false }`.

502 returns at 666–668 and 508 returns at 2867–2869 are `attackPower`, `posBuff`, `negBuff`, each `Type = "number", Nilable = false`. 512 returns at 3035 are `result`, `Type = "number", Nilable = false`.

Same-named event blocks begin at 4007, 4427 and 4457, with `Type = "Event"` and literal names `UNIT_ATTACK_POWER`, `UNIT_RANGED_ATTACK_POWER`, `UNIT_SPELL_HASTE`. They were not used as function evidence. Existing argument policy is correctly excluded from row credit; cached declarations do not establish native execution.

### 3. Provider chains, wrapping and haste disagreement

All three chains use `src/lua_api/globals/unit_stats.rs`:

- 191–193: `unit_token_at` converts the stack value and uses `unwrap_or_else(|| "player".to_string())`.
- 196–207: `unit_token` reads slot 1; `stats_for` calls `stats_for_unit`, which borrows simulator state and calls `lookup_unit_stats`.
- 150–170: lookup maps `"player" | "pet" => player_stats(&sim.player)`; target/focus use their snapshots, party uses `parse_party_index`; mouseover and absent units use default stats.
- 82–93: player snapshot computes `let ap = (player.stats.strength + player.stats.agility) + level_f * 10.0`, stores `attack_power: ap as i32`, and copies `haste_rating`.

502, 307–312: three separate `push_stat_number` calls for `stats.attack_power as f64`, `0.0`, `0.0`; `Ok(3)`.

508, 316–318: `unit_ranged_attack_power` calls `unit_attack_power(state)`. Three wrapped result positions through the same three call sites, not a distinct ranged model.

512, 335–338: `push_stat_number(state, rating_to_percent(stats.haste_rating))`; `Ok(1)`. Conversion at 567–569 is `(rating as f64) / 180.0`.

Thus 3 + 3 + 1 = seven wrapped returned positions across the three queries, with four distinct wrapping call sites because ranged delegates.

Wiring: `UNIT_STAT_GLOBALS` at 607–616 binds the names to these functions; 630–637 registers its entries. External caller `src/lua_api/globals/register.rs:282` invokes `super::unit_stats::register_all(lua)?`. Restriction tests are included under `retail-12-0-5` at `tests/character_stats.rs:9–11`.

`src/c_api/c_secrets.rs:67–79` reads `unit_stats_restricted` under `retail-12-0-5`, uses `wrap_host_secret_number(state, number)` when true, otherwise `Val::Num(number)`, and pushes one result. Other epochs push plain numbers. The test assertions, rather than the helper comment alone, substantiate caller-taint expectations.

Haste disagreement is real: `src/lua_api/state_types/character_world.rs:111–113` computes `self.haste_rating as f64 / 170.0`. `real/combat_stats.rs:143–150` uses this for GetHaste/GetMeleeHaste; 153–165 delegates GetRangedHaste outside Forever. Forever has a distinct two-plain-result branch. B85's retail comparison is accurate; no native divisor is established.

### 4. Fixtures and assertions

`tests/character_stats/stat_restriction.rs:12–20` seeds level10, strength300, agility200, armor1234, crit360 and haste510. Consequently player AP is `300 + 200 + 10*10 = 600`, with no truncation loss for this fixture; spell haste is `510/180`.

`stat_restriction_fixtures.lua:35,38,40` contains precisely:

- `{'UnitAttackPower', {'player'}, {600, 0, 0}}`
- `{'UnitRangedAttackPower', {'player'}, {600, 0, 0}}`
- `{'UnitSpellHaste', {'player'}, {510/180}}`

Its 44–57 use `n = select('#', ...)`, assert `actual.n == #expected`, and check each ordered result's secrecy, access and numerical value with `math.abs(result - value) < 1e-10`; restricted values are trusted-unwrapped first. Values are tolerance-checked, not bitwise-equality-checked.

`stat_restriction_all_supported_outputs_preserve_values_arity_and_toggle`, Rust 39–64, invokes this loop plain → restricted → plain. `stat_restriction_tainted_callers_receive_opaque_host_results`, 68–100, checks fixture arity, every result secret/inaccessible, failed unwrap/arithmetic and unchanged `StatRestrictionProbe` taint. That tainted inner loop does not check payload values; the trusted loop does, including at line97 afterward.

`stat_restriction_absent_unit_shapes_and_constant_outputs`, 104–137, includes `UnitAttackPower('missing')` at 112, checks every result is secret and unwraps to zero at 117, and checks arity3 at 124. Among the three B85 candidates, it calls only 502. It also tests other APIs; B85 does not claim otherwise. No 508/512 missing-unit proof follows.

### 5. Proof-reuse boundary and compiled-source uncertainty

Ran both requested git diffs with explicit cwd. At initial requested HEAD, `tests Cargo.toml Cargo.lock` was unchanged; `src` had only 16 additions in `src/c_api/aura_duration.rs`. Because a combined output lost its leading results and HEAD subsequently moved, also established the stable boundary explicitly: `git diff d21d4a208 68e76142a -- tests Cargo.toml Cargo.lock` returned empty, exit0. `git log d21d4a208..HEAD -- src` identified only `608d52558 Add C_UnitAuras.DoesAuraHaveExpirationTime` at the inspection point.

`git show 608d52558` adds namespace registration and a boolean query. Current source 47–52 registers it; 61–68 reads public arguments, finds a matching aura and returns `aura.expiration_time != 0.0` or false. No stat provider, player-stat model, restriction flag, wrapper or restriction test is edited. Its query body is not on these three provider chains. Registration adds another namespace callback during setup; this is not evidence of byte-identical whole-runtime construction.

Commit message states: “Previously carried as an uncommitted change through the 12.0.5 audit builds; registration call reflowed by hand to match rustfmt layout”. That corroborates dirty aura provenance but is a historical assertion, not preserved compiler input.

B82's durable record at `docs/specs/spell-bonus-stat-security.md:68` states “12 distinct refreshed PASS =7 new+4 existing restriction controls+1 proxy control” and producer `d21d4a208`; line72 explicitly says “Dirty-combined provenance/guarded source equality is not clean whole-tree identity” and retains the unowned aura formatting issue. Coverage capability at `12.0.5-page-coverage.json:1409–1424` likewise records d21d4a208 and qualified dirty/global-format gaps. These records do not preserve the actual historical aura patch bytes. They permit scoped stat-source continuity, not proof that committed 608d52558 is byte-identical or wholly equivalent to every then-compiled source input. Hand reflow alone plausibly changes layout, but its exclusivity cannot be independently proven from this record.

Final moving-HEAD qualification: at a342c6eec the requested test/Cargo diff is nonempty because of the new unrelated aura test file. Therefore “empty through current HEAD” fails literally at audit end; the B85 supplement-time claim at 68e76142a passes. No new executed proof is implied.

### 6. Evidence class and overstatement

Direct Python glob of `/tmp/patch-12.0.5-*` returned `[]`. Historical raw proof streams, executable hashes and timings could not be independently checked on this host.

B85:242 explicitly cites a “committed B82 aggregate record” and says “No rerun; raw artifacts unreadable on this host.” It does not fabricate three new tests, individually named historical result lines or fresh GREEN. Evidence validated: fresh static source/fixture/wiring inspection, read-only git continuity, cached declaration inspection and durable committed acceptance/accounting records. Historical aggregate PASS remains record reuse, not independent re-execution or compiled-source reconstruction. B82's qualified check at spec:72 is not a retained direct check exit0.

### 7. Limits, placeholders and accepted precedent

B85:246–248 explicitly limits synthetic AP, zero buff components, shared ranged values, inconsistent haste conversions, one seeded player case, unverified selectors/nonplayer/absence/profile/activation/native behavior. These are accurate.

Crediting the three positions does not require native positive/negative buff models: `Exact-eight` at the same spec:114–115 describes four zero positions for UnitDamage and shared bounds/zero modifiers for UnitRangedDamage; acceptance:144 expressly includes “all21 numeric result positions”. Limits:133–138 and146 retain placeholder/native/component/mutation gaps. Shared getter precedent at113 says “Shared getter is legitimate local annotation backing, not native equivalence”; B76:154 similarly accepts computed existing-model outputs while rejecting native-formula and activation inference. B85 is consistent: a state-backed primary result plus wrapped placeholder positions is not a zero-placeholder-only model claim.

Additional qualifications: ordinary omitted/nil/wrong-type unit arguments are not covered. `unit_token_at` defaults failed conversion to player, so declared nonnil UnitToken/signature compliance must not be inferred. Other player values, live stat mutations, modifiers and fractional/AP truncation boundaries also lack exact restriction-case proof here. Source continuity plus historical aggregate tests does not prove clean whole-tree compiled equivalence. These are limits, not demonstrated failures of the bounded output annotation. No in-scope provider or fixture description was falsified.

### 8. Deferrals, no credit

478: `unit_stats.rs:517–520` calls `stats_for` and wraps `stats.armor`. Its selector path reads slot1 and defaults to player; fixture Lua:27 expects1234. Cached `PlayerScriptDocumentation.lua:849–857` declares a Function, restricted output and one nonnil number, with no Arguments. Armor is a real stored field but not a shield-block model. B85's description and refusal to equate wrapping with meaningful shield-block behavior are substantiated.

504: `unit_stats.rs:369–372` pushes wrapped literal `2.0`, `2.0`, returns2, and reads no unit or stat state. Fixture Lua:36 expects `{2,2}`. Calling it constant means its numerical payloads, not its secrecy flag. Deferral is consistent with the absence of a modeled attack-speed producer. Existing fixture assertions alone do not require either row to receive credit.

### 9. Accounting

`12.0.5-page-coverage.json` rows:

| Row | Lines | Status | Capabilities |
|---|---|---|---|
| 502 | 3265–3268 | audit-pending | [] |
| 508 | 3285–3288 | audit-pending | [] |
| 512 | 3299–3302 | audit-pending | [] |
| 478 | 3179–3182 | audit-pending | [] |
| 504 | 3271–3274 | audit-pending | [] |

Each note says “Source retained; behavioral applicability audit not completed.” Parsed counts: 362 source rows; 162 audit-pending, 182 bounded-coverage, 11 partial-development-green, 7 metadata-only; 88 capabilities. These remained the observed counts after the concurrent test-only commit. No B85 capability or promotion is already present.

## Final verdicts

- **502 — ACCEPT WITH QUALIFICATIONS:** output annotation only for seeded player `{600,0,0}`; synthetic AP and literal-zero buff components, no native decomposition/selector/general-input/mutation/profile parity; historical aggregate PASS reuse only, not current execution or byte-identical compiled-source proof.
- **508 — ACCEPT WITH QUALIFICATIONS:** same bounded annotation mechanism for seeded player `{600,0,0}`; melee provider reused, no separate ranged model; zero components and general-input/nonplayer/absence/native/profile limits; same historical-proof qualifications.
- **512 — ACCEPT WITH QUALIFICATIONS:** seeded player `{510/180}` and one wrapped position only; local 180 divisor disagrees with the separate 170 conversion, neither native-verified; no general-input/nonplayer/absence/mutation/profile parity; same historical-proof qualifications.

Requested revision was clean and substantiates the supplement-time continuity claim; final HEAD moved externally, so do not transplant the empty-test-diff assertion to a342c6eec. Did not run or verify current tests/build/check/startup, historical raw logs/hashes/timings, actual historical compiler input equivalence, native behavior, activation or older profiles. No scope beyond these rows is recommended.
