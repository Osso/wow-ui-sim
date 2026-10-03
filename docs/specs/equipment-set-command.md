# Equipment-set slash command

Retail 12.0.5 prose rows `prose-2026-03-25-121` and `prose-2026-03-31-178` in [source notes](../../data/patch-api/sources/12.0.5-api-changes.txt) state: “Using the /equipset command with an unknown or invalid set name no longer causes Lua errors.” This bounded slice connects the existing macro text dispatcher to the existing equipment-manager records and equip transition.

## What it must do

- [x] Unknown, empty and whitespace-only set names complete without Lua errors or equipment/last-used-set mutation; an invalid command must not poison a following valid command.
- [x] A known name equips the saved slot contents through the existing equipment-manager path, preserving ignored slots and updating last-used-set ID.
- [x] Reuse existing swap-pending, swap-finished and sets-changed notifications; invalid names emit no swap notifications.
- [x] Renaming or deleting a stored set changes subsequent command lookup immediately.
- [ ] **INFERRED:** Resolve names by exact case-sensitive equality after existing macro parser whitespace trimming. Names containing spaces work; no quoted-name or conditional-option syntax is interpreted.

## How it works

- [Lua API architecture](../lua-api.md)
- [Event system](../event-system.md)

## Implementation inventory

- `src/c_api/equipment_set_command.rs` — lookup stored set names and invoke the shared equipment transition.
- Edits in `src/c_api/mod.rs`, `src/c_api/item_spell/mod.rs`, `src/c_api/item_spell/c_equipment_set.rs` and `src/lua_api/globals/spell_macro_verbs.rs`.

## Tests asserting this spec

- `tests/equipment_set_command.rs` — invalid-to-valid recovery, real equipment changes, ignored slots, notifications, rename/delete behavior.
- Generated integration binary filter: `equipment_set_command::`.
- RED and GREEN executed by the main session; see the proof section.

## Development proof and independent bounded acceptance — 2026-10-03

Inputs and producer landed together in `92e4ea045`. RED with producers withheld: 0 PASS / 2 FAIL. GREEN: 2/2 inside a 62/62 narrow run; `cargo fmt --check` exit0; startup `lua-errors` `[]`. A broad run gave 1189 PASS / 3 FAIL: a parked bundle slice (since removed) and two tooltip tests that fail identically with this commit's tooltip change reverted.

Main accepts an independent GPT-6.1-sol source review (it did not rerun tests): **ACCEPT WITH QUALIFICATIONS**, [report](../../data/patch-api/evidence/12.0.5-session-2026-10-03/b97-verify-quest-equipset.md) SHA256 `bc83a97ae035ad51aef209367c4fb540040549f7710865e75e2d521d77470a01`. Name-resolution policy is inferred and only partly tested (casing, names with spaces). Checked requirements are bounded simulator proof on the tested fixtures, not native parity.

[Page accounting](../../data/patch-api/sources/12.0.5-page-coverage.json): rows prose-2026-03-25-121, prose-2026-03-31-178 under new capability `equipment-set-command`; **110 capabilities/362 IDs; 71 pending /243 bounded /15 partial /33 metadata**.

## Known gaps (current cycle)

- [ ] Valid-name trimming and quoted or conditional syntax are untested; direct `UseEquipmentSet` preservation has source-comparison proof only.

## Out of scope

- Native security/secret argument parity, combat restrictions, macro conditional/quoted syntax, gear availability, enchant preservation and existing equipment-manager transition policies. This slice reuses, not redesigns, those policies; it does not claim whole macro-engine parity.
