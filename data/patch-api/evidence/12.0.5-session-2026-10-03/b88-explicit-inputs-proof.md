# B88 independent read-only verification
Commits: `be7c0cc2c`, `62d0ce70f`.

## 1. Producers — PASS
`git show 62d0ce70f --stat`: "3 files changed, 45 insertions(+), 23 deletions(-)".
All ten producers read their corresponding inputs: block_chance, shield_block,
hit_modifier, spell_hit_modifier, expertise, expertise_percent,
mod_resilience_damage_reduction, pvp_power_damage, pvp_power_healing,
and pet.spell_bonus_damage. None remains a fixed constant (pet None intentionally yields zero).
Evidence: "push_player_stat(state, |stats| stats.hit_modifier)",
"push_player_stat_triple(state, |stats| stats.expertise_percent)",
"push_stat_number(state, value)?", "pet.spell_bonus_damage.unwrap_or(0.0)",
and direct block/shield calls to push_stat_number.
Diff changes no other producer behavior. Removed get_restricted_zero_percent had
exactly three callers, all among the requested functions. stats_for remains used
by many other producers; its removal from shield block does not make it dead.
GetShieldBlock now ignores arguments instead of selecting unit armor.

## 2. Fields and profiles — PASS (static inspection)
Input commit adds nine ungated CharacterStats fields and ungated
"pub spell_bonus_damage: Option<f64>". Corresponding producers/registrations
are also ungated; nearby retail-only pet melee haste gates do not cover spell bonus.
CharacterStats and PetState derive Default: scalar/array inputs default to zero,
pet optional input to None. base_stats uses "..Self::default()"; compute starts
from base_stats, so new fields require no additional struct initializer entries.
No field/producer cfg mismatch found. Shared registrations also affect older
profiles; this is not retail-exclusive behavior. No profile compilation performed.

## 3. Tests and requirements — PASS with coverage qualifications
All loops have concrete nonempty fixtures; helpers check arity, every expected
value and secrecy. No vacuous test identified. Nine tests reviewed:
- unconfigured_inputs_return_public_zeros_with_declared_arity: "&[0.0, 0.0, 0.0]";
  old zero producers can satisfy most assertions, but old shield armor cannot.
- each_scalar_reads_its_own_configured_input: "GetShieldBlock(), 79.0" and eight
  distinct nonzero inputs; old constants/proxy fail.
- scalar_inputs_update_live_without_moving_neighbours: "hit_modifier = 3.5",
  spell-hit still 2.75; old producers fail.
- shield_block_is_not_armor_and_takes_no_unit: armor becomes 9999, shield stays
  79; target/missing arguments still 79; input then 83. Old producer fails.
- expertise_triples_are_ordered_and_independent: "[11.0, 23.0, 37.0]" versus
  "[1.5, 2.5, 3.5]", independent component updates; old producers fail.
- pet_spell_bonus_is_optional_and_independent_of_player_stats: 41.5 then 63
  then None/zero, intellect mutation irrelevant; old constant fails.
- environments_do_not_share_inputs: configured block 12.5 versus fresh zero;
  old constant fails. Isolation explicitly sampled for block/expertise only.
- restriction_toggle_preserves_configured_values_and_arities: "[false, true, false]",
  all ten fixtures, secretunwrap equality and arity; old producers fail.
- restricted_tainted_callers_receive_opaque_configured_outputs: all ten fixtures,
  "not pcall(secretunwrap, value)", arithmetic failure, taint unchanged, then
  trusted value checks. Old constants would pass opacity alone but fail final values.
Seven requirement mappings:
1. Live own input/default/arity: default, scalar, triple, pet and toggle tests.
2. Independence: scalar-neighbours, triple and pet tests. Not every possible
   single-field mutation/cross-field combination is explicitly exercised.
3. Shield versus armor/ignored arguments: shield test.
4. Pet independence/optional input: pet test; no explicit pet-presence assertion
   proves "does not require or create a pet" (producer is read-only by inspection).
5. Per environment: isolation test (sampled inputs, not all ten).
6. Secret host numbers/value/arity/toggle: restriction-toggle test.
7. Tainted opacity and unchanged taint: restricted-tainted test.
No whole requirement lacks assertions; pet noncreation subclause lacks a direct assertion.

## 4. Prebuilt execution and saved logs — PASS
Requested source diff against HEAD is empty; worktree status inspected.
Command: `timeout 90 target/debug/deps/integration-8ea324359263a4d2 character_stats:: --test-threads=1` (repo cwd).
> test result: ok. 44 passed; 0 failed; 0 ignored; 0 measured; 10030 filtered out; finished in 6.65s
Exit code 0; clean worktree. All nine new tests execute and pass.
Saved RED: "test result: FAILED. 33 passed; 11 failed; 0 ignored; 0 measured; 10030 filtered out; finished in 6.54s".
Saved GREEN: "test result: ok. 79 passed; 0 failed; 0 ignored; 0 measured; 9995 filtered out; finished in 12.25s".
Saved startup stdout ends with "[]" (preceded by native source/artifact metadata).
Logs match claimed counts and empty error array; revision attribution is supplied
context, not cryptographically established by these result lines.

## 5. Shield-block regressions — PASS (static caller inspection)
rg unavailable; used Python file reads over src, tests, Interface/AddOns and
retail cache, searching Rust/Lua/XML/YAML/text source files. No addon caller
found in Interface/AddOns. Unit-argument calls occur only in the new tests.
No caller asserts equality with player armor; old restriction fixture deliberately
changes from 1234 to zero. Existing unit_stats test checks nonnegative numbers.
Retail PaperDollFrame.lua:760 calls "local shieldBlockArmor = GetShieldBlock()"
without arguments. It passes this independent armor-like amount through
PaperDollFrame_GetArmorReduction and its against-target counterpart, then
"statFrame.tooltip2 = CR_BLOCK_TOOLTIP:format(blockArmorReduction)".
The helpers invoke C_PaperDollInfo.GetArmorEffectiveness / AgainstTarget and
multiply results by 100; no division by GetShieldBlock or equality-to-armor assumption.
Thus zero changes displayed mitigation, not Lua value type/arity. No harmful
zero-specific error found; saved startup error array is empty. Panel interaction
itself was not executed, so absence of all possible runtime errors is not claimed.

## 6. Spec honesty — FAIL (stale status, not implementation defect)
Known-gaps text still says "Inputs only: producers still return constants or armor;
no compiled RED or GREEN recorded yet." This is contradicted by producer commit
and saved RED/GREEN evidence. Must remove/update this obsolete gap before claiming
spec status accurately represents completed work.
Other limits are candid: "configured inputs do not survive a recompute";
"Older profiles share these producers and inputs but no older-profile execution
or parity is claimed"; native formulas, automatic restriction and native parity
are out of scope. Recompute sites include admin_equipment and C_EquipmentSet,
which replace the snapshot. No recompute-persistence promise made.
No-argument declaration does not assert rejection of extra arguments; spec
explicitly says "any argument is ignored", and target/missing tests prove that policy.
Pet noncreation is supported by read-only producer inspection, not a direct
presence assertion. Independence/isolation claims have sampled behavioral coverage
plus distinct state-field reads, not exhaustive mutation permutations.

## Verdict — ACCEPT WITH QUALIFICATIONS
Implementation fits the bounded explicit-input contract; no merge-blocking
producer defect found. Merge risk: default shield mitigation display changes to
zero and gear/equipment recomputation clears configured inputs (documented policy).
Qualifications:
1. Update stale spec known-gap status; it contradicts shipped producer source/proof.
2. Add explicit pet-presence/noncreation assertion if claiming that subclause has
   direct test coverage; isolation and independent mutation coverage are sampled.
3. Profile compatibility and retail panel zero-safety were inspected statically,
   not proved by older-profile compilation/execution or panel interaction.

Not verified: cargo checks, builds, formatters, full suite/CI, native WoW parity,
older-profile runtime, GUI/panel interaction, or binary-to-commit provenance.
Only requested character_stats prebuilt filter rerun; 79-test GREEN and startup
are inspected saved evidence, not independently rerun. No repository edits,
commits, agent/model calls, Bash, or operational changes performed.

Additional step-5 evidence: src/c_api/c_paper_doll_info.rs:124-126 explicitly
handles zero: "if armor <= 0.0 { return 0.0; }". This supports zero-safe
mitigation computation, without claiming panel execution proof.
