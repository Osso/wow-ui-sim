# Macro action textures

`041c8235a` routes both formerly spell-only queries through `src/c_api/c_action_bar.rs`, which reads the explicit icon stored for an assigned macro; see [Lua API](../lua-api.md).

## What it must do

- [ ] Both queries return the assigned macro's nonempty stored icon string; subsequent `EditMacro` icon changes are visible without reassignment.
- [ ] Moving a macro action moves its query result; clearing/deleting the assignment returns nil. An empty stored icon returns nil without a fabricated replacement.
- [ ] Preserve spell action icon resolution and agreement between both queries.

Cached retail `Blizzard_ActionBar/Shared/ActionButton.lua` queries `C_ActionBar.GetActionTexture(action)` and hides the button icon on nil. This contract connects existing explicit macro state to that consumer; it does not claim full native macro-icon evaluation.

## How it works

- [Lua API](../lua-api.md)
- [Macro action associations](macro-action-showtooltip.md)

## Implementation inventory

- `src/c_api/c_action_bar.rs`: shared texture resolution from macro association/icon or spell metadata.
- `src/lua_api/globals/action_bar_api.rs`: namespaced query.
- `src/lua_api/globals/inventory_verbs.rs`: legacy query.

## Tests asserting this spec

Three unguarded cases in `tests/action_macro_tooltip.rs`, grouped `integration` target, exercise edit visibility, move/clear/delete/empty-icon transitions, and spell resolution.

## Known gaps (current cycle)

- [ ] Post-fix verification pending. The pre-fix macro cases are RED while the spell control passes; `/tmp/cross-version-macro-texture-proof.md` records revisions and commands, and `/tmp/cross-version-macro-texture-verification-ledger.md` starts pending.

## Out of scope

Numeric icon-ID coercion, automatic `#showtooltip`/conditional icon evaluation, outfit icons, `GetMacroInfo` fixtures, notification timing, rendered pixels and complete native macro conformance. No vendor changes or alternate icon fallback.
