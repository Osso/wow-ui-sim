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

- `tests/forbidden_aspect_creation.rs` — five `event_registration_aspect_` tests cover mutation rejection, existing delivery, callback replacement, and zero-mask controls. Ordinary/unit/all listeners use the Rust event producer; callbacks use public `FireEvent`.
- `tests/keyboard.rs` — three `always_propagate_input_` tests cover state, inherited restrictions, and handler-time mask changes through real `send_key_press` parent routing.

## Known gaps (current cycle)

Independent verification at `eb2dcbf35` retained eight focused retail policy proofs and passed four earlier-12.0.7 controls, formatting, retail checking, PTR build/startup (`[]`, exit 0), readability, and canonical audit validation. The earlier-profile build warned that `CastInfoSnapshot.delay_time` is unused in untouched `spell_api.rs`; its regression status was not established.

A pre-change fixture exposed an unrelated producer difference: public `FireEvent` delivered ordinary `UNIT_HEALTH` `OnEvent` again after `UnregisterAllEvents`, while the Rust event producer did not. This slice neither changes that dispatcher nor claims producer equivalence.

## Out of scope

Secret values, caller-taint/security enforcement, VM changes, native error wording and timing, mouse propagation, `OnKeyUp`, and changing keybinding/EditBox priority. This policy does not establish universal input propagation or Blizzard-private caller privileges. The separately scoped, still-unproven `ScriptedInput` and `QueryFocus` method policy is specified in [forbidden-aspect scripted input and focus queries](forbidden-aspect-scripted-input-query-focus.md).
