# Cached vendor auto-hide runtime

Bounded simulator coverage of the active profile's cached `Blizzard_RestrictedAddOnEnvironment/SecureHoverDriver.lua`, loaded through its actual addon dependency closure. This is a runtime test contract, not a new simulator driver or a native/historical compatibility claim. See the [source audit and limits](../wiki/investigations/patch-3-1-0-api-audit.md).

## What it must do

- [x] A normal/insecure caller, explicitly closure-tainted as `AutoHideRuntimeFixture`, can call the registered `RegisterAutoHide` for a shown frame with duration 1.0. With host cursor input selected from observed resolved geometry, real OnUpdate ticks enter then leave the frame; it stays shown before the duration and hides strictly after it.
- [x] Calling the registered `UnregisterAutoHide` from that caller while expiry is pending leaves the frame shown after more than duration 1.0 has elapsed outside.

- [x] Before the first update, the insecure caller adds a disjoint child rectangle with `AddToAutoHide`. At the child-only cursor point, treatment stays shown while otherwise equivalent control expires; after treatment exits both rectangles, its countdown expires normally.

## How it works

- [Frame geometry and layout](../layout-system.md)
- [Addon dependency loading](../addon-loading-pipeline.md)
- [Script/event dispatch](../event-system.md)

## Implementation inventory

- Active profile cache: `Blizzard_RestrictedAddOnEnvironment/SecureHoverDriver.lua` — existing vendor registration, hover countdown and hide driver; unchanged.
- `tests/common/blizzard_addon_harness.rs` — existing actual cached addon closure loader; unchanged.
- `src/lua_api/rect_geometry.rs` — resolves frame geometry queries; unchanged.
- `src/lua_api/globals/real/mouse_probes.rs` — reads host mouse position, converting renderer Y to screen Y; unchanged.
- `src/lua_api/env_runtime.rs` and `src/lua_api/script_helpers/event_dispatch.rs` — actual registered OnUpdate entry/dispatch; unchanged.

## Tests asserting this spec

Target `integration`, module `secure_group_headers` (registered by `build.rs` top-level test discovery):

- `secure_group_headers::cached_auto_hide_child_rectangle_extends_hover_set`
- `secure_group_headers::cached_auto_hide_enter_leave_expires_after_duration`
- `secure_group_headers::cached_auto_hide_unregister_cancels_pending_expiry`

Combined filter: `secure_group_headers::cached_auto_hide_`.

Prerequisites: populated active-profile Blizzard UI cache resolved by `default_blizzard_ui_addons_path()`, including the root `Blizzard_RestrictedAddOnEnvironment`, declared dependency `Blizzard_FrameXML` and its transitive closure. Both addons must finish loading without recorded Lua errors; the vendor manager must retain its real OnUpdate and OnAttributeChanged scripts. No cache synchronization is performed by these tests. Target geometry must resolve to a finite nonempty rectangle. Closure taint uses the existing simulator test pattern; the tests do not change manager scripts or invoke callbacks directly. Fixture and tick diagnostics report fixed stage labels/counts, not arbitrary Lua payloads.

## Known gaps (current cycle)

Saved retail receipts at `9635468d994be53d1f3571a8005115a49864c18a`, verified 2026-10-10: three distinct cases passed, zero failed/ignored, runtime/compile/fmt exit 0. Both original cases remain byte-identical. [Sanitized audit](/home/osso/Projects/wow/wow-ui-sim/data/patch-api/evidence/3.1.0-session-2026-10-09/child-rectangle-20261010/audit.md), [receipts](/home/osso/Projects/wow/wow-ui-sim/data/patch-api/evidence/3.1.0-session-2026-10-09/child-rectangle-20261010/receipts.json), and [SHA manifest](/home/osso/Projects/wow/wow-ui-sim/data/patch-api/evidence/3.1.0-session-2026-10-09/child-rectangle-20261010/sha256-manifest.json) retain the scoped proof. Compile retains six existing manifest deprecation warnings; no cargo check or warning-clean claim.

Runtime vendor-cache inputs and explicit host/cache provenance guard are not sealed. This proves the saved cached-driver simulator assertions, not clean startup, native/historical parity, installed third-party addon loading, all-profile/full-suite acceptance or publication/count credit. Main owns suite integration and accounting.

## Out of scope

Native/security/historical buggy/largely-unusable parity, clean startup, combat behavior, exact deadline equality, re-entry, re-registration, movement cancellation, other profiles' acceptance, and production/vendor changes. Scope is exactly the three cases above.
