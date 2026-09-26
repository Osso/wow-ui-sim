# Macro action textures

`041c8235a` routes both formerly spell-only queries through `src/c_api/c_action_bar.rs`, which reads the explicit icon stored for an assigned macro; see [Lua API](../lua-api.md).

## What it must do

- [x] Both queries return the assigned macro's nonempty stored icon string; subsequent `EditMacro` icon changes are visible without reassignment.
- [x] Moving a macro action moves its query result; clearing/deleting the assignment returns nil. An empty stored icon returns nil without a fabricated replacement.
- [x] Preserve spell action icon resolution and agreement between both queries.

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

None for this bounded correction. Independent verification passes four macro-query/profile cases, 25 inventory controls, format/check and changed-code readability. `/tmp/cross-version-macro-texture-verification-ledger.md` records exact revisions; the separate proof ledger retains RED evidence.

## Out of scope

Numeric icon-ID coercion, automatic `#showtooltip`/conditional icon evaluation, outfit icons, `GetMacroInfo` fixtures, notification timing, rendered pixels and complete native macro conformance. No vendor changes or alternate icon fallback.
