# Forbidden-aspect event and keyboard policies

Patch 12.1 names `EventRegistrations` and `AlwaysPropagateInput` in `Enum.ForbiddenAspect`. Their runtime semantics here are explicit **simulator assumptions**, not native security conformance. Stored masks and inheritance remain governed by [forbidden-aspect inheritance](forbidden-aspect-inheritance.md).

## What it must do

### Event registration — modeled policy

- [x] Reject `RegisterEvent`, `RegisterUnitEvent`, `RegisterAllEvents`, `RegisterEventCallback`, `RegisterUnitEventCallback`, `UnregisterEvent`, and `UnregisterAllEvents` when the receiver has `EventRegistrations`.
- [x] Reject before changing membership, unit filters, all-event listeners, callback storage, or dispatch indexes. Existing registrations and callbacks continue to deliver events; queries remain available.
- [x] Apply the restriction uniformly, without a caller-taint exemption. Unrestricted frames retain existing validation, return values, registration, and delivery behavior. Engine-owned cleanup is not a Lua registration mutation.

### Keyboard propagation — modeled policy

- [x] A frame carrying `AlwaysPropagateInput` reports effective keyboard propagation as true, including an inherited mask and a mask added after propagation was disabled.
- [x] Reject disabling propagation on such a frame without changing state; explicitly enabling it remains allowed.
- [x] The existing parent-chain `OnKeyDown` dispatcher uses effective propagation after the handler returns, including a mask added by that handler.
- [x] Zero-mask frames and earlier profiles retain ordinary propagation behavior and key-dispatch ordering.

## How it works

- [Event system](../event-system.md)
- [Keybinding system](../keybinding-system.md)
- [Patch 12.1 audit](../wiki/investigations/patch-12-1-api-audit.md)

## Implementation inventory

- `src/lua_api/frame/methods/forbidden_aspects.rs` — stored-aspect restrictions and effective keyboard propagation.
- `src/lua_api/frame/methods/text_attribute_event/events.rs` — Lua mutation gates and propagation queries.
- `src/lua_api/key_dispatch.rs` — parent-chain keyboard routing.

## Tests asserting this spec

Focused grouped integration tests exercise registration outcomes and real event delivery, and `tests/keyboard.rs` exercises real `send_key_press` parent routing.

## Known gaps (current cycle)

- [ ] Native routing, caller authority, error wording, and security equivalence remain unverified.

## Out of scope

Secret values, caller-taint/security enforcement, VM changes, native error wording and timing, other forbidden aspects, mouse propagation, `OnKeyUp`, and changing keybinding/EditBox priority. This policy does not establish universal input propagation or Blizzard-private caller privileges.
