# Call-pet spell results in the Retail flyout

`GetCallPetSpellInfo(spellID)` supplies the legacy tuple that the selected Retail `Blizzard_ActionBar/Shared/SpellFlyout.lua` consumer reads. The native result contract below is bounded to Retail 12.1.5.70688 evidence in `pr46-49-decomp-answers.md`, sections R88i.28 and R88j.3; the consumer is selected by the Retail `Blizzard_ActionBar` TOC.

## What it must do

- [x] For a supplied spell membership/index in the range 0–4, return exactly two values: the index and the current player's pet-info name, or nil when that name is absent.
- [x] Preserve an empty pet-info name as an empty string.
- [x] Return zero values when the spell is not supplied as a member or its index is outside 0–4.
- [x] In the original action-bar flyout, keep a named call-pet spell visible, hide a call-pet spell with a nil or empty name, and keep an unmapped spell visible.
- [x] Keep spell membership/index data and current-player pet names as explicit, empty-default host inputs. Do not seed native IDs or fixed pet names.

## How it works

- [Spellbook R88m.4 follow-ups](../wiki/investigations/spellbook-r88m4-followups.md) — prior boundaries and the updated call-pet input model.
- [Spell flyout integration test](../../tests/blizzard_ui/blizzard_actionbar/behavior_spell_flyout.rs) — original consumer coverage.

## Implementation inventory

- `src/lua_api/globals/real/spell_flyout_legacy.rs` — Retail global registration and result projection.
- `src/lua_api/state/sim_state.rs` — empty-default Retail host inputs.
- `src/lua_api/state.rs` — host-input defaults.
- `src/lua_api/workarounds/temporary/retail_spellbook_legacy.rs` — shared legacy argument conversion; call-pet's old zero-result fallback is retired.
- `src/lua_api/workarounds/temporary/spell_book_static_defaults.rs` — preserves the existing non-Retail fallback without registering a Retail stub.

## Tests asserting this spec

- `tests/blizzard_ui/blizzard_actionbar/behavior_spell_flyout.rs::spell_flyout_filters_call_pet_slots_using_supplied_names` — the original Retail `SpellFlyoutMixin:Toggle` consumer, named/empty/missing-name inputs, and absent membership.
- `tests/blizzard_ui/blizzard_actionbar/behavior_spell_flyout.rs::spell_flyout_consumer_preserves_slots_without_call_pet_catalog` — visible slots with empty-default call-pet inputs.

## Known gaps (current cycle)

- [ ] Same-build Retail spell-ID membership/index rows and the runtime writer for current-player pet-info names remain UNKNOWN; the simulator must leave both inputs empty until supplied by the host.

## Out of scope

- Other client profiles; the decomp answer covers Retail only.
- A static spell-ID catalog, fixed pet names, and event-driven writers; the evidence does not establish them.
