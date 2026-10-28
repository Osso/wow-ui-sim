# Wrath spellbook tooltip

Wrath's `GameTooltip:SetSpell` reads supplied spellbook slots. The original consumer is pinned `FrameXML/SpellBookFrame.lua:332–339`.

## What it must do

- [ ] Accept a numeric slot and string book type, using the same slot resolution as `GetSpellName` and `GetSpellTexture`.
- [x] A resolved slot replaces the tooltip with its supplied name and shows it. The modeled path returns nil.
- [x] An unresolved slot returns nil without changing the tooltip.

The historical 3.3.5a decompilation is a **comparison**, not proof of Wrath Classic parity: `Tooltip.cpp:3148–3301` describes slot validation, unresolved preservation, and conditional cooldown/effect-chance returns. Cost and rank right text are UNKNOWN. Range, cast time, requirements, reagents, cooldown, effect chance and description are not modeled here; their contents and availability are UNKNOWN for supplied slot records.

## How it works

- [Tooltip architecture](../lua-api.md)

## Implementation inventory

- `src/lua_api/frame/methods/widgets/tooltip.rs` registers the Wrath method.
- `src/lua_api/frame/methods/widgets/tooltip/content.rs` populates the tooltip.
- `src/lua_api/globals/real/wrath_spell_slots.rs` resolves supplied slots.

## Tests asserting this spec

- `tests/common/wrath_spell_panels_f10.rs` exercises the original hover consumer and unresolved preservation.

## Known gaps (current cycle)

- [ ] Cooldown and effect-chance return paths are not modeled.

## Out of scope

- Additional native tooltip lines without supplied scenario data or established behavior.
