# Tooltip payload identity

`C_TooltipInfo` payloads preserve the identity supplied by modeled action, spell and item sources. Existing builders live in `src/lua_api/globals/missing_surface/tooltip_info/`; native `TooltipUtil.GetDisplayedSpell` reads the primary payload's `id` before calling `C_Spell.GetSpellName`.

## What it must do

- [x] Preserve spell ID 45524 through action slot 5 and direct/link spell getters even when local spell metadata is absent; retain known spell 19750 identity and existing empty-action behavior.
- [x] Preserve item 6948, modeled toy 166779 and mount spell 23338 identities without changing their existing text.
- [x] Run an actual ActionButton5 `OnEnter` through Blizzard TooltipDataHandler and a Clicked-style spell postcall: `GetSpell()` returns ID 45524, the callback appends its binding line, and no callback error is recorded.
- [ ] `GetHyperlink("unit:<GUID>")` resolves known modeled player/target GUIDs to a Unit tooltip with a first-line name and matching GUID; unknown GUIDs yield nil, including repeated ATT-shaped reads.
- [ ] `GetHyperlink` returns nil for malformed/unsupported links rather than an empty Item tooltip; known item/spell hyperlinks retain their existing typed payloads.

## How it works

- [Lua API architecture](../lua-api.md)
- [Lua error reporting](lua-error-reporting.md)

## Implementation inventory

- `src/lua_api/globals/missing_surface/tooltip_info/builders.rs` — construct identified payloads and populate known items.
- `src/lua_api/globals/missing_surface/tooltip_info/spell.rs` — retain spell identity independently of metadata; identify modeled toy and mount payloads.
- `src/lua_api/globals/missing_surface/tooltip_info/probes.rs` — action/direct/link getters dispatch by source; unmodeled links yield nil.
- `src/lua_api/globals/missing_surface/tooltip_info/unit.rs` — resolve modeled unit GUIDs and populate unit tooltip lines.

## Tests asserting this spec

- `tests/tooltip_item_sources.rs` — supplied spell identity, known item/toy/mount identity, aliases, known player/target unit links, unknown GUID retry, and malformed/unsupported absence.
- `tests/tooltip_hover.rs` — native ActionButton OnEnter and TooltipDataHandler postcall boundary.

## Known gaps (current cycle)

Targeted RED at `4f62f6f93` reproduced all three new cases, including `TooltipUtil.lua:29` calling `GetSpellName(nil)` during ActionButton5 OnEnter (`/tmp/pi-tooltip-identity-red.*`). At `ec95f51f2`, the native hover/postcall and item/toy/mount cases passed; the combined run failed because the direct-source fixture passed a bare string where the hyperlink getter expected hyperlink syntax. The corrected fixture at `b5ee0a180` passes separately (`/tmp/pi-tooltip-identity-links-green.*`). Together these retained runs prove three cases; the first combined run is not an all-green result. Independent checks and full-scene verification remain with the parent integration task.

## Out of scope

This correction does not populate missing spell metadata, change strict spell API validation, change unknown item existence behavior, infer unmodeled NPC names or GUIDs, patch addon/vendor handlers, or correct unit-frame ordering. Spell 45524's name/icon coverage remains a separate data-model concern.
