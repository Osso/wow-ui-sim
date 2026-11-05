# GetArtifactPowerByID

`C_TooltipInfo.GetArtifactPowerByID(powerID)` composes one artifact-power rank
row and its spell text. Retail, PTR and Forever pinned
`TooltipInfoDocumentation.lua:53` declare it.

## Requirements

1. **Power record.** The `ArtifactPower` row supplies `ArtifactID`,
   `MaxPurchasableRank`, `Flags` and `Tier`. A missing row returns no values.
2. **Rank selection.** The stored rank comes from the current artifact's power
   entry. A stored zero selects rank 1, and `Flags & 0x40` forces rank 1. The
   rank record (`RankIndex == selected - 1`) supplies the spell id; a missing
   rank record returns no values. Missing artifact state supplies zero counts; it
   is not an unconditional rejection.
3. **Rank row.** Emitted only when neither `0x40` nor `0x4` is set. It shows
   `TOOLTIP_TALENT_RANK` with the stored rank and the displayed maximum.
   `INCREASE_STAT_COLOR` is used for a nonzero bonus, `HIGHLIGHT_FONT_COLOR`
   otherwise. The blank row that follows it is also the spacer before the spell
   text, so the spacer exists only when the rank row is shown.
4. **Displayed maximum.** `MaxPurchasableRank`, plus the current artifact tier
   minus the power `Tier` when `0x20` is set, and 255 when `0x8` is set. When
   `0x10` is set, a positive bonus is reduced by one before the colour test.
   The result is clamped to `0..=255`.
5. **Rows.** Spell-name heading when the name record exists, the conditional rank
   row, the blank spacer, then the spell description in green.

## Emulated seams

Each seam is named in the module:

- **Footer gate.** The pinned export has no `Artifact` table, so the simulator
  cannot resolve an artifact object to `ArtifactCategoryID == 2`. WM108 makes the
  footer-suppression query true in that case, so every footer row — cost, learn,
  end-game unlinked and unlinked — is skipped. Affordability, `ArtifactCanLearn`
  and `ArtifactOtherLinked` stay unreached rather than invented.
- **Raw-maximum player bit.** The evidence does not identify the local
  player-state bit that selects the raw maximum, so the capped maximum is
  published.
- **Data type.** WM94/WM108 do not record the published `data.type`. The
  simulator publishes 1 (`Enum.TooltipDataType.Spell`) because the composition is
  a name plus spell text.
- **Rank-row line type.** Not recorded; the producer owns a plain row and
  restores that type after the shared colour remap.

## Not modelled

`Flags` masks `0x1` and `0x80` are not read in this tooltip path, so they do not
change the output.
