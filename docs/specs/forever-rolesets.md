# Forever rolesets

Frame rolesets control which frames match `C_Roleset` filters. Forever's CombatLog assigns the `chat` tag to `CombatLogQuickButtonFrame`.

## What it must do

- [x] Expose `AddRoleset`, `GetRolesetNames`, `RemoveRoleset`, and `SetRolesets` on the frame API.
- [x] Keep roleset membership per frame. `SetRolesets` replaces the list; `AddRoleset` and `RemoveRoleset` change one name.
- [x] Return `GetRolesetNames` as a one-based array of names, not a name-to-boolean table.
- [x] Split `SetRolesets` input at commas, trim ASCII spaces, and skip empty parts.
- [x] An empty-string `SetRolesets` call clears stored membership without storing `roleless`.
- [x] Report `Usage: self:SetRolesets([rolesetsString])` for a table argument. **INFERRED:** the non-string, non-number trigger follows the native optional-string convention; only the usage text is verified.

## How it works

- [Frame and Lua API](../lua-api.md)
- [Widget model](../widget-system.md)

## Implementation inventory

- `src/lua_api/frame/methods/text_attribute_event/rolesets.rs` — frame roleset methods and result publication.
- `src/c_api/c_roleset.rs` — stored membership updates and active filter evaluation.

## Tests asserting this spec

- `tests/wowforever_rolesets.rs` — Forever CombatLog roleset membership, array results, parsing, filtering, clearing, and the inferred usage-error trigger.
- `tests/patch_12_1_0_rolesets.rs` — Retail 12.1 roleset filtering and lifecycle.

## Known gaps (current cycle)

- [ ] Decomp evidence for the `SetRolesets` usage-error trigger condition.

## Out of scope

- **INFERRED, unasserted:** `GetRolesetNames` returns `roleless` for an untagged frame. The current result is preserved, but this lane does not test it.
- **UNKNOWN, unasserted:** the order of multiple names and the return count on every path.
- Native callback effects from the frame update helper.
