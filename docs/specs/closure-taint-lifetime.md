# Closure taint lifetime

Addon and loadstring taint belongs to one closure allocation, not a reusable arena index. The simulator uses rilua's shared host stamp/query API; see [taint architecture](../lua-api.md).

## What it must do

- [ ] Preserve a reachable function's explicit taint across collection and secure calls.
- [ ] Permit an unreachable stamped function to be collected; never transfer its stamp to a new function reusing that arena slot.
- [ ] Use the same identity for addon-load stamping, loadstring stamping, VM call entry and secret-function classification.
- [ ] Report attempts to stamp a collected closure rather than silently ignoring them.

## How it works

- [Lua API](../lua-api.md)
- [Attribute dispatch boundary](secure-attribute-delegation.md)

## Implementation inventory

- Rilua `stdlib::taint` — weak, generation-aware closure-key storage and host APIs.
- `src/lua_api/taint.rs` — host stamp bridge.
- `src/loader/lua_file.rs`, `src/lua_api/env_init/mod.rs` — addon/loadstring stamping and error propagation.
- `src/lua_api/globals/security/secret_values.rs` — shared stamp lookup for legacy secret loadstring markers.

## Tests asserting this spec

- Rilua `tests/helpers/closure_taint.rs` — GC reuse, live stamp retention, host setter/getter, stale-handle rejection.
- `tests/security_api.rs` — live addon/loadstring secrecy and secure-call controls.

## Known gaps (current cycle)

- [ ] Independent dependency verification and current simulator replay pending. Standalone pre-fix Lua reproduction assigns a collected function's `CollectedClosureProbe` taint to a new clean closure.

## Out of scope

Changing secure-call policy, clearing legitimate live closure taint, or altering addon/vendor code.
