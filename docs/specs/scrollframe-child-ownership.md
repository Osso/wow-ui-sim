# ScrollFrame child ownership

`SetScrollChild` replaces or clears the designated content frame using the shared hierarchy model in `src/lua_api/frame/methods/widget_scroll.rs`. See [widget system](../widget-system.md).

## What it must do

- [x] Replacing the designated child detaches its previous parent link, attaches the replacement to the ScrollFrame, and updates `GetScrollChild` and child enumeration.
- [x] Clearing with nil removes the designation and detaches the old child; effective alpha and scale no longer inherit from the ScrollFrame.
- [x] Preserve unrelated siblings and the old child's descendants. Reassigning the same child must not duplicate its parent link.

These are bounded simulator requirements corroborated by local Wowless `data/uiobjects/ScrollFrame/SetScrollChild.lua`, which unparents the old child before assigning another. They are not native-client observations.

## How it works

- [Widget system](../widget-system.md)
- [Scroll presentation](scrollframe-presentation.md)

## Implementation inventory

- `src/lua_api/frame/methods/widgets/slider.rs`: public setter and getter.
- `src/lua_api/frame/methods/widget_scroll.rs`: shared assignment, clearing and cache invalidation.
- `src/lua_api/frame/methods/methods_hierarchy.rs`: parent links and inherited properties.

## Tests asserting this spec

`tests/scroll_widgets.rs`, grouped `integration` target:

- `scroll_child_replacement_detaches_old_child_without_disturbing_siblings`
- `scroll_child_nil_clears_designation_and_detaches_old_subtree`
- `scroll_child_same_child_reassignment_keeps_single_parent_link`

## Known gaps (current cycle)

Tests-only `3eea04853` fails both detach assertions while the same-child control passes. `/tmp/cross-version-scroll-child-ownership-proof.md` records exact commands and revision. Independent verification of `07afe68ae` passes 32 scroll-widget, seven hierarchy, and one XML ScrollChild test, plus format/check and integration compilation. The hierarchy error-routing control deliberately logs `reparent hide failure`; scroll and XML logs contain no Lua errors. Changed Rust readability passes. `/tmp/cross-version-scroll-child-ownership-verification-ledger.md` records revision-scoped proof; later docs-only changes do not invalidate it.

## Out of scope

Native ownership/callback/anchor-reset policy, implicit range-refresh timing, cross-ScrollFrame transfers, Lua custom `ScrollChild` properties, and invalid argument coercion. Existing anchor behavior is unchanged; no vendor changes.
