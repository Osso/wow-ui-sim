# Forever rolesets

Forever 1.60.1 CombatLog assigns `chat` membership to its quick-button frame.

## Contract

- Expose existing `AddRoleset`, `RemoveRoleset`, `SetRolesets`, and `GetRolesetNames` methods to Forever and retail 12.1.
- Keep membership per frame. Set replaces membership; an empty set clears it.
- Preserve existing name-to-boolean table representation and method lookup.
- Do not expose unrelated security or forbidden-aspect methods through this change.

## Proof

`tests/wowforever_rolesets.rs` exercises the CombatLog call sequence, replacement, removal, clearing, frame isolation, and metatable publication.

Native roleset/security semantics are not claimed.
