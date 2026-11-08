# Crafting reagents

B08 implements nested item/currency reagent state in `src/c_api/`, owned by `CraftingState.reagents`. [Contract and proof boundary](../../specs/crafting-reagent-contracts.md).

## Model

`CraftingReagent` has Item/Currency variants. Recipe slots initialize from the existing catalog; host fixtures can supply alternatives, variable quantities and metadata. Allocation validation precedes inventory changes. Currency quantities share `SimState.currency_info`; items share bags. Host resourcefulness outcomes subtract saved resources from gross consumption after gross-resource preflight.

Typed order requests copy every documented input field. Explicit ability-to-recipe mapping prevents conflating IDs. A required queued host placement result supplies lifecycle, fee, quality, identity and reward fields; no container sentinel is fabricated. Valid placement consumes one host response; rejected requests preserve the queue. Local orders serialize nested reagentInfo records; no Lua request/output table is retained. Intermediate output tables stay rooted across allocation and event dispatch.

## Crafted item quality source checkpoint — 2026-10-13

Source `05665ddad` registers `C_TradeSkillUI.GetItemCraftedQualityInfo` for Retail/PTR. The existing `CraftingQualityInfo` record is reused. `SimState.crafted_item_quality` is an initially empty `HashMap<u32, CraftingQualityInfo>`, distinct from recipe/quality-pair inputs.

Original VM-secret authentication precedes Num/Str validation and the existing item parser. Missing supplied data returns one nil as simulator policy. A present record is cloned before allocation; shared projection pushes the fresh result table onto the VM stack before creating field strings. Recipe queries reuse projection without changing their paired lookup. [Crafted-item contract](../../specs/crafted-item-quality-info.md) owns field requirements and links execution evidence; [recipe-input contract](../../specs/api-contract-probe.md#manual-recipe-quality-input-acceptance) stays separate.

Executed Retail/PTR initial and positive-query/original-consumer RED preceded the getter. At this source checkpoint, GREEN was pending; current retained GREEN evidence records 24 outcomes and two separate startup passes. Consumer visibility uses a real host-created Frame, not the original professions panel. Native catalog/name/render/panel parity remains unproved.

## Limits

See [out-of-scope contract](../../specs/crafting-reagent-contracts.md#out-of-scope). Structural/host-snapshot proof does not establish native server lifecycle or full CraftingOrderInfo policy parity. Native placement, fee computation, escrow and fulfillment remain host responsibilities; unavailable host responses are errors, not placeholder orders.

## Sources

- [Crafted-item contract](../../specs/crafted-item-quality-info.md)
- [Quality getter and shared projection](../../../src/c_api/c_trade_skill_quality.rs)

- [Extract scout](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/12.0.0-session-2026-10-05/p1200-extract-scout.md)
- `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/TradeSkillUITypesDocumentation.lua`, `CraftingOrderUISharedDocumentation.lua`, `TradeSkillUIDocumentation.lua`, `CraftingOrderUIDocumentation.lua`, `ProfessionConstantsDocumentation.lua`.
- Cached `Blizzard_Professions/Blizzard_ProfessionsCraftingOutputLog.lua` and `Blizzard_ProfessionsCustomerOrders/Blizzard_ProfessionsCustomerOrdersForm.lua`.

## See Also

- [12.0.0 API audit](../investigations/patch-12-0-0-api-audit.md)
