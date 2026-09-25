# Attribute Callback Error Recovery

Commit `87f2a48af` replaces raw attribute-callback calls with `protected_lua_pcall_state`, so a handled callback failure preserves its caller state. This records RED evidence only; GREEN verification is pending.

## Content

### Root cause

Both registered and direct `OnAttributeChanged` dispatch called `LuaState::call_function` directly. On failure they invoked the Lua error handler and restored only `state.top`; the active caller frame and captured locals remained corrupted. The next Lua operation then failed with `expected Lua closure in execute`.

`/tmp/attribute-handler-error-red.log` reproduces two failures: one registered handler and one direct frame-method handler. `/tmp/retail-before.stderr` shows the same error after DandersFrames' `Wrap frame cannot be used` failure. Full addon startup recorded 480 error records, versus 2 with `--no-addons`; disabling cache did not change the reproduction.

### Change

`87f2a48af` routes both dispatch paths through `protected_lua_pcall_state` and passes the protected error to the existing error handler. The helper restores the caller state before returning while preserving the secure-delegate taint boundary.

### Status

The committed regression covers caller values, captured locals, later callbacks, and error recording for both paths. No GREEN test, full-startup replay, or clean-startup claim is recorded yet.

## Sources

- [attribute dispatch](../../../src/lua_api/frame/methods/text_attribute_event/attributes.rs) — both protected callback paths
- [Lua error reporting spec](../../specs/lua-error-reporting.md) — pending attribute-callback recovery contract
- [lua-call-frame-restoration](lua-call-frame-restoration.md) — prior call-state recovery boundary

## See Also

- [[lua-call-frame-restoration]] — direct-call and nested-dispatch restoration history
- [[retail-ptr-full-startup-lua-errors]] — broader startup-error investigation
