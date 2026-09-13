# FontString scale animation mode

## Contract

Pinned `SimpleFontStringAPIDocumentation.lua` declares Get/SetScaleAnimationMode; `UISharedDocumentation.lua` defines FontSize=0 and Vertex=1.

- Retail 12.0.0 and later expose explicitly-set numeric mode roundtrips independently per FontString.
- Getter returns one number; setter returns no values.
- Mode is distinct from text-scale magnitude.
- Simulator policy initializes mode to 0 and accepts only numeric 0/1. Native defaults, validation/coercion and exact errors are unverified.

## Proof

`tests/widget_methods_colorselect.rs`: four focused tests, committed `fc24b8e4f`, RED 1/4 (enum values passed; missing setter blocked other assertions). Post-implementation proof pending.

## Gaps

Animation, layout, vertex scaling, justification, rendering, native lifecycle and earlier-profile runtime publication remain unproven. Secret-argument enforcement is deferred. State roundtrips do not establish animation behavior.
