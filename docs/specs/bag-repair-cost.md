# Retail bag-item repair cost

## Contract and evidence

VERIFIED native evidence: R88n.4b, Retail 12.1.5.70688, reproduced on tracker lt-235b comment 1212. The ordinary bag-item tooltip calls shared item composition and then a separate repair-cost producer. Only a positive queried cost adds the top-level integer `repairCost`. Zero or absent input adds no field. This is not a sell-price line field.

The simulator's explicit `bag_item_repair_costs` map contains host-supplied quotes keyed by bag/slot alongside its bag snapshot. It is empty by default. The quote calculation and native upstream writers remain UNKNOWN and unmodeled. The host must keep quotes consistent with its bag snapshot; no inventory mutation or durability event invents a quote. A missing bag item has no quote projection.

Publication is restricted to ACTIVE Retail and the ordinary `GetBagItem` path, after existing item composition. The `GetItem` item-location route retains its prior composition; no evidence here extends repair publication to it. Other profiles, battle-pet fields and type-11 `price`/`maxPrice` are unchanged.

## Original consumer selection

Pinned Retail `Blizzard_SharedXMLGame.toc:4` selects Game; line 9 admits TooltipDataHandler except vanilla and line 10 admits TooltipDataRules for mainline. `TooltipDataHandler.lua:506–517` constructs the original accessor and line 543 maps `SetBagItem` to `GetBagItem`. `TooltipDataRules.lua:151–156` reads the top-level field and adds a repair label/money in repair mode; line 177 registers the item postcall. `Blizzard_GameTooltip_Mainline.toc:5,8` selects original Mainline Lua; `Mainline/GameTooltip.lua:315–322` formats the actual repair money line.

The optional Retail selector `tooltip_bag_repair_cost::original_bag_tooltip_repair_rule_consumes_distinct_positive_costs_and_omits_zero_absent` verifies selection/source ownership, two distinct backed item IDs 6948/7005 at costs 731/2468, and zero/absent quotes. It asserts the produced integer, absence of line-level repair data, and the real rendered repair label/money FontString. Actual merchant/cursor state enables repair mode; no vendor or API function is replaced. Unrelated preexisting load errors are retained, with no added errors allowed from this path. Either a constant or no-op producer fails this selector independently.

## Sources

- Tracker lt-235b comment 1212, native R88n.4b.
- `src/lua_api/globals/missing_surface/tooltip_info/sources.rs` and explicit state input.
- Original pinned Retail cache files cited above; owner repository only.
