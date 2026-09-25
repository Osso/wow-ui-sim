# Retail aura-duration secret values

Retail 12.1+ exposes variadic `secretwrap`/`secretunwrap` to unchanged Blizzard AuraButton Lua; the simulator bridges these calls to rilua's existing secret-value representation. This is bounded simulator policy inferred from the observed consumer, not native-verified general secret semantics.

## What it must do

- [ ] Preserve the exact argument count and position, including nil and zero arguments, while wrapping or unwrapping values. Rewrapping an existing native secret preserves its identity.
- [ ] Let unchanged `AuraButtonPrivateMixin:UpdateAuraDuration` configure its existing duration object using all three wrapped expiration, duration, and rate arguments, or the wrapped zero-span branch.
- [ ] Mark native wrappers secret in `issecretvalue` and inaccessible in `canaccessvalue`/`canaccessallvalues`, without replacing existing taint-marker and closure checks.
- [ ] Reject tainted wrapping and unwrapping of secrets; secret-duration timing and mutation remain inaccessible to tainted callers.
- [ ] Preserve Forever's native registration and older profiles' existing fallbacks; do not publish `settablesecurity` on retail.

## How it works

- [Secret-origin aura display](aura-secret-display.md)
- [Duration core](duration-core.md)

## Implementation inventory

- `src/lua_api/globals/security/secret_values.rs`: variadic native-wrapper bridge and secret-value detection.
- `src/lua_api/globals/security/mod.rs`: scoped registration export.
- `src/lua_api/env_init/mod.rs`: retail registration before temporary defaults and secure-environment snapshot.
- `src/lua_api/workarounds/temporary/debug_environment_defaults.rs`: guarded older-profile identity fallback.

## Tests asserting this spec

- `tests/duration_core.rs`: unchanged cached Blizzard AuraButton duration consumer, wrapper arity/nil, security and duration-state checks (GREEN pending).

## Known gaps (current cycle)

- [ ] Targeted compiled regression and cross-profile gates pending integration.

## Out of scope

- General VM secret arithmetic, `settablesecurity`, scrub behavior, native-conformance claims, and vendor changes.
