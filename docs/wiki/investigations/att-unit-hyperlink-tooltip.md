# ATT unit hyperlink tooltip

`AllTheThings/src/Classes/NPC.lua:58` reads `C_TooltipInfo.GetHyperlink("unit:Creature-…").lines[1].leftText`. Previously `GetHyperlink` handled item/spell IDs and returned an empty Item tooltip for every other link. The missing first line caused the reported failure; the consumer guard was not the source of the invalid payload.

## Resolution

`GetHyperlink` now resolves `unit:<GUID>` against the modeled player, current target/focus, and active party GUIDs. A known named unit reuses its Unit tooltip lines and GUID. Unknown or unsupported links return nil rather than a fabricated Item payload, consistent with the cached `TooltipInfoDocumentation.lua` declaration `MayReturnNothing = true` for `GetHyperlink`. Item/spell dispatch remains unchanged. No NPC names or data are inferred from GUID components.

## Proof boundary

An existing binary before the change returned `ATT_TOOLTIP_RED 0 0` for an unknown creature GUID (Item type, zero lines). Grouped Lua-facing regression cases were committed in `tests/tooltip_item_sources.rs` at `dc6672dc2` for known target/player, repeated unknown lookup, malformed/unsupported links, and item/spell controls. Cargo ownership is elsewhere; these new tests, rebuilt diagnostic and full GUI hover are **not yet verified**.

## Sources

- [Tooltip identity spec](../../specs/tooltip-identity.md) — public contract and grouped test inventory.
- [Tooltip unit model](../../../src/lua_api/globals/missing_surface/tooltip_info/unit.rs) — modeled GUID and line construction.
- [Tooltip hyperlink dispatch](../../../src/lua_api/globals/missing_surface/tooltip_info/probes.rs) — source selection and nil behavior.

## See Also

- [[lua-api]] — Lua surface and state ownership.
- [[tooltip-money-line]] — separate tooltip investigation.
