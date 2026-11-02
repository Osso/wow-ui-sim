# Additional action-state discrepancies

Static comparison at the [expanded-pass revisions](additional-rendering-and-map.md). Original legacy exports are compared to the simulator's actual modeled/global/namespace paths, not declared missing merely because modern retail moved a name into `C_ActionBar`. No fixtures were executed. Historical semantics and current profile contracts remain distinct.

## G1 — Attack-action classification is constant false

**External:** `WoW/Source/Ui/ActionBarFrame_Scripts.cpp:459-475`, calls `CGActionBar::IsAttackAction`. `ActionBarFrame.cpp:232-246`, resolves the slot's spell record and tests whether its first effect is 78.

**Simulator:** `src/lua_api/globals/action_bar_api.rs:249-252` unconditionally returns false. `action_bar_api/registration.rs:34-44,77-95` registers the predicate in the actual `C_ActionBar` namespace; a namespace name being callable is not implementation of the classification.

**Source-derived fixture:** slot 1 references a supplied spell record with first effect 78; slot 2 references a different effect. Reconstruction returns true/false; simulator returns false/false regardless of metadata. That producer/metadata state is not yet exposed by this predicate's backing path, so this is a missing model capability, not a runnable regression already demonstrated here.

**Rework/regression:** model the target's attack-action classification from real supplied spell/action metadata and resolve it through slot/action-kind state. Do not hardcode a special spell ID or return true for every occupied slot. Test the two concrete metadata records, empty/invalid slots and macro indirection where supported. Original effect number 78 is historical evidence, not an unverified modern enum contract.

## G2 — Active auto-repeat actions cannot be reported

**External:** `ActionBarFrame_Scripts.cpp:478-494`; `ActionBarFrame.cpp:1824-1835`. Resolves the slot spell, excludes pet spells, and compares against active auto-repeating or auto-ranged combat spell identity.

**Simulator:** `src/lua_api/globals/action_bar_api.rs:254-257` always returns false; the public registration is adjacent to G1. Active casting state alone is not active auto-repeat state.

**Source-derived fixture:** slots 1 and 2 resolve to spells 100 and 200; active repeated spell is 100. Expected true/false; simulator false/false. Changing active repeat identity should change the result without rearranging action slots. This exact backing repeat-state producer is missing; no current runnable fixture is claimed.

**Rework/regression:** implement repeat/ranged-attack lifecycle identity in the spell/action backing model, derive the predicate through slot resolution, and test activation, stop, slot replacement and a pet-spell control. No live combat server is required; use the simulator's supplied-state/input model. Confirm supported-profile treatment of auto-ranged actions rather than guessing from spell names.

## G3 — Action usability equals occupancy; no-mana is always false

**External:** `ActionBarFrame_Scripts.cpp:25-48`, reads distinct `m_slotUsable` and `m_slotNoMana` state. An occupied action can be unusable, and insufficient resource is a distinct result. `ActionBarFrame.cpp:260-398` contains action-kind/spell/item/equipment/state-dependent usability logic rather than an occupancy test.

**Simulator:** `src/lua_api/globals/action_bar_api.rs:414-427` checks only whether the slot exists in `action_bars`, `action_outfits` or `action_macros`, then returns `(occupied,false)`. Real metadata/cost/resource/restriction state cannot affect this result.

**Source-derived fixture:** occupied slot 1 is usable; occupied slot 2 has insufficient resource; occupied slot 3 is unusable for another reason. All three return the same simulator pair. Reconstruction's usable/no-mana states distinguish them. These inputs need the missing backing capability; this is not an assertion that arbitrary flag mutation is an adequate public test boundary.

**Rework/regression:** resolve the action kind and derive usable/resource state from the modeled spell/item/equipment and player conditions. Keep resource failure distinct from other failures. Test supplied concrete costs/resources and restrictions through public query consumers, including transitions without changing slot occupancy. Do not replace the constant with a different plausible default or copy legacy numeric/nil return encoding into a modern boolean contract without target evidence.

## G4 — `UseAction` bypasses cursor placement and non-spell actions

**External:** `ActionBarFrame_Scripts.cpp:216-231`, resolves optional target/button and calls the action dispatcher. `ActionBarFrame.cpp:1307-1390`, first handles a cursor payload as slot placement when appropriate, then dispatches item, macro, equipment-set, pet or spell behavior. Macro dispatch receives the button. `WoW/Source/Ui/UIMacros.cpp:167-168,892-929`, `RunMacro` for the body overload, installs/restores execution context and signals `EXECUTE_CHAT_LINE` for stored non-comment lines; it is not merely an identity setter.

**Simulator:** `src/lua_api/globals/combat_verbs.rs:507-527` reads only `action_bars[slot]` and executes that spell ID. A slot stored in `action_macros`/`action_outfits` is treated as empty; cursor-placement state and target/button arguments are not consulted. Existing populated macro slots are real simulator state: `tests/action_macro_tooltip.rs:79-110` creates/edits macros and installs them through `A_Admin.SetMacroActionSlot` but calls `RunMacro` directly, not through `UseAction`.

Named `RunMacro` currently only sets `running_macro` (`src/lua_api/globals/spell_macro_verbs.rs:131-140`); that test does not prove stored-body execution. The distinct `C_Macro.RunMacroText` executor at `:177-219,394` handles a limited supported command set, including `/target`, via `TargetUnit`. It does not handle the test's `/say` body.

**Counterexamples:** put a macro in slot 1, then use that action; the reconstruction invokes macro dispatch, the simulator logs/returns an empty spell slot. Separately, with an ordinary action cursor payload and no key-state override, use an empty destination slot: reconstruction places the cursor action first, simulator returns empty without consuming/placing it. The latter is a concrete state-transition boundary, not merely missing query flags.

**Rework/regression:** dispatch through the action-kind/cursor model rather than the spell map alone. Preserve per-profile placement and protected-action policy; forward supported target/button context to the selected model. Macro activation identity and stored-body execution are separate boundaries: reproducing the latter requires **new stored-body dispatch wiring** to the existing supported-command executor where applicable, not merely calling today's identity-only `RunMacro`. Equipment outfits and old equipment sets are not automatically identical contracts.

**Regression:** distinguish macro identity activation from body effects. For the body-dispatch requirement, store `/target player` in a macro-backed slot, clear the target, and assert that using the slot selects the supplied player at the correct execution boundary. Direct `C_Macro.RunMacroText('/target player')` is the existing supported-command control (`src/lua_api/globals/targeting_verbs.rs:233-260,294-311` models target/clear); named `RunMacro` is **not** a body-effect control. That future regression needs the new stored-body wiring above; it is not claimed to pass today. Separately, cursor placement must update destination/source/cursor state, and ordinary spell use must still execute once with the intended target. Unsupported macro commands and wider native macro-runtime behavior remain separate gaps.

## Bounded matches and unresolved areas

Ordinary spell occupancy/cursor pickup, basic page getters and selected current-cast queries have intersecting behavior, but no whole-method/native equivalence is asserted. Spellbook selector resolution, pet slots, temporary/bonus pages, action range, consumables/equipped actions, modern input restrictions and macro indirection remain incompletely compared. Calling a method successfully does not classify it as matched. The coverage ledger records narrower checked aspects and unchecked exports separately.
