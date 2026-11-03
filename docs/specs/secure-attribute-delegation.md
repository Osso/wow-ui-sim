# Secure attribute delegation

Registered `OnAttributeChanged` scripts on explicitly protected frames provide the dispatch boundary used by unchanged Blizzard `SecureHandlersUpdateFrame`. Implementation lives in `src/lua_api/frame/methods/text_attribute_event/attributes.rs`; see [protected-frame enforcement](../protected-frame-enforcement.md).

## What it must do

- [ ] Run trusted registered handlers on explicitly protected or forbidden delegate frames without inheriting the setter's stack taint; restore the caller's taint afterward.
- [ ] Preserve the callback closure's own addon taint. An addon-supplied handler must not gain permission to mutate protected state in combat.
- [ ] Enforce protected-write restrictions before dispatch; delegate handling cannot rescue a prohibited attribute write.
- [ ] Allow unchanged Blizzard secure-handler execution and wrapping from an out-of-combat addon caller; a subsequent host click must execute the installed restricted wrapper.

## How it works

- [Protected frames](../protected-frame-enforcement.md)
- [Attribute error recovery](../wiki/investigations/attribute-callback-error-recovery.md)

## Implementation inventory

- `src/lua_api/frame/methods/text_attribute_event/attributes.rs` — eligible registered-callback boundary and protected write gate.
- `src/lua_api/taint.rs` — save/clear/restore caller stack taint.

## State snippet ownership

Before the state template is ready, changed `state-*` attributes use the
simulator's `_onstate-*` fallback, including on frames with an unrelated
`OnAttributeChanged` observer.

An untainted `SecureHandlerStateTemplate` declaration marks its VM ready.
The selected TOC declares it after loading the restricted handlers and before
the state-driver files. The vendor callback then dispatches state snippets
with `self, stateid, newstate`. The restricted `self` handle provides
`RunAttribute` and frame references in the managed environment. Plain frames
receive their ordinary attribute callbacks. Lua global shadows and tainted
template declarations leave VM readiness unchanged. Rebuilding the UI
environment resets readiness.

## Tests asserting this spec

- `tests/blizzard_restricted_addon_environment_loads.rs` — actual vendor execution/wrapping, host click, caller taint and addon-handler combat denial.
- `tests/protected_attribute_enforcement.rs` — secure/insecure, combat/out-of-combat write controls.
- `tests/secure_attribute_delegate.rs` — existing forbidden-frame delegate behavior.
- [Forever state dispatch](../../tests/forever_secure_state_dispatch.rs) — restricted
  callback arguments and exactly-once execution after a late tainted template replacement.

## Known gaps (current cycle)

- [ ] Corrected runtime verification pending. Vendor execution and wrapping fail before the fix; the bounded prefork RED completes in 16.1 seconds.

## Out of scope

Vendor changes, making addon closures secure, relaxing combat restrictions, inherited-only protection semantics, and full native security conformance.
