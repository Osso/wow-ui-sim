# Supplied item-upgrade eligibility

`src/c_api/item_upgrade_eligibility.rs` supplies eligibility for real bag and equipment locations. It does not calculate native upgrade policy. See [the model](../wiki/systems/item-upgrade-eligibility.md).

## What it must do

- [x] Publish `C_ItemUpgrade.CanUpgradeItem` on Retail, PTR, Forever, Mists, Era and Anniversary only; leave the excluded Wrath path unchanged.
- [x] Resolve an actual ItemLocation through existing inventory/bag state and return exactly one supplied boolean. Supply independent eligibility at two concrete locations; no catalog policy is inferred.
- [x] `A_Admin.SetItemUpgradeEligibility(location, boolean)` supplies an observation for the existing item. `ClearItemUpgradeEligibility(location)` removes it; omitted/nil location clears all observations in that environment.
- [x] Move observations through bag/cursor/equipment swaps and backpack placement. Removal, deletion and admin replacement must not leak observations to a replacement instance, including one with the same item ID. `A_Admin.SetReagentCount` must discard observations when removing or creating bag records; remove/recreate at the same location must fall back to the unsupplied default.
- [x] Return `false` for unsupplied and empty bag/equipment locations (decision 2026-11-01; see Best-guess basis). Raise `C_ItemUpgrade.CanUpgradeItem: itemLocation must be a bag or equipment ItemLocation` for nil, non-table and unresolvable arguments.

Pinned declarations: `Blizzard_APIDocumentationGenerated/ItemUpgradeDocumentation.lua:11–24` on Retail/PTR/Forever and `:11–23` on Mists/Era/Anniversary. Required `ItemLocation` argument; one non-nil boolean result. Generic `Blizzard_FrameXMLUtil/ItemUtil.lua` query lines rechecked on 2026-10-26: Retail147, PTR/Forever217, Mists/Era/Anniversary142. Forever's Camelot utility is not this consumer.

## How it works

- [Environment-local observations and transfer boundary](../wiki/systems/item-upgrade-eligibility.md).

## Implementation inventory

- `src/c_api/item_upgrade_eligibility.rs`: supplied observation store, native query, admin input, transfer helpers.
- `src/c_api/container_inventory.rs`: existing bag/equipment/cursor transfer lifecycle.
- `src/lua_api/globals/admin_actionbars_bags.rs`, `admin_equipment.rs`, `inventory_verbs.rs`: existing inventory mutation boundaries.
- `src/lua_api/globals/admin_crafting.rs`: SetReagentCount/seed-helper record removal and creation invalidate supplied observations.
- `src/lua_api/state/sim_state.rs`: environment-local state.

## Tests asserting this spec

- `tests/api_shape_item_upgrade.rs`: original generic ItemUtil match/mismatch through two distinct items/locations, changes, moves, removal/replacement, clear and independent environments; documented boolean arity and explicit unsupported-input diagnostics.
- `tests/item_upgrade_admin_reagent_count.rs`: real SetReagentCount remove/recreate of the same item ID at the same location must require a new observation.

The normally selected FrameXMLUtil/ObjectAPI closure is loaded without source overrides. A plain shown `ItemUpgradeFrame` supplies the context input. It is not an original upgrade window. No Blizzard function is replaced. The old boolean-only contract is preservation, not eligibility acceptance.

## Known gaps (current cycle)

- [x] Execute byte-identical original-consumer RED/GREEN on all six declaring profiles, bound stock pairs and seven zero-warning canonical checks. See [the final proof](../wiki/systems/item-upgrade-eligibility.md#final-proof--2026-10-26).
- [ ] Bag-helper preservation on Mists/Era/Anniversary is BLOCKED before the added seed/query: the existing fixture requires Blizzard_POIButton, which has no compatible TOC. Both final branches and earlier RED receipts stop at this same setup boundary. No assertions or applicability gates were weakened.

## Best-guess basis

best-guess, native UNKNOWN. Unmodified Blizzard consumers (TokenUI via `ToggleBackpack`, auction-house tab switching through `ItemUtil.lua`) query every bag item, so an error for unsupplied locations broke them. Most items cannot be upgraded, and an empty location holds nothing to upgrade, so `false` is the default. Supplied observations still take precedence. Invalid-argument error text is simulator wording, not native.

## Out of scope

Native eligibility policy, currencies, upgrade execution, invalid-input semantics, persistence and full upgrade-window acceptance. Era/Anniversary do not select the Mists upgrade window; tests do not force-load it. Wrath, equipment-set lock behavior and equipment-set transfers are excluded. The equipment-set producer rebuilds equipped records by item ID, without instance identity; this lane adds no identity token or equipment-set workaround. Non-admin crafting and snapshot service rewrites are excluded; no observation lifecycle is claimed for them. Split stacks require fresh supplied observations for the new cursor stack; no native eligibility inheritance policy is inferred.
