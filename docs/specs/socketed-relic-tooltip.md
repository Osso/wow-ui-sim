# Socketed relic tooltip

`C_TooltipInfo.GetSocketedRelic(slotIndex)` returns the tooltip data for the relic
in one socket of the currently viewed artifact. `GameTooltip:SetSocketedRelic`
dispatches it, and `Blizzard_ArtifactUI`'s title hover
(`ArtifactTitleTemplateMixin:OnRelicSlotMouseEnter`) is the original consumer.

Retail, PTR and Forever declare this getter at
`Blizzard_APIDocumentationGenerated/TooltipInfoDocumentation.lua:1012`
(`MayReturnNothing = true`, `SecretArguments = AllowedWhenUntainted`). Wrath,
Mists, Era and Anniversary do not, so the getter is not published there.

## Inputs

- The viewed artifact identity supplies the catalog item. Its
  `ItemSparse.ArtifactID` must be nonzero.
- The internal slot must be below 3, below the nonzero socket-marker count of the
  artifact's item record, and below the derived socket count. The derived count
  adds the artifact instance's extra contribution: item-state flag bit 8192
  suppresses it, and otherwise enchantment entry index 6 of the item instance is
  resolved in `SpellItemEnchantment`, whose first type-8 effect supplies its
  `EffectPointsMin`.
- The selected live socket supplies an item descriptor (item ID, optional bonus
  context and modifiers). A socket without a descriptor, or an index outside the
  bounds above, is a miss. No catalog item is fabricated.

## Output

The descriptor is resolved as an owned instance: its bonus lists set the item
level, quality, stats and required level before composition. The result contains
the shared item lines plus the relic lines of the item's `GemProperties` row:

- **Type line**: `RELIC_TOOLTIP_TYPE` with the `RELIC_SLOT_TYPE_<token>` label
  selected by `GemProperties.Type` (64 Iron, 128 Blood, 256 Shadow, 512 Fel,
  1024 Arcane, 2048 Frost, 4096 Fire, 8192 Life, 16384 Water, 32768 Storm,
  65536 Holy). Any other exact value uses the `UNKNOWN` token. The line uses
  artifact coloring.
- **Item-level line**: `RELIC_TOOLTIP_ILVL_INCREASE` appears only when the
  contribution is strictly positive. It never changes the item level.

`SpellItemEnchantment` effects run in order 0, 1, 2 into one fresh accumulator:
effect 11 applies its nonzero argument as a bonus list; effect 12 clamps the
relic's display item level to 1–1300, evaluates curve 1718 in single precision,
truncates toward zero, floors at 1, and applies the `ItemBonusListLevelDelta` row
whose `ItemLevelDelta` equals that value exactly. A miss skips the effect.

The published number is the Type 42 absolute-level selection, not the Type 1
level sum. In the pinned 70391 data every effect-11 and delta list carries only
Type 1 rows, so base relic tooltips show no item-level number.

## Data and profiles

Pinned 70391: `GemProperties`, the `SpellItemEnchantment` rows it names, curve
1718, `ItemBonusListLevelDelta`, and `ItemSparse` artifact/socket/gem fields.
Native evidence is Retail 12.1.5 build 70688 (WM97 DQ4, WM104).

The relic joins are Retail records and PTR shares that descriptor set. Forever
publishes the getter but applies none of these records: a supplied socket
descriptor resolves the shared catalog item only, and a session without a viewed
artifact takes the ordinary miss path. No Retail 70391 relic record reaches
Forever.

## Known limits

- The artifact identity is the panel's viewed artifact; the equipped main-hand or
  off-hand fallback is not modelled.
- Artifact socket markers come from the catalog item record; bonus-added sockets
  on a resolved artifact description are not modelled.
- Owner floor/cap, stored instance level, artifact attunement text and live
  hotfix rows are not modelled.
- The native failure arity for a missing artifact, socket or descriptor is
  UNKNOWN; the simulator returns no values, and the caller's tooltip hides.
