# Retail 12.1.0 inheritance — C07

Bounded simulator proof for [prose-2026-06-23-090](../../data/patch-api/sources/12.1.0-api-changes.txt), captured 2026-10-04. No native-client parity or live-service claim.

## What it must do

- [x] L90: SetPoint/SetParent reject acquiring previously absent layout forbidden aspects; failed calls leave zero anchor count and do not acquire foreign parent. Hierarchy inheritance at initial creation remains transitive and path-specific.
- [ ] Preservation of preexisting nonempty anchor tuples and every forbidden-aspect bit is not exhaustive.

## How it works

- [Lua API](../lua-api.md)
- [Addon loading](../addon-loading-pipeline.md)

## Implementation inventory

- `src/lua_api/frame/methods/ — shared forbidden acquisition guard`

## Tests asserting this spec

- `tests/forbidden_aspect_creation.rs::native_children_inherit_only_hierarchy_aspects_at_creation`

## Known gaps (current cycle)

- [ ] Preservation of preexisting nonempty anchor tuples and every forbidden-aspect bit is not exhaustive.

## Out of scope

- Historical intermediate publication timing, native parity, live services and visual rendering beyond asserted frame state.
