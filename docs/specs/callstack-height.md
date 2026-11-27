# Callstack height

`GetCallstackHeight` reports the active rilua call-frame index on Retail and WoW Forever. Its mapping to native call-stack height is **INFERRED R88g9 (native evidence pending)**.

## What it must do

- [x] Report a greater value inside a nested Lua call than at the shallower call site.

## How it works

- [Lua API system](../wiki/systems/lua-api.md)

## Implementation inventory

- `src/lua_api/globals/real/debug_environment.rs` — Retail and Forever call-frame query.
- `src/lua_api/workarounds/temporary/debug_environment_defaults.rs` — legacy-profile fallback.

## Tests asserting this spec

- `tests/callstack_height.rs`

## Known gaps (current cycle)

- [ ] Confirm whether native results count Lua frames, C frames, or another stack boundary (R88g9).

## Out of scope

- Error call-stack height configuration and formatting; no model for that policy is available.
