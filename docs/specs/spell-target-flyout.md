# Target spell and loose flyout membership

PTR and WoWForever expose target-spell and spellbook flyout queries used by their pinned UI. The simulator reads scenario inputs rather than inferring native state.

## What it must do

- [ ] Publish `C_Spell.GetTargetSpellID` and `C_SpellBook.IsSpellBookItemLooseFlyoutMember` only for PTR and WoWForever. Their pinned declarations are `SpellDocumentation.lua:648` (PTR), `:631` (WoWForever), and `SpellBookDocumentation.lua:634` (both).
- [x] Return the supplied target spell ID. An explicitly unset input returns nil, which deviates from the declarations' `Nilable = false`; the Forever stock idle scenario supplies `0` as a researched best guess for the unknown no-target sentinel.
- [x] Return supplied flyout membership for the modeled player spellbook entry. Missing membership yields false as a scenario default, not a native claim.
- [ ] Unmodeled slots or banks yield false as a scenario default, not a native claim.
- [x] Authenticate secret slot and bank arguments with the declared `AllowedWhenUntainted` policy.

## How it works

- [Lua API](../lua-api.md)

## Implementation inventory

- `src/c_api/forever_startup.rs` — target-spell producer and scenario input.
- `src/c_api/c_spell_book.rs` — flyout membership query.
- `src/lua_api/state/sim_state.rs` — supplied spellbook membership.

## Tests asserting this spec

- `tests/test_keybindings_spellbook_talents.rs` — PTR original consumers and secret arguments.
- `tests/forever_startup_gaps.rs` — Forever target-spell consumer.

## Known gaps (current cycle)

- [ ] The no-target sentinel is not native evidence. The 2026-10-25 aso source-only audit classified the idle target-spell return as native-unknown; `0` satisfies the non-nilable declaration and the original consumer's `== FEED_PET_SPELL_ID` comparison, but no running-client observation confirms it.
- [ ] Runtime absence of these queries on undeclared profiles is not independently tested; publication gates are source-inspected and all profiles compile.

## Out of scope

Native target lifecycle and flyout acquisition policy are not established.
