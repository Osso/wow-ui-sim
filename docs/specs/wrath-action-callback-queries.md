# Wrath action callback queries

Native Wrath action, stance and pet callbacks must read profile-correct query results from simulator state. Native fixture coverage is described in [the callback fixture](../../tests/fixtures/wrath-action-callbacks.md); the shared range backing model lives under `src/c_api/` without publishing modern functions on Wrath.

## What it must do

### Range snapshots and publication

- [x] Supplied spell/target GUID range snapshots can be seeded by Wrath fixtures without publishing `C_ActionBar.IsActionInRange`, `HasRangeRequirements` or `GetActionAutocast` on Wrath. Modern registration is selected by Retail's `retail-12-0-0` and Forever; the shared backing model uses the `action-spell-range` capability.
- [x] Legacy `IsActionInRange(slot[, unit])` returns exactly one numeric 1/0 or nil; never boolean. Valid supplied in-range/out-of-range snapshots drive original `ActionButton_OnUpdate` hotkey colors.
- [x] Invalid/missing slot or index outside 1..144 returns one nil; ordinary decimal numeric strings are accepted. Empty/unresolved action, target or snapshot returns nil. Omitted unit uses current target for directly bound spells.
- [x] No special unranged-action result is invented. **INFERRED host model:** absent supplied range/eligibility data remains indeterminate; snapshots represent the result of the shared native range/target-validity checker, not computed geometry.

### Stance and pet shapes

- [x] Wrath `GetShapeshiftFormInfo` returns texture, name, numeric 1/nil active and castable flags; active forms are castable. Invalid zero/nonnumeric arguments error; out-of-range forms return zero values. Native ShapeshiftBar_UpdateState checks the selected button.
- [x] Wrath `GetPetActionInfo` returns seven values name, subtext, texture, token, active, autocast-allowed, autocast-enabled; empty slots return seven nils and nonnumeric input errors. Spell subtext is empty when unsupplied, spell active/token flags are nil, other flags are numeric 1/nil. Token names/textures are supplied native token strings, subtext/autocast flags nil, active numeric 1/nil.
- [x] `IsPossessBarVisible()` returns exactly one boolean from existing supplied possess visibility; `UPDATE_BONUS_ACTIONBAR` drives the native PossessBarFrame show/hide callback without a getter-side vehicle/bonus-bar gate.
- [x] Retail query values and registration remain unchanged; other non-Wrath branches retain their existing projections.

## How it works

- [Lua API state architecture](../wiki/systems/lua-api.md)
- [F04/F05 native callback fixture](../../tests/fixtures/wrath-action-callbacks.md)

## Implementation inventory

- `src/c_api/action_spell_range.rs` — supplied spell/target snapshot type; no Lua registration.
- `src/c_api/c_action_bar_queries.rs` — shared Retail/Forever query registration and readers; not published on Wrath.
- `src/lua_api/state{,/sim_state}.rs` — profile-scoped snapshot storage.
- `src/lua_api/globals/real/pet_bar.rs` — Wrath seven-value spell/token projection; modern nine-value output preserved.
- `src/lua_api/globals/real/shapeshift.rs` — Wrath texture/name/numeric-flags projection; modern texture/boolean-flags/spellID preserved.
- `src/lua_api/globals/real/wrath_legacy/action_bar.rs` — native legacy numeric/nil range and boolean possession adapters over supplied state; no C_* registration.
- `src/wrath/compat_bootstrap.lua` — obsolete nil-only range shim removed.

## Tests asserting this spec

- `tests/wrath_action_contract.rs` — sourced return shapes, unknown/invalid inputs and publication isolation.
- `tests/forever_action_range.rs` — Forever's selected legacy wrapper reads the shared supplied range observations; direct range-requirement and pet-autocast queries reuse their backing state.
- `tests/{c_pet_bar_globals,c_shapeshift_globals}.rs` — direct producer oracles select native Wrath positions/arity without weakening modern assertions.
- `tests/common/wrath_action_assertions.rs` — original native ActionButton_OnUpdate, ShapeshiftBar_UpdateState and PetActionBar_Update consumers, reached through original F04 test IDs.

## Bounded F04 proof

Original 38 cases all reproduced FAIL at `53bc1e85d`. Native fixture/source-bound consumer RED at `cd76d412f` exposed range, pet, stance and possession producer defects before behavior repairs. Final bounded Wrath coverage: 34 retained F04 PASS, four documented modern-only gates, four native contract/publication PASS and 34 direct producer controls PASS. Seventy-one unchanged cases passed at `83f7dfb2b`; the sole later delta corrected the admin test's modern spellID oracle, and that case passed 1/1 at `acd7e24e4`. Production and other test bodies are unchanged across that delta.

Retail preservation: identical 122-case sets before/after, 121 PASS and one unchanged FAIL (literal missing `Blizzard_ActionBar_Mainline.toc` in the Retail cache), with identical failure cause and zero new regressions. Model-only and final stock startup `lua-errors` are `[]` on both profiles. Raw modern range publishers remain absent on Wrath. Every executed selection has positive completed-test counts; sealed ELFs contain this worktree's test paths only. Original CVars' size/mtime/inode/SHA256 are unchanged.

Receipts are retained in the private handoff for this batch. These are bounded simulator/FrameXML proofs, not native engine execution or independent final-review acceptance.

## Known gaps (current cycle)

- [ ] Numeric-string coercion beyond ordinary decimal input (for example hexadecimal strings) is not modeled here; exceptional/out-of-int conversions remain unverified.
- [ ] Native aura-236 possession visibility exception is unmodeled: existing supplied possess boolean has no possess-spell/aura record. No missing-spell behavior is claimed.

## Out of scope

Native engine execution, range geometry/eligibility computation, unsupported macro stored-target sentinel distinctions, missing spell/item records, active-icon lookup, pet command/mode inference, pet-presence reconstruction and unrelated client profiles. Existing supplied pet/form records remain fixture inputs, not a claim that modern default spell IDs are 12340 game data.

Authoritative source: laptop static 12340 notes `handoff/laptop-labels-answers-20261018/{action-inrange-12340.md,pet-shapeshift-info-12340.md,possess-bar-visible-12340.md}` (2026-10-20). Native consumer positions: cached `FrameXML/{ActionButton.lua:458+,BonusActionBarFrame.lua:152+,PetActionBarFrame.lua:106+}`. Static reconstruction is not native execution.
