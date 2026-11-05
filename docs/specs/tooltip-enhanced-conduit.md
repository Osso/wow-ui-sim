# GetEnhancedConduit

`C_TooltipInfo.GetEnhancedConduit(conduitID, rank)` composes the enhanced-conduit
tooltip from the conduit entry and its rank rows. Retail, PTR and Forever pinned
`TooltipInfoDocumentation.lua:254` declare it.

## Requirements

1. **Membership.** The conduit entry id is the membership key, the same lookup
   `GetConduit` performs. There is no separate enhanced-socket table. Because the
   earlier lookup must succeed, the two-rank increase applies to every valid
   conduit.
2. **Rank adjustment.** The original rank is `max(rank, 1)`. The cap is the number
   of `SoulbindConduitRank` rows for that conduit, 15 in the pinned 70391 export.
   The adjusted rank is `original + 2` with 32-bit wrap, floored to 1 and clamped
   to the cap. The exact rank row (`RankIndex == adjusted - 1`) is required; a
   lower rank is never substituted.
3. **Split sources.** Quality and item level come from the rank-properties row of
   the **original** clamped rank. The spell id and scaling come from the
   **adjusted** rank row. The name colour follows the original rank's quality.
4. **Rows.** Quality-coloured spell-name heading when the spell-name record
   exists, the Finesse/Potency/Endurance label, the `SOULBIND_CONDUIT_ENHANCED`
   label, an item-level row carrying integer `itemLevel`, `ITEM_SOULBOUND`, then
   the spell description in green. Highlight-colour tokens in the description
   become enhanced-colour tokens; the comparison ignores hex case.
5. **Missing data.** An unknown conduit or a missing rank row returns no values.
   A missing rank-properties row uses quality 4 and item level 0.

## Emulated seam

The enhanced colour resolves through the simulator's named-colour surface. The
pinned evidence does not record the `GlobalColor` value, so the token rewrite is
a no-op while that surface publishes the placeholder white. No colour value is
invented. The enhanced label would also be re-typed from 0 to 41 by the shared
value-based remap while the colour resolves white, so the producer restores the
verified type 0 after appending it. Retire both behaviours when the colour table
is modelled.

## Unverified

The final native nil/no-values form for a missing conduit and the exact localized
description rendering. The published tooltip data type is
`Enum.TooltipDataType.EnhancedConduit` (13), the only conduit-typed value.
