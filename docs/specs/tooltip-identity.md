# Tooltip payload identity

`C_TooltipInfo` payloads preserve the identity supplied by modeled action, spell and item sources. Existing builders live in `src/lua_api/globals/missing_surface/tooltip_info/`; native `TooltipUtil.GetDisplayedSpell` reads the primary payload's `id` before calling `C_Spell.GetSpellName`.

## What it must do

- [ ] Preserve spell ID 45524 through action slot 5 and direct/link spell getters even when local spell metadata is absent; retain known spell 19750 identity and existing empty-action behavior.
- [ ] Preserve item 6948, modeled toy 166779 and mount spell 23338 identities without changing their existing text.
- [ ] Run an actual ActionButton5 `OnEnter` through Blizzard TooltipDataHandler and a Clicked-style spell postcall: `GetSpell()` returns ID 45524, the callback appends its binding line, and no callback error is recorded.

## How it works

- [Lua API architecture](../lua-api.md)
- [Lua error reporting](lua-error-reporting.md)

## Implementation inventory

- `src/lua_api/globals/missing_surface/tooltip_info/builders.rs` — construct identified payloads and populate known items.
- `src/lua_api/globals/missing_surface/tooltip_info/spell.rs` — retain spell identity independently of metadata; identify modeled toy and mount payloads.
- `src/lua_api/globals/missing_surface/tooltip_info/probes.rs` — existing action/direct/link getters delegate to those builders.

## Tests asserting this spec

- `tests/tooltip_item_sources.rs` — supplied spell identity, known item/toy/mount identity and alias delegation.
- `tests/tooltip_hover.rs` — native ActionButton OnEnter and TooltipDataHandler postcall boundary.

## Known gaps (current cycle)

Targeted RED reproduced all three new cases, including `TooltipUtil.lua:29` calling `GetSpellName(nil)` during ActionButton5 OnEnter. GREEN is pending.

## Out of scope

This correction does not populate missing spell metadata, change strict spell API validation, change unknown item existence behavior, infer ambiguous unit/aura identities, patch addon/vendor handlers, or correct unit-frame ordering. Spell 45524's name/icon coverage remains a separate data-model concern.
