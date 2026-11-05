# Conduit tooltip

## Publication and selection

Retail, PTR and Forever pinned `TooltipInfoDocumentation.lua:204` declare
`C_TooltipInfo.GetConduit(conduitID, conduitRank)`. PTR and Forever reuse the
same supplied conduit catalog; that is **INFERRED**, not separate native proof.

- The signed rank is clamped to at least 1 (WM94 §5).
- A conduit record is required (`CONDUIT_TYPES`), then the exact rank row whose
  zero-based `RankIndex` equals `rank - 1` (`CONDUIT_RANKS`). The nearest lower
  rank is never substituted.
- An unknown conduit or a rank past the supplied rows uses
  [the shared UNKNOWN failure seam](tooltip-appearance-item.md): zero return
  values are **EMULATED** simulator policy.

## Composition

| Row | Content | Type | Colour |
|---|---|---|---|
| 1 | conduit spell name | 13 (`SpellName`) | rank `QualityID` from `CONDUIT_RANK_PROPERTIES` |
| 2 | `CONDUIT_TYPE_FINESSE` / `_POTENCY` / `_ENDURANCE` | 0 (plain) | default |
| 3 | `ITEM_LEVEL` formatted with the rank item level; carries integer `itemLevel` | 31 (`ItemLevel`) | default |
| 4 | `ITEM_SOULBOUND` | 20 (`ItemBinding`) | default |
| 5 | the conduit spell's resolved description | 34 (`SpellDescription`) | `GREEN_FONT_COLOR`, wrapped |

- Missing rank-properties data yields default quality 4 and item level zero
  (WM94 §5); the label row is omitted for an unmapped conduit type.
- A missing spell-name record skips row 1 but the description stage still runs
  (WM94 §5). An empty description omits row 5.

**INFERRED layout:** the only conduit-typed value in the pinned `TooltipDataType`
enum is `EnhancedConduit` (13), so both conduit getters use it for the tooltip
type. Rows 2 and 3 use the simulator's established plain and item-level types.
The native heading and label line types are not recorded in the available
evidence, and **UNKNOWN**: the exact localized item-level and description text.

## Evidence and proof

Clean-room native source: WM94 `native-evidence-70688/WM94-tooltipinfo-getters.md`,
section 5. `Blizzard_SharedXMLGame/Tooltip/TooltipDataHandler.lua:570` supplies
the original `GameTooltip:SetConduit` accessor; the original hover site is
`Blizzard_Soulbinds/Blizzard_SoulbindsConduitList.lua:296`.

`tests/tooltip_conduit.rs` runs that consumer for two ranks of one conduit (rank
1 and rank 5: item levels 145/200, qualities 2/4) plus the Finesse and Endurance
types, asserting the name colour, type label, item-level type and numeric field,
soulbound row and description. It also proves the rank-below-one clamp, the
missing-rank rejection and the unknown-conduit miss. Native protected-return
arity and the localized strings are not claimed.
