# Descriptor data

Pinned wago.tools exports: 12.1.5.70391.

- ItemBonus.csv SHA256 `bbb5dc694d41d2862432c18b637c327d31e43945c58b8e519dd33b9b6375d9f8`
- ItemSparse.csv SHA256 `b370bc7d3ac06e2142824da92311bcf0542740e6ffefef240103cd12bcb19a40`

Regenerate with `python3 tools/gen_item_descriptor_data.py --csv-root PATH`. Bonus rows retain list membership and ascending OrderIndex within each list. Equal-OrderIndex CSV stability is INFERRED; native tie order is UNKNOWN. Scaling selectors are not scaling records; unresolved scaling must error.
