# Tooltip data processor

The tooltip data processor registers callbacks for tooltip data that Blizzard’s original tooltip handler produces.

## What it must do

- [x] **VERIFIED (Forever and Mists pinned UI cache and preservation tests):** `Blizzard_SharedXMLGame/Blizzard_SharedXMLGame.toc:9` loads `Tooltip/TooltipDataHandler.lua` on both profiles; Mists excludes only `vanilla` at that line.
- [x] **VERIFIED (Forever and Mists pinned UI cache and preservation tests):** `Tooltip/TooltipDataHandler.lua:177-200` defines `TooltipDataProcessor.AllTypes` as `"ALL"` and implements `AddTooltipPostCall`; `:525` maps `GameTooltip:SetItemByID` to `C_TooltipInfo.GetItemByID`.
- [x] **VERIFIED (Forever preservation test before and after fallback deletion):** A registered item post-call runs once when the original `GameTooltip:SetItemByID(6948)` consumer populates item data.
- [ ] **INFERRED (simulator mapping):** A partial environment without `Blizzard_SharedXMLGame` does not receive these globals from this workaround; their availability follows the loaded Blizzard files.
- [ ] **UNKNOWN (Wrath):** The compile-only gate does not establish runtime consumer behavior; no Wrath shim is retained without a failing consumer proof.

## How it works

- [Blizzard UI loading pipeline](../addon-loading-pipeline.md)
- [Lua API surface](../lua-api.md)

## Implementation inventory

- `tests/tooltip_processor_retirement.rs` — checks the original item-setter callback path on Forever and original handler loading on Mists.
- `src/lua_api/workarounds/mod.rs` — no longer registers the temporary processor defaults.
- `src/lua_api/workarounds/temporary/mod.rs` — no longer declares the retired module.

## Tests asserting this spec

- `tests/tooltip_processor_retirement.rs`

## Known gaps (current cycle)

- [ ] **UNKNOWN (Wrath):** This lane compiles Wrath but does not run its UI; runtime consumer behavior remains unverified.

## Out of scope

- **VERIFIED (Mists pinned UI cache):** Callback-dispatch testing does not apply to Mists: `Blizzard_GameTooltip/Blizzard_GameTooltip_Classic.toc:4` selects `AllowLoadGameType: classic`, and `Blizzard_SharedXMLGame/Blizzard_SharedXMLGame.toc:10` marks `TooltipDataRules.lua` mainline-only.
- Modeling TooltipDataProcessor behavior for profiles whose selected Blizzard UI does not load `TooltipDataHandler.lua`.
