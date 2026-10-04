# Retail 12.1.0 framexml-migrations — C02

Bounded simulator proof for [prose-undated-011; prose-undated-012](../../data/patch-api/sources/12.1.0-api-changes.txt), captured 2026-10-04. No native-client parity or live-service claim.

## What it must do

- [x] L11: migrated loader loads a real disk addon exactly once, returns one value, displays missing/disabled errors and suppresses repeated missing-name dialogs; old UIParentLoadAddOn absent.
- [x] L12: migrated mouse helper uses actual native frame hit bounds, unequal positional offset forwarding, one return and hidden frame rejection; MouseIsOver absent.
- [ ] Native offset naming/sign conventions and scaled-frame cursor parity are not established; assertions bound to current native positional forwarding.

## How it works

- [Lua API](../lua-api.md)
- [Addon loading](../addon-loading-pipeline.md)

## Implementation inventory

- `cached Blizzard_SharedXML/AddOnUtil.lua and InputUtil.lua — unmodified helpers`
- `src/lua_api/frame/methods/core_state/region.rs — native mouse bounds`
- `src/c_api/c_addons.rs — addon state/loading`

## Tests asserting this spec

- `tests/patch_12_1_0_framexml_migrations.rs — both patch_12_1_0_migrated tests`

## Known gaps (current cycle)

- [ ] Native offset naming/sign conventions and scaled-frame cursor parity are not established; assertions bound to current native positional forwarding.

## Out of scope

- Historical intermediate publication timing, native parity, live services and visual rendering beyond asserted frame state.
