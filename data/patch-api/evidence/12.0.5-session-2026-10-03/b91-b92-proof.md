# B91/B92 independent read-only proof — 2026-10-03

## Revision and proof boundary
HEAD observed `ca7e6778e6a0b564241f5326cad7de6b01e23cfc`.
`git diff ca7e6778e HEAD -- src tests Cargo.toml Cargo.lock build.rs` empty, exit 0.
Working tree had unrelated untracked `docs/specs/aura-duration-object.md` and `tests/aura_duration_object.rs`; neither edited or included in proof.
No builds, formatters, commits, agents, model CLIs, or repository writes.
Prebuilt binary provenance is assumed from supplied path and allowed diff check, not independently reconstructed.

## Part A — B91
`git show --stat cbfe0ad8e`: only tests/font_api.rs (+70) and tests/housing_catalog_base_lookups.rs (+41); no producer change.
Independent prebuilt runs, cwd repository, timeout 90, --test-threads=1:
- `integration-8ea324359263a4d2 font_api::`: 45/45, exit 0 (12.15s).
- `integration-8ea324359263a4d2 housing_catalog_base_lookups::`: 15/15, exit 0 (3.99s).

### 547 — numeric height
Non-vacuous: tests/font_api.rs:791-794 iterates `{13.2, 0.5, 40}`, asserts `fs:SetFont(path, height, nil) == true`, then `type(got) == "number" and math.abs(got - height) < 0.00001`.
This checks real round-trip state, not merely a truthy result. Fractional numbers are a fair bounded example of `uiUnit -> number`, not proof of a semantic distinction: uiUnit might itself already permit fractions.
Cached SimpleFontStringAPIDocumentation.lua:509 says `Type = "uiFontHeight"`, not the historical `number`; later declaration cannot establish 12.0.5 numeric-domain semantics.
Unasserted: complete number domain, range/invalid-height validation, precision beyond these samples, native unit-conversion behavior.

### 548 — nilable flags
Non-vacuous: tests/font_api.rs:769-778 seeds OUTLINE; `SetFont(path, 18)` and `SetFont(path, 20, nil)` both succeed, heights change, and `flags == "OUTLINE"`; empty string explicitly clears outline.
Literal omitted/nil acceptance is asserted. Keeping old flags is existing simulator behavior, not independently established native semantics from Nilable=true.
Unasserted: other flag combinations, invalid flags, native nil/reset policy.

### 549 — success return
Non-vacuous shape: tests/font_api.rs:743-744 `select('#', ...) == 1` and `(...) == true`; repeated valid-shaped calls followed by state assertion :749. :751 asserts exactly one false for nil/table paths; :756 checks unchanged state.
Producer formatting.rs:343-364 rejects a non-string path, ignores nonnumeric size (:348-350), never resolves font asset, and otherwise pushes true and returns one result.
“Success” is effectively “path was a string,” NOT meaningful confirmation of a resolvable font and valid height. Unresolvable string paths and nonnumeric heights can report true.
Cached declaration :502-503 explicitly requires valid asset and height; these later declarations add context but are not historical native execution proof.
Credit: partial only—return arity/boolean presence is covered, semantic success/failure is not. Tests never try an unresolvable string path or nonnumeric height.

### 645/646 — removed entry-ID fields
Non-vacuous fixture: tests/housing_catalog_base_lookups.rs:362-365 calls record lookup, `assertChair(info)`, then `rawget(info, field) == nil` for both removed fields.
:368-382 requires THREE variants, matching record/type, `count == 3`, absent removed fields on every entryVariantID, and `seen[0] and seen[1] and seen[2]`.
:384-388 supplies `{recordID = 81003, entryType = decor, entrySubtype = 999, subtypeIdentifier = 999}`, asserts three results and unchanged per-index variant identifiers.
These are real populated outputs and input selection assertions; no empty-loop pass.
Unasserted: every catalog API/seed/wrapper output, each removed input independently, native rejection versus ignoring obsolete extras. Variant-ID coverage also exceeds the exact base-structure removal delta.
Full Python text search of src for `entrySubtype` or `subtypeIdentifier`: **zero hits** (therefore no remaining emitting producer, seed or wrapper spelling either field was found).

## Part A provisional row verdicts
- 547: ACCEPT WITH QUALIFICATIONS — sampled plain-number round-trip; no proof that uiUnit formerly excluded fractions or complete numeric validation.
- 548: ACCEPT — literal nilability covered; preserve-flags policy/native parity not claimed.
- 549: ACCEPT WITH QUALIFICATIONS — PARTIAL CREDIT ONLY for arity/boolean shape; no semantic success credit.
- 645: ACCEPT — populated absence and ignored obsolete-input coverage; not exhaustive over all APIs.
- 646: ACCEPT — same bounded scope as 645.

## Part B — B92 / row 504
Independent prebuilt run: `timeout 90 integration-8ea324359263a4d2 unit_attack_speed:: --test-threads=1`, cwd repository: **4/4, exit 0**, 1.96s.
Read supplied logs: b92-red.log contains 0/4; b92-green.log contains 73/73; b92-startup.stdout contains `[]`. These historical logs are corroboration, not independently rerun startup or 73-test evidence.

### Producer and other behavior
ca7e6778e diff adds only swing-time fields/defaults to UnitStats snapshots and changes UnitAttackSpeed; existing snapshot fields and other producers remain unchanged.
unit_stats.rs:383-389 reads `stats_for(state)`, passes main and every present off-hand number through `push_stat_number`, pushes `Val::Nil` directly for absence, then `Ok(2)`.
c_secrets.rs:67-79 wraps actual host numbers only under `retail-12-0-5` plus explicit restriction flag; other profiles stay plain. Every numeric branch—including unknown-unit zero—uses this helper.
Player snapshot :104-105 copies explicit inputs; target :131-132 and party :155-156 seed synthetic 2.0/None; unknown-unit default :77-78 is 0/None.
No other query reads the two new UnitStats fields; therefore no other query's outputs changed by adding those fields. UnitRangedDamage intentionally retains its separate constant speed.

### Removed UnitExists assertion
unit_api.rs:3-7 parses party1 to index 0. unit_stats.rs:163-189 uses `sim.party_members.get(index)` without testing `party_group_active`.
group_queries.rs:423-440 makes UnitExists(party1) require BOTH active group and in-range roster index.
state.rs:781-784 seeds `default_party()` but sets `party_group_active = false`. The roster can exist while UnitExists reports false.
This disagreement predates B92: the unchanged lookup also feeds UnitArmor (:309-316), UnitDamage (:357-366), and other unit-stat queries; parent-revision source confirms the same lookup.
Removing the assertion was legitimate for this literal snapshot-based contract, not hiding a new B92 resolver defect. B92 newly applies that existing resolver to UnitAttackSpeed instead of returning constants; it inherits the old discrepancy. Native existence consistency remains unproven.
Important historical-proof limit: the original RED `other_units` case failed at UnitExists before reaching speed assertions. It was not then evidence against the old speed producer. The CURRENT case independently fails the old producer at the party nil off-hand (and unknown 0/nil).

### Four tests / four spec bullets
All tests use concrete expectations, not empty fixtures or vacuous checks. Helper tests/unit_attack_speed.rs:13-21 asserts exactly two results, main secrecy/value, and either plain nil or off-hand secrecy/value.
| Spec bullet (explicit-stat-inputs.md) | Assertions / old constant producer | Missing direct assertions |
|---|---|---|
| :63 live player inputs and base pair | :41-48 checks 2/2, then 2.4/1.7, then 2.8/1.7; old producer fails configured values. | Off-hand-only update, per-environment isolation, recompute behavior. |
| :64 plain nil second result | :54-58 checks 3.1/nil unrestricted AND restricted; helper enforces arity and non-secret nil. Old producer fails main and off-hand. | Tainted caller specifically with missing off-hand. |
| :65 target/focus/party synthetic and unknown zero, independent of player | :64-69 sets player 2.4/1.7, asserts party1 2/nil, establishes missing-unit nonexistence, asserts 0/nil. Old producer fails party off-hand and unknown results. | Target/focus present and absent; other party members; restricted party/unknown; pet alias. Target/focus supported by inspection only. |
| :66 secret numeric values, toggle, opacity and taint | :80-85 toggles false/true/false preserving 2.4/1.7. :92-105 checks two opaque secrets, denial of unwrap/arithmetic, taint retained inside and nil outside. Old producer fails values before opacity check. | All selector branches under restriction; automatic activation/native policy. |
Old producer already wrapped constants; the new opacity test strengthens proof of the shared mechanism, not proof that B92 invented secrecy.

### Other profiles / default initialization
CharacterStats fields (:53-55), base seeds (:83-84), UnitStats fields and UnitAttackSpeed producer are ungated.
state.rs:267 initializes player with `PlayerState::seeded()`; character_world.rs:329-342 creates `stats = CharacterStats::compute(...)`; compute :70 calls base_stats :77, setting `2.0` and `Some(2.0)`.
Thus default player still returns two positive speeds, satisfying client-mists test unit_stats.rs:109-125 by inspection. Derived CharacterStats::default() alone yields 0/None; it is NOT the seeded player path.
No older-profile build/test was run; retail-only new tests cannot prove older-profile execution.

### Cached Blizzard consumers
Full cached retail *.lua search found only generated declaration/event entries plus two actual calls, both in Blizzard_UIPanels_Game/Mainline/PaperDollFrame.lua.
SetDamage :813 takes the pair; :875 stores main; :879 guards off-hand work with `offhandSpeed and minOffHandDamage and maxOffHandDamage`; :900 clears absent off-hand.
SetAttackSpeed :910-917 formats main directly and formats/concatenates off-hand only if present. Numeric main zero is format-compatible; nil off-hand skips that branch.
CharacterDamageFrame_OnEnter :1877 formats main; :1880-1884 guards off-hand formatting. No division by attack speed found in these consumers.
Consequently no nil-off-hand or zero-main-specific Lua error is indicated by inspection; zero displays as zero and nil removes off-hand display. NOT runtime paper-doll/tooltip interaction proof.

### Spec honesty and merge risk
B92 spec :61 correctly identifies constant old producer; :63-66 label seeds and synthetic values as simulator policy. :68 discloses missing/non-string fallback and unmodeled AllowedWhenUntainted selector policy. :77-79 excludes formulas, recompute persistence, automatic activation, native parity and older-profile proof.
Four B92 requirement checkboxes remain unchecked; development proof is also unchecked pending verification/accounting. Honest pending status, not completed native-parity claim.
Qualification needed if checking :65: target/focus are source-inspected, NOT directly tested. Party snapshots are roster-based even while UnitExists=false; spec does not explicitly describe inactive-group discrepancy.
Risk of merging reviewed changes: B91 no runtime change; weak SetFont success meaning remains pre-existing. B92 changes nonplayer/unknown outputs from 2/2 to synthetic 2/nil or 0/nil, and supports configured player inputs. Inspected retail consumers tolerate those shapes; older-profile nonplayer behavior and native semantics remain unproven.

### Concurrent revision boundary
HEAD advanced during verification to `b9b9eeec8a82b680b87d575260955900ba42e2cf`; restricted code/test diff now includes `tests/aura_duration_object.rs`.
Final diff confirms ZERO changes to reviewed B91/B92 producer, resolver, state, helper, test, and spec files listed above.
No additional prebuilt runs after observing this advancement. Binary results cover supplied B91/B92 baseline, NOT the newer HEAD as a whole. Later aura work is outside this report.

## Final verdicts
| Row | Verdict | Qualification |
|---|---|---|
| 547 | ACCEPT WITH QUALIFICATIONS | Bounded numeric round-trip; historical uiUnit distinction and full numeric domain not proved. |
| 548 | ACCEPT | Literal omitted/nil flags covered; no native preservation-policy claim. |
| 549 | ACCEPT WITH QUALIFICATIONS | PARTIAL CREDIT: exact boolean return shape only; “success” effectively string-path acceptance, not valid font installation. Full semantic-success claim rejected. |
| 645 | ACCEPT | Concrete populated absence; no src hits; sampled producers/input compatibility, not exhaustive runtime coverage. |
| 646 | ACCEPT | Same bounded absence/input proof as 645. |
| 504 | ACCEPT WITH QUALIFICATIONS | Explicit player inputs and restriction proved; target/focus inspection-only; pre-existing roster/existence discrepancy inherited; no older-profile/native/automatic-policy proof. |

## Not verified
No builds, cargo checks, formatters, full suites, old-revision execution, independent startup rerun, native WoW execution, font resolution/height validation, exhaustive catalog output matrix, older-profile execution, actual paper-doll interaction, or current newer-HEAD acceptance.
Report is bounded source inspection plus 64 independently passing prebuilt tests (45 + 15 + 4), supported by—not substituting for—the supplied historical logs.
