# Descriptor data

Pinned wago.tools exports: 12.1.5.70391. Item bonus/sparse records from the
shared CSV export directory; scaling tables from `data/db2/70391` (commit
`887cc18a5`).

- ItemBonus.csv SHA256 `bbb5dc694d41d2862432c18b637c327d31e43945c58b8e519dd33b9b6375d9f8`
- ItemSparse.csv SHA256 `b370bc7d3ac06e2142824da92311bcf0542740e6ffefef240103cd12bcb19a40`
- GemProperties.csv SHA256 `8985ac1145749b62d4933903bee5755b9965066db5a78506630ed83f504fcb9b`
- SpellItemEnchantment.csv SHA256 `5247872136da2c35397ea307bc7c1be8ab485524cf53a225eac67843cc9414f8`

Regenerate with `python3 tools/gen_item_descriptor_data.py --csv-root PATH
--scaling-root PATH`. Bonus rows retain list membership and ascending
OrderIndex within each list. WM97c verifies the 70391 base export has no repeated
OrderIndex within a parent while the native sort is not stable for large ranges;
the reader raises a named gap if a duplicate appears.

Relic metadata keeps every GemProperties row and only the SpellItemEnchantment
rows those rows name. It retains the three effect positions and their argument
and minimum-point values. It does not retain other enchantment columns.

Curves are limited to those referenced by ItemSparse selectors, ItemOffsetCurve,
ItemSquishEra and the WM104 relic contribution (curve 1718). Curve 121405/121406
and their points are absent from the export; any lookup that reaches an absent
curve raises a named gap error.
Scaling selectors are not themselves scaling records.
