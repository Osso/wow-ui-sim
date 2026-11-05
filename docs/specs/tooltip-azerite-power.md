# Azerite power tooltip

## Publication and selection

Retail, PTR and Forever pinned `TooltipInfoDocumentation.lua:102` declare
`C_TooltipInfo.GetAzeritePower(itemID, itemLevel, powerID [, owningItemLink])`.
PTR and Forever use the same supplied power catalog; that shared-model reuse is
**INFERRED**, not separate native proof.

The required power ID selects one row of the generated 70391
`AZERITE_POWER_SPELLS` table (`src/c_api/tooltip_power_catalog_data.rs`). A
missing or negative power ID uses
[the shared UNKNOWN failure seam](tooltip-appearance-item.md): zero return values
are **EMULATED** simulator policy, not measured native failure arity.

## Composition

1. Read three required signed integers and the optional owning-item link. Every
   present argument is secret-checked before conversion (WM94 §5).
2. Resolve the power record's spell ID and return the common spell producer's
   data unchanged, including its spell identity and lines (WM94 §5). The producer
   has no separate extra-line invocation.
3. A covered power whose spell has no `SpellName` row in the pinned export
   publishes the identified but empty spell tooltip, the same result as
   `C_TooltipInfo.GetSpellByID` for that ID.

**UNKNOWN and not represented:** what the supplied item ID, item level and parsed
owning-item link change inside the native common producer. The simulator's common
spell producer publishes the same lines with or without item context, so the
three catalog inputs are authenticated and otherwise unused.

## Evidence and proof

Clean-room native source: WM94 `native-evidence-70688/WM94-tooltipinfo-getters.md`,
section 5. `Blizzard_SharedXMLGame/Tooltip/TooltipDataHandler.lua:563` supplies
the original `GameTooltip:SetAzeritePower` accessor.

`tests/tooltip_azerite_power.rs` runs that consumer for two distinct powers (IDs
1 and 7, spells 55564 and 41695), asserts the published spell type/identity and
the spell-name line, and rejects the resolved spell table with unknown and
negative power IDs. Native protected-return arity and the item-context effects
are not claimed.
