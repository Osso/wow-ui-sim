# Pending secure callback storage — 12.0.7

Eight [12.0.7 source](../../data/patch-api/sources/12.0.7-api-changes.txt) rows (46, 47, 55–57, 59–61) publish namespace ping callback registration/clear and legacy button, ping and run callback accessors. This contract covers environment-local callback slots, not native secure/gamepad invocation. Cached generated documentation may postdate 12.0.7: `PingManagerSecureDocumentation.lua:159–169` declares `SetPendingPingOffScreenCallback(cb)` with `cb: PendingPingOffScreenCallback`, nonnil, no returns, `AllowedWhenUntainted`, `HasRestrictions`; the namespace is `SecureOnly`. Its callback type has no arguments. No exact current declaration was found for the other seven names.

## What it must do

### Stored callbacks

- [ ] Retain independent button, pending-ping and toggle-run slots per environment; namespace and legacy ping setters address the same slot.
- [ ] Preserve exact closure identity and captured values through replacement and full GC; getters return the current closure, not a manufactured callback.
- [ ] INFERRED legacy contract: setters return zero values, getters return one value including nil on an empty slot, and nil clears a legacy slot.
- [ ] Namespace ping setter returns zero values; namespace clear returns zero values and affects only the ping slot. INFERRED clear idempotence.
- [ ] Storage/access operations do not run callbacks or change caller taint. Retrieved closures retain their existing object taint when invoked by an explicit consumer.
- [ ] When a real frame event consumer retrieves and invokes callbacks, replacement/clear affects subsequent reads, nested FireEvent sees live replacement, and failed callback invocation does not mutate storage. Consumer-defined call order, payload and pcall error handling are not provider/native dispatch guarantees.

### Namespace argument authentication

- [ ] Under `retail-12-0-7`, authenticate callback and every extra argument with `rilua::table_security::unwrap_secret` before callback type validation or any storage mutation.
- [ ] Accept authentic secret-wrapped functions from an untainted caller and store their underlying closure identity; deny secret arguments from a tainted caller, including an extra argument paired with malformed public arg1.
- [ ] INFERRED representation policy: accept only functions; reject missing/nil, numbers, strings, tables and frames atomically. Public callbacks remain usable by tainted callers; this does not enforce the later declaration's SecureOnly/HasRestrictions domain.
- [ ] Preserve original secret wrappers through GC, denial and public recovery. INFERRED extra-argument policy: authenticate then ignore extras, do not silently accept inaccessible secrets.

## How it works

- [Event dispatch](../event-system.md)
- [Lua API](../lua-api.md)

## Implementation inventory

- `src/c_api/c_ping_secure.rs` — existing shared callback table/accessors; staged namespace-only authenticated setter. No new SimState field.
- `src/lua_api/env_events.rs` — unchanged real dispatcher used by test-owned frame consumers.
- `src/lua_api/globals/real/event_callbacks.rs` — unchanged ordinary global event callback subsystem; not the provider of these eight APIs.

## Tests asserting this spec

`tests/p1207_pending_callbacks.rs`: twelve grouped tests in the auto-included integration target, gated by `retail-12-0-7`. They use the real `env.fire_event_with_args` and `FireEvent`, without replacing any API. Three namespace validation/authentication tests are predicted RED against current master. Remaining nine are predicted passing by source reading, not execution.

## Known gaps (current cycle)

- [ ] Staged Rust formatted successfully; no compile or test execution yet; authoring task forbids cargo/tests and repo writes. All checkboxes remain unchecked.
- [ ] Historical declarations for seven APIs, native secure/gamepad producer, restriction enforcement and native error/extra-argument policy are unproven.
- [ ] Existing callback state is stored in an addon-visible global table; hostile state tampering/recovery is not covered or redesigned in this slice.

## Out of scope

- Native pending-action creation, consumption, ordering, payloads, cancellation and protected execution: no backing producer or historical/native policy evidence.
- Ordinary RegisterEventCallback/UnregisterEventCallback semantics: earlier APIs, not these source rows. Consumer pcall does not establish provider error isolation.
- New secret policies on undocumented legacy globals or on namespace clear; only the declared namespace setter is changed.
- Older-profile behavior changes: new authentication is gated to `retail-12-0-7`, while existing earlier namespace publication stays unchanged.
