# FontString scale animation mode

## Contract

Pinned `SimpleFontStringAPIDocumentation.lua` declares Get/SetScaleAnimationMode; `UISharedDocumentation.lua` defines FontSize=0 and Vertex=1.

- Retail 12.0.0 and later expose explicitly-set numeric mode roundtrips independently per FontString.
- Getter returns one number; setter returns no values.
- Mode is distinct from text-scale magnitude.
- Simulator policy initializes mode to 0 and accepts only numeric 0/1. Native defaults, validation/coercion and exact errors are unverified.

## Proof

`tests/widget_methods_colorselect.rs`: four focused tests, committed `fc24b8e4f`, RED 1/4 (enum values passed; missing setter blocked other assertions). Independent proof reuses exact bytes and passes 4/4 each on 12.0.0/12.0.5/12.0.7; fmt/check/build/startup (`[]`)/readability passed. Final metadata validation at `88e4998ee` passes the validator with 14,897 fresh hashes, zero stale hashes, 26 renewals, and ten additions; two credits update totals to **2312 / 1096 / 2**. Ledger: `/tmp/verify-fontstring-scale-mode-metadata-ledger.json`.

## Gaps

Animation, layout, vertex scaling, justification, rendering, native lifecycle and earlier-profile runtime publication remain unproven. Secret-argument enforcement is deferred. State roundtrips do not establish animation behavior.
