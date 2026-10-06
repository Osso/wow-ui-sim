# Crafting reagent contracts

12.0.0 B08 nested reagent contracts from the [extract scout](../../data/patch-api/evidence/12.0.0-session-2026-10-05/p1200-extract-scout.md). Cached Retail TradeSkillUITypes/CraftingOrderUIShared declarations and Professions consumers define serialized inputs and outputs. [Model](../wiki/systems/crafting-reagents.md).

## What it must do

- [x] Return item/currency identities, variable quantities, optional slot metadata and detached schematic/modification snapshots.
- [x] Consume nested CraftingReagentInfo through CraftRecipe; preflight item/currency quantities before mutation and publish nested resource returns.
- [x] Craft results carry the committed bag item's existing C_Item GUID policy and catalog hyperlink, not an empty/fabricated identity. Missing output catalog data fails before inventory mutation.
- [x] Parse every NewCraftingOrderInfo field and RegularReagentInfo/CraftingReagentInfo entries; post local orders without retaining Lua tables and return CraftingOrderReagentInfo.
- [x] Require host placement metadata for order lifecycle, fees, quality, identities and rewards; consume it only after valid placement. No fabricated container fields.
- [x] Reject invalid identities, quantities, unknown slots/abilities and old-only flattened inputs without partial mutation.
- [x] Preserve VM table-access policy and AllowedWhenUntainted input authentication, including nested identities; reject secret inputs from tainted callers without changing state/taint.

## How it works

- [Crafting reagent state and boundaries](../wiki/systems/crafting-reagents.md).

## Implementation inventory

- `src/c_api/crafting_reagents.rs`: typed identities, slots, requests, orders, validation.
- `src/c_api/crafting_input.rs`: secret-aware typed input decoding.
- `src/c_api/crafting_tables.rs`: rooted nested serialization.
- `src/c_api/crafting_plan.rs`: quantities, refunds and inventory preflight.
- `src/c_api/crafting_execution.rs`: existing inventory/cast path and result event.
- `src/c_api/c_crafting_orders.rs`: local placement and public snapshots.
- `src/lua_api/state_types/crafting.rs`: state ownership.
- `src/lua_api/globals/missing_surface/professions/`: existing schematic/registration boundary calls the C API model.

## Tests asserting this spec

`tests/patch_12_0_0_crafting_shapes.rs` checks cached Type/Nilable/InnerType declarations recursively, concrete two-variant values, input validation, snapshots, item/currency consumption and resource-return event payloads.

## Known gaps (current cycle)

No unresolved B08 local model test failure. [Per-source outcomes and proof](../../data/patch-api/evidence/12.0.0-session-2026-10-05/p1200-b8-prose-report.md) distinguish bounded current-cache structure proof from historical/native parity.

## Out of scope

Native server placement, escrow, commission, order expiry/fulfillment and randomized resourcefulness are not inferred from structural declarations. Exclusive identity variants, strict integral i32 input ranges, complete required allocations with permitted over-allocation, local order IDs and immediate resource-result event timing are explicitly INFERRED policy, not native parity. Existing CraftRecipe boolean return is retained; changing its legacy compatibility arity is outside this extract slice.
