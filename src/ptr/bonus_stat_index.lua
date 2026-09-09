-- Gethe 49b69918fcdc77e109813281e4f537d45ec7dcbf: preserve base members 0–82.
for value = 83, 141 do
  Enum.BonusStatIndex["Reserved_" .. value] = value
end
Enum.BonusStatIndexMeta = {
  MinValue = 0,
  MaxValue = 141,
  NumValues = 142,
}
