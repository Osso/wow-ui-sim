# Tooltip data processor

The tooltip data processor registers callbacks for tooltip data that Blizzard’s original tooltip handler produces.

## What it must do

- [ ] **VERIFIED (Forever and Mists pinned UI cache):** Load `TooltipDataHandler.lua` from `Blizzard_SharedXMLGame` before code uses its processor. Forever’s TOC lists the file at line 9; Mists lists it at line 9 with `ExcludeLoadGameType vanilla`.
- [ ] **VERIFIED (Forever and Mists pinned UI cache):** `TooltipDataHandler.lua:177-200` defines `TooltipDataProcessor.AllTypes` as `"ALL"` and implements `AddTooltipPostCall`; `:525` maps `GameTooltip:SetItemByID` to `C_TooltipInfo.GetItemByID`.
- [ ] **VERIFIED (simulator integration test):** After the original handler loads, a post-call registered for item tooltips runs once when `GameTooltip:SetItemByID(6948)` populates item data.
- [ ] **INFERRED (simulator mapping):** Wrath may need a profile-only compatibility shim if a selected Wrath consumer requires this surface; do not add one without a failing Wrath proof.

## How it works

- [Blizzard UI loading pipeline](../addon-loading-pipeline.md)
- [Lua API surface](../lua-api.md)

## Implementation inventory

- `tests/tooltip_processor_retirement.rs` — checks the original handler and item-setter callback path on Forever and Mists.
- `src/lua_api/workarounds/temporary/tooltip_data_processor_defaults.rs` — legacy partial-load defaults to retire.

## Tests asserting this spec

- `tests/tooltip_processor_retirement.rs`

## Known gaps (current cycle)

- [ ] Confirm whether Wrath has a selected consumer that requires a compatibility shim.

## Out of scope

- Modeling TooltipDataProcessor behavior for profiles whose selected Blizzard UI does not load `TooltipDataHandler.lua`.
