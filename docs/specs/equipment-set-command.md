# Equipment-set slash command

Retail 12.0.5 prose rows `prose-2026-03-25-121` and `prose-2026-03-31-178` in [source notes](../../data/patch-api/sources/12.0.5-api-changes.txt) state: “Using the /equipset command with an unknown or invalid set name no longer causes Lua errors.” This bounded slice connects the existing macro text dispatcher to the existing equipment-manager records and equip transition.

## What it must do

- [ ] Unknown, empty and whitespace-only set names complete without Lua errors or equipment/last-used-set mutation; an invalid command must not poison a following valid command.
- [ ] A known name equips the saved slot contents through the existing equipment-manager path, preserving ignored slots and updating last-used-set ID.
- [ ] Reuse existing swap-pending, swap-finished and sets-changed notifications; invalid names emit no swap notifications.
- [ ] Renaming or deleting a stored set changes subsequent command lookup immediately.
- [ ] **INFERRED:** Resolve names by exact case-sensitive equality after existing macro parser whitespace trimming. Names containing spaces work; no quoted-name or conditional-option syntax is interpreted.

## How it works

- [Lua API architecture](../lua-api.md)
- [Event system](../event-system.md)

## Implementation inventory

- `src/c_api/equipment_set_command.rs` — lookup stored set names and invoke the shared equipment transition.
- Main-session edits required in `src/c_api/mod.rs`, `src/c_api/item_spell/mod.rs`, `src/c_api/item_spell/c_equipment_set.rs` and `src/lua_api/globals/spell_macro_verbs.rs`; exact replacements are in the scratchpad handoff.

## Tests asserting this spec

- `tests/equipment_set_command.rs` — invalid-to-valid recovery, real equipment changes, ignored slots, notifications, rename/delete behavior.
- Generated integration binary filter: `equipment_set_command::`.
- Tests authored before implementation; RED/GREEN execution prohibited by this authoring task. No passing-test claim.

## Known gaps (current cycle)

- [ ] Main session must apply handoff edits, format existing-file changes, compile and run the focused fixtures.

## Out of scope

- Native security/secret argument parity, combat restrictions, macro conditional/quoted syntax, gear availability, enchant preservation and existing equipment-manager transition policies. This slice reuses, not redesigns, those policies; it does not claim whole macro-engine parity.
