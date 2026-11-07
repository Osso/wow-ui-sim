# Loot and buyback tooltip data

Retail, PTR and Forever publish these getters; the original tooltip handler displays the item data.

## What it must do

### Common

- [x] `GetLootItem` resolves a one-based slot in supplied loot state.
- [x] `GetBuybackItem` resolves a one-based slot in supplied buyback state.
- [x] The original tooltip handler displays the selected item's data.
- [ ] Missing slot behavior is **EMULATED** as no returned values. The native
      result count is stored as 1 only on success (**VERIFIED**), and the publication tail that
      runs with a count of 0 is **UNKNOWN**; "returns nothing" is therefore **INFERRED**, and no
      native parity is claimed for the arity itself.

### `GetBuybackItem(index)` index contract

Evidence: Round 6 of `pr46-49-decomp-answers.md`, Retail 12.1.5 (**70688**) and Forever 1.60.1
(**70707**), getter bodies decoded in both builds.

- [x] The reader requires a Lua number, accepts `0` to `4294967295`, subtracts `1.0` and truncates
      toward zero, and has no integral test. `1.9` selects entry 1 and `2.9` selects entry 2
      (**VERIFIED** in both builds).
- [x] A value at or above `0` and below `1` truncates toward zero after the subtraction and
      therefore also selects entry 1 (**INFERRED** from the verified subtract-then-truncate order).
- [x] The index is bounded by the live buyback entry count, not by the 12-slot storage bound.
      Index `0` becomes unsigned all-ones after the subtraction, and a 13th entry never exists, so
      both take the same miss arm as an absent entry (**VERIFIED**: one bound check, one failure
      target).
- [x] An entry whose item does not resolve takes that same miss arm, so no empty tooltip table is
      published (**VERIFIED** that an unresolvable item is a miss condition).
- [x] A missing or non-number argument reports the getter's own usage text,
      `Usage: local data = C_TooltipInfo.GetBuybackItem(index)` (**VERIFIED** literal and route).
- [x] A negative, above-domain or non-finite number reports
      `outside of expected range 0 to 4294967295` followed by the usage text (**VERIFIED**
      composition). The final Lua envelope around either message is **UNKNOWN**, so only those
      fragments are guaranteed text.
- [x] A resolved entry publishes the shared item tooltip, whose identity type is `Item` (0), through
      the same producer the by-ID item getter uses. This route does not copy the item instance's
      gem/socket projection (**VERIFIED**: those calls are absent), so a gemmed buyback item lacks
      the instance gem rows (**INFERRED**).
- [x] The original consumers reach this getter: `MerchantItemButton_OnEnter`
      (`Blizzard_UIPanels_Game/Mainline/MerchantFrame.lua:717`, buyback tab) and
      `MerchantBuyBackButton_OnEnter` (`:1081`) call `GameTooltip:SetBuybackItem`, which the
      handler map at `Blizzard_SharedXMLGame/Tooltip/TooltipDataHandler.lua:545` routes to this
      getter (**INFERRED** from shipped Lua; asserted by `tests/tooltip_buyback_item_index.rs`).

### `GetLootItem(slot)` index contract

Evidence: Round 6-B §1 of `pr46-49-decomp-answers.md`, Retail 12.1.5 (**70688**) and Forever
1.60.1 (**70707**), getter bodies decoded in both builds.

- [x] The reader is the same one-based reader as `GetBuybackItem`: a Lua number is required, the
      domain is `0` to `4294967295`, the value is reduced by `1.0` and truncated toward zero with no
      integral test, so `1.9` selects slot 1 and `2.9` selects slot 2 (**VERIFIED** for the Forever
      reader, **INFERRED** for 12.1.5 from the identical reason string and guard).
- [x] A value in `(0,1)` truncates toward zero after the subtraction and therefore also selects
      slot 1 (**INFERRED** from the verified subtract-then-truncate order).
- [x] The bound is the live loot slot count, compared unsigned, so slot `0` and every slot at or
      past the count take the miss arm (**VERIFIED** in both builds).
- [x] An entry whose item identity is empty is a miss condition of the loot-item reader
      (**VERIFIED**). INFERRED: an empty identity is item id `0`.
- [x] A missing or non-number argument reports `Usage: local data = C_TooltipInfo.GetLootItem(slot)`
      (**VERIFIED** literal, character for character, in both images).
- [x] A negative, above-domain or non-finite number reports
      `outside of expected range 0 to 4294967295` followed by the usage text (**VERIFIED**
      composition). The final Lua envelope is **UNKNOWN**, so only those fragments are guaranteed.
- [x] The getters never consult loot window or frame state: they read a process-global slot count
      and entry array, and the tooltip service handle passed into the callback is never read
      (**VERIFIED**). With an empty list every slot fails the bound test, so no window gating is
      modelled and a closed window does not gate a slot the list still holds.
- [x] A resolved entry publishes the shared item producer's table, identity type `Item` (0)
      (**VERIFIED**).
- [x] The original consumer reaches this getter: `LootFrameItemElementMixin:OnEnter`
      (`Blizzard_UIPanels_Game/Mainline/LootFrame.lua:353` Retail, `:631` Forever) calls
      `GameTooltip:SetLootItem`, which the handler map at
      `Blizzard_SharedXMLGame/Tooltip/TooltipDataHandler.lua:537` routes to this getter
      (**INFERRED** from shipped Lua; asserted by `tests/tooltip_loot_item_index.rs`).

## Not modelled (native behaviour with no simulator counterpart)

- The loot entry's **kind byte** (item `0`, currency non-zero) is not modelled: `SimState.loot_slots`
  holds item entries only, so no currency slot can exist and the loot-item reader's kind test has no
  state to act on. `GetLootCurrency` is a separate lane and is not registered here.
- Whether the client clears the global loot list when the loot frame closes is **UNKNOWN** (it
  depends on the loot-frame writer, not on these getters). The simulator's `CloseLoot` clears only
  `loot_frame_open`, so a slot the list still holds stays readable.
- The native entry maps the Lua slot to a loot entry and requires a non-empty item identity; the
  simulator stores the item id directly, so the entry lookup itself has no counterpart.
- Top-level `guid` and `hyperlink` for these paths are **UNKNOWN** (nothing decoded sets them, and
  the item producer is only partially decoded).
- The ordered line-type vocabulary of the shared item producer is **UNKNOWN**.

- The native entry maps the Lua index to the player's inventory slot through the buyback slot
  table, then requires a nonzero slot and a non-empty inventory GUID at that slot. Supplied
  buyback entries carry the item identity directly, so those lookups and the "no local player"
  condition have no state to act on here.
- Top-level `guid`: the producer writes the resolved item's GUID, and this route supplies the
  player GUID and the item GUID to the request (**VERIFIED** at the call sites). The simulator
  models no item GUID, so it publishes none.
- Top-level `id`: the getter's own id publication is **UNKNOWN**; the shared simulator builder
  publishes the item id for every item tooltip.
- The ordered line-type vocabulary of the shared item producer is **UNKNOWN**.

## How it works

- [Merchant cursor presentation](merchant-cursor-presentation.md)

- [Tooltip and Lua API](../lua-api.md)

## Implementation inventory

- `src/c_api/tooltip_getter_args.rs`: `one_based_index`, the shared subtract-then-truncate reader
  with the two verified argument-error texts.
- `src/c_api/c_tooltip_info_loot.rs`: supplied item selection and the missing-result seam.
- `src/lua_api/globals/missing_surface/tooltip_info/`: item tooltip publication.

## Tests asserting this spec

- `tests/tooltip_data_loot_consumer.rs`
- `tests/tooltip_data_buyback_consumer.rs`
- `tests/tooltip_buyback_item_index.rs`
- `tests/tooltip_loot_item_index.rs`

## Known gaps (current cycle)

- [x] `GetLootItem` uses the shared one-based reader, so fractional and above-`2^31` indices are
      accepted and the live loot slot count bounds them. `GetTotem` still parses its argument with
      the strict unsigned reader (`c_tooltip_info_totem.rs::read_slot`), so fractional and
      above-`2^31` indices are not accepted there; Round 6 §GetTotem records the same reader family
      with usage `Usage: local data = C_TooltipInfo.GetTotem(slot)`.
- [x] Shared D56 publication and presentation-side markup parsing follow the [serialization contract](tooltip-data-serialization.md). Getter-specific nested-row eligibility remains unknown.

## Out of scope

- Native missing-result arity and unsupplied item metadata.
