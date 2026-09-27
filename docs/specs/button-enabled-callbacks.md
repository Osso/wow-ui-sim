# Button enabled-state callbacks

`Enable`, `Disable`, and `SetEnabled` dispatch registered transition scripts through the simulator's existing binding model. Source: `src/lua_api/frame/methods/button_anchor_hierarchy/buttons.rs`; see [widget system](../widget-system.md).

## What it must do

- [ ] On an enabled-state change, run intrinsic precall, normal script/hooks, and intrinsic postcall for `OnEnable` or `OnDisable`, with the button as the sole argument and committed state visible to handlers.
- [ ] Retain unchanged-state no-ops and normal hook-only behavior.
- [ ] Report handler errors and continue later bindings without undoing the state change.

## How it works

- [XML template system](../xml-template-system.md)
- [Event dispatch](../event-system.md)

## Implementation inventory

- `src/lua_api/frame/methods/button_anchor_hierarchy/buttons.rs`: state transitions and binding dispatch.
- `src/lua_api/script_helpers.rs`: existing ordered binding lookup, reused unchanged.

## Tests asserting this spec

`tests/methods_button/enabled_bindings.rs`, within the existing grouped integration target, loads XML pre/post declarations and confirms their presence through public `GetScript` binding queries before exercising transitions.

## Known gaps (current cycle)

Tests-only `960c20b32` is RED in two dispatch cases: registered intrinsic bindings are skipped, leaving only normal/hook calls. The normal hook-only control passes; there is no normal-hook omission bug. `/tmp/cross-version-button-enabled-bindings-proof.md` records exact revisions and outputs. Post-change verification is pending.

This corrects a demonstrated mismatch between registration and dispatch in the simulator's established XML binding model; it is not a fresh native-client probe. Intrinsic `HookScript` restrictions remain unchanged.

## Out of scope

`SetButtonState` semantics, argument coercion, opposite-state recursive transitions, native all-profile parity, and vendor/UI changes.
