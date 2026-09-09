# Macro action tooltip directive query

The PTR `C_ActionBar.IsMacroActionWithShowTooltip(actionID)` query reads the body of the macro currently assigned to an action slot. Backing associations and recognition live in `src/c_api/action_macros.rs`; [Lua API architecture](../lua-api.md) describes the runtime boundary. The pinned 12.1.5 register establishes the API occurrence, not the recognition rules below.

## What it must do

- [ ] PTR returns a boolean for positive action slots; empty, spell, outfit, and macros without a matching directive return false. Earlier retail does not expose the query, including through namespace fallback.
- [ ] Query reads current `MacroInfo.body`, so `EditMacro` changes results without reassignment.
- [ ] `A_Admin.SetMacroActionSlot(slot, macroID)` assigns an existing macro; `GetActionInfo` returns `"macro", macroID, nil` and `HasAction` recognizes it. Existing spell assignment positional arguments remain unchanged.
- [ ] Admin clear/replace, cursor macro pickup/place, and `PutActionInSlot` preserve a single action kind per slot and remove obsolete associations.
- [ ] `CreateMacro` stores name/icon/body; `DeleteMacro` clears bound slots and matching cursor/running state without renumbering IDs.

### Simulator assumptions, not native facts

- Recognition is case-sensitive lowercase `#showtooltip`, at the beginning of any LF/CRLF line after optional ASCII spaces/tabs. The token must end the line or be followed by an ASCII space/tab. Remaining text is not evaluated.
- Any positive `u32` slot is accepted. Assignment requires an existing nonempty macro name. Zero slots and nonexistent macro IDs produce errors; no native maximum-slot or coercion claim is made.
- Creation uses the first empty account slot 1–120 or character slot 121–138; deletion leaves a reusable empty slot. These allocation rules are simulator conventions, not verified native behavior.

## How it works

- [Lua API and state](../lua-api.md)
- [Event dispatch](../event-system.md)

## Implementation inventory

- `src/c_api/action_macros.rs`: slot association operations, recognition and profile publication.
- `src/lua_api/state{.rs,/sim_state.rs}`: existing spell/outfit maps plus macro associations.
- `src/lua_api/globals/action_bar_api{.rs,/registration.rs}`: action queries and moves.
- `src/lua_api/globals/admin{.rs,_actionbars_bags.rs}`: explicit assignment and replacement/clear controls.
- `src/lua_api/globals/inventory_verbs.rs`: legacy action info and cursor operations.
- `src/lua_api/globals/spell_macro_verbs.rs`: stored macro creation, editing and deletion.
- `src/lua_api/workarounds/temporary/macro_defaults.rs`: remaining UI fixtures; obsolete create/delete stubs removed.

## Tests asserting this spec

- `tests/action_macro_tooltip.rs`, within the existing integration target: directive boundaries, edit visibility, assignment/replacement/movement/clear, validation and profile absence.

## Known gaps (current cycle)

- [ ] Native directive parsing, validation, secret/taint access and valid-slot boundaries remain unverified.
- [ ] Existing macro UI info/count fixtures and macro execution do not model full native macro storage or execution.

## Out of scope

- Evaluating macro conditions or resolving tooltip spell/item tokens: this query only recognizes the directive.
- General action-kind redesign, tooltip rendering, macro event timing and security enforcement: separate compatibility work.
