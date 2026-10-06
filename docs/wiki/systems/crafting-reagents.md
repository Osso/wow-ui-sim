# Crafting reagents

B08 implements nested item/currency reagent state in `src/c_api/`, owned by `CraftingState.reagents`. [Contract and proof boundary](../../specs/crafting-reagent-contracts.md).

## Model

`CraftingReagent` has Item/Currency variants. Recipe slots initialize from the existing catalog; host fixtures can supply alternatives, variable quantities and metadata. Allocation validation precedes inventory changes. Currency quantities share `SimState.currency_info`; items share bags. Host resourcefulness outcomes subtract saved resources from gross consumption after gross-resource preflight.

Typed order requests copy every documented input field. Explicit ability-to-recipe mapping prevents conflating IDs. Local orders serialize nested reagentInfo records; no Lua request/output table is retained. Intermediate output tables stay rooted across allocation and event dispatch.

## Limits

See [out-of-scope contract](../../specs/crafting-reagent-contracts.md#out-of-scope). Structural proof does not establish native server lifecycle or full CraftingOrderInfo parity. Order container fields outside the reagent slice remain bounded local defaults. Unsupported fields must not receive broader ledger credit.

## Sources

- [Extract scout](../../../data/patch-api/evidence/12.0.0-session-2026-10-05/p1200-extract-scout.md)
- `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/TradeSkillUITypesDocumentation.lua`, `CraftingOrderUISharedDocumentation.lua`, `TradeSkillUIDocumentation.lua`, `CraftingOrderUIDocumentation.lua`, `ProfessionConstantsDocumentation.lua`.
- Cached `Blizzard_Professions/Blizzard_ProfessionsCraftingOutputLog.lua` and `Blizzard_ProfessionsCustomerOrders/Blizzard_ProfessionsCustomerOrdersForm.lua`.

## See Also

- [12.0.0 API audit](../investigations/patch-12-0-0-api-audit.md)
