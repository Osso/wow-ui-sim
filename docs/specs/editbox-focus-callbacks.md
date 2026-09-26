# EditBox focus callbacks

Lua-driven `EditBox:SetFocus` and `EditBox:ClearFocus` transitions notify focus scripts. The implementation is in `src/lua_api/frame/methods/widgets/editbox.rs`; see [event dispatch](../event-system.md).

## What it must do

- [x] A new focus owner receives `OnEditFocusGained`; transfer sends `OnEditFocusLost` to the former owner before gaining the new owner. `HasFocus` reflects the committed transition inside callbacks.
- [x] Clearing the focus owner sends `OnEditFocusLost`; repeated `SetFocus`, `ClearFocus`, and clearing a non-owner send no callbacks.
- [x] When a loss callback redirects focus, the displaced requested owner receives no stale gained callback.
- [x] Dispatch normal scripts and `HookScript` callbacks; report handler errors and continue the transfer to the new owner's gained callback. Focus-loss hooks can clean up popups.

## How it works

- [Event system](../event-system.md)

## Implementation inventory

- `src/lua_api/frame/methods/widgets/editbox.rs` — Lua-method focus transitions and script dispatch.
- `src/lua_api/script_helpers.rs` — ordered script bindings, protected calls, and error reporting.

## Tests asserting this spec

- `tests/widget_methods.rs` — transitions, no-ops, reentry, hook cleanup, and error reporting.

## Known gaps (current cycle)

- [ ] Native-client callback ordering and reentrant focus semantics are not verified.

## Out of scope

Mouse-driven focus dispatch and vendor Lua changes are unchanged.
