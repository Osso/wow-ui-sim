# Descriptor data

Pinned wago.tools exports: 12.1.5.70391. Item bonus/sparse records from the
shared CSV export directory; scaling tables from `data/db2/70391` (commit
`887cc18a5`).

- ItemBonus.csv SHA256 `bbb5dc694d41d2862432c18b637c327d31e43945c58b8e519dd33b9b6375d9f8`
- ItemSparse.csv SHA256 `b370bc7d3ac06e2142824da92311bcf0542740e6ffefef240103cd12bcb19a40`

Regenerate with `python3 tools/gen_item_descriptor_data.py --csv-root PATH
--scaling-root PATH`. Bonus rows retain list membership and ascending
OrderIndex within each list. WM97c verifies the 70391 base export has no repeated
OrderIndex within a parent while the native sort is not stable for large ranges;
the reader raises a named gap if a duplicate appears.

Curves are limited to those referenced by ItemSparse selectors, ItemOffsetCurve
and ItemSquishEra. Curve 121405/121406 and their points are absent from the
export; any lookup that reaches an absent curve raises a named gap error.
Scaling selectors are not themselves scaling records.
