# Retail 12.1.0 bootstrap — C03

Bounded simulator proof for [prose-2026-06-18-055; prose-2026-06-18-056; prose-2026-06-18-057](../../data/patch-api/sources/12.1.0-api-changes.txt), captured 2026-10-04. No native-client parity or live-service claim.

## What it must do

- [x] L55: bootstrap-only LoD publication runs once, defers normal files/full completion, preserves interleaved/eager file order and later full-load idempotence.
- [x] L57 bounded mechanism: hard visual dependency is deferred until full load and then precedes owner; eager dependencies execute at startup.
- [ ] L56: disabled startup scheduling test exists in wow-sim binary tests, but authorized integration/lib commands do not select it; disabled startup proof remains audit-pending.
- [ ] L57: actual complete UIParent refactoring is not established by synthetic loader fixtures.

## How it works

- [Lua API](../lua-api.md)
- [Addon loading](../addon-loading-pipeline.md)

## Implementation inventory

- `src/loader/addon.rs — phased loading`
- `src/loader/startup_addons.rs — startup discovery`
- `src/toc/mod.rs — Bootstrap annotation`

## Tests asserting this spec

- `tests/load_order.rs::lod_bootstrap_lifecycle_publishes_once_before_full_load`
- `tests/load_order.rs::lod_bootstrap_lifecycle_preserves_mixed_stream_order`
- `tests/load_order.rs::lod_bootstrap_lifecycle_eager_files_keep_literal_toc_order`
- `tests/load_order.rs::bootstrap_owner_dependency_waits_for_full_load`

## Known gaps (current cycle)

- [ ] L56: disabled startup scheduling test exists in wow-sim binary tests, but authorized integration/lib commands do not select it; disabled startup proof remains audit-pending.
- [ ] L57: actual complete UIParent refactoring is not established by synthetic loader fixtures.

## Out of scope

- Historical intermediate publication timing, native parity, live services and visual rendering beyond asserted frame state.
