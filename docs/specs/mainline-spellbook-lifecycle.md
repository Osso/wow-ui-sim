# Mainline spellbook lifecycle

The simulator must exercise the Blizzard `PlayerSpells` keybinding lifecycle under every client profile that ships the Mainline panel contract: Retail, PTR, and Forever. Profile-specific implementation details are documented in [[mainline-spellbook-lifecycle]].

## What it must do

### Runtime interaction

- [x] Pressing the default spellbook key opens the profile's spellbook frame.
- [x] Ten runtime update ticks after opening produce no collected Lua errors.
- [x] Pressing the key again closes the spellbook frame.
- [x] Ten runtime update ticks after closing produce no collected Lua errors.
- [x] The same behavioral regression runs under Retail, PTR, and Forever rather than a Retail-only fixture.

### Forever producers

- [x] Forever startup loads `[Bootstrap]` publishers from LoadOnDemand Blizzard addons, including `InClickBindingMode`.
- [x] `C_SpellBook.GetClassSkillLineInfo()` returns the modeled class skill line.
- [x] `GetPetIcon()` reads the modeled summoned-pet icon and returns nil when no pet is summoned.
- [x] Forever publishes the documented clear-transmog spell ID and `ActionBarSet` enum.
- [x] `C_Spell.GetSpellInfo(1247917)` resolves the source-backed clear-transmog spell metadata consumed by the SpellBook.

## How it works

- [[mainline-spellbook-lifecycle]]
- [[client-profiles]]

## Implementation inventory

- `src/loader/startup_addons.rs` — admits Forever Blizzard bootstrap publishers into startup order.
- `src/c_api/c_spell_book.rs` — publishes modeled class skill-line information.
- `src/lua_api/globals/real/pet_bar.rs` — publishes the state-backed pet icon query.
- `src/lua_api/sim_substates/mod.rs` — stores summoned-pet icon state.
- `src/c_api/forever_finite_constants.rs` — publishes the Forever clear-transmog spell ID.
- `src/lua_api/globals/enum_data/forever_shared.rs` — publishes shared Mainline/Forever action-bar-set values.
- `src/spell_lookup.rs` — supplies exact clear-transmog spell metadata.

## Tests asserting this spec

- `tests/blizzard_player_spells_loads.rs`

## Known gaps (current cycle)

- [ ] Mists uses the Cata PlayerSpells implementation and currently fails its distinct panel lifecycle before this Mainline-family assertion applies.
- [ ] Wrath, Era, and Anniversary use legacy `SpellBookFrame`; they need a separate legacy lifecycle regression.

## Out of scope

- Mists/Cata and legacy SpellBook behavior are separate compatibility contracts, not silent skips of the Mainline test.
- Spellbook asset extraction and draw-stall performance are unrelated to lifecycle correctness.
- Native client execution is not required for source-published enum, constant, and spell metadata.
