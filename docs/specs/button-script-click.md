# Scripted Button clicks

Public Lua `Button:Click` dispatches the scripted click lifecycle. The implementation lives in `src/lua_api/frame/methods/button_anchor_hierarchy/buttons.rs`; see [Lua API](../lua-api.md).

## What it must do

- [x] Dispatch `PreClick`, `OnClick`, then `PostClick`, including registered hooks. Missing `OnClick` must not suppress the other phases.
- [x] Pass the button itself, supplied mouse-button name and down flag to callbacks; omitted arguments default to `LeftButton` and `false`.
- [x] Suppress callbacks for disabled buttons and same-button recursive clicks; allow another button to dispatch. Enabling restores dispatch.
- [x] Toggle CheckButton state before enabled and recursion guards; disabled or recursive script clicks may change checked state without dispatching callbacks.
- [x] Route handler errors through the error handler, continue later phases and release the recursion guard so a later click can run.
- [x] Preserve the `ScriptedInput` forbidden-aspect guard before mutation or callbacks.

## How it works

- [Lua API](../lua-api.md)
- [Event dispatch](../event-system.md)

## Implementation inventory

- `src/lua_api/frame/methods/button_anchor_hierarchy/buttons.rs`: public click, enabled/recursion checks, ordered dispatch and error cleanup.
- `src/lua_api/frame/methods/widgets/slider/checkbutton.rs`: checked-state and texture updates.
- `src/lua_api/script_helpers.rs`: registered script bindings and error routing.

## Tests asserting this spec

- `tests/methods_button.rs`: scripted phases, arguments/hooks, disabled state, CheckButton recursion, independent-button recursion and handler-error recovery.
- `tests/forbidden_aspect_creation.rs`: scripted-input guards.

## Known gaps (current cycle)

Tests-only `8ff05b19a` reproduces six failures; two new and two existing controls pass. `/tmp/cross-version-button-click-proof.md` records exact commands and revision. Independent verification of `7b5f40bf3` passes 37 scoped tests: 31 Button cases, two CheckButton texture controls, one scripted-input guard and three hook/taint controls. Format/check, grouped integration compilation and changed Rust readability pass. Two deliberate handler errors are captured and asserted; other scoped logs have no Lua errors. `/tmp/cross-version-button-click-verification-ledger.md` records exact proof and later concurrent-change scope. Intrinsic pre/post binding order is source-inspected, not directly covered by these new fixtures.

Local Wowless `data/uiobjects/Button/Click.lua` and its product schema corroborate phase order, enabled/recursion checks and arguments. Its protected `CallSandbox` dispatch reports errors without aborting the following phases. Solarity `crates/ui/src/script/simple_script/buttons.rs` independently corroborates phase order and explicitly places CheckButton toggling before the guards; unlike Wowless it propagates handler errors out of its phase loop. This simulator retains error-handler reporting and follows Wowless's continuation model. Wowless has no separate CheckButton click wrapper; it does not corroborate toggle ordering. None of these comparisons is a fresh native-client probe.

## Out of scope

Physical mouse dispatch, native-client parity across profiles, legacy global `this`/`arg1`/`arg2` emulation, and invalid-argument coercion. Existing unrelated enable/disable behavior and vendor code are unchanged.
