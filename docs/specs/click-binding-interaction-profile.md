# Click-binding interaction profile

`C_ClickBindings` exposes per-environment interaction bindings for secure unit clicks. The initial unmodified LeftButton target / RightButton context-menu entries are **inferred simulator policy**, not native-verified defaults.

## What it must do

- [x] Report interaction bindings for unmodified left and right clicks; report no binding for an unassigned button/modifier pair.
- [x] Return `ClickBindingInfo` entries with numeric `type`, numeric `actionID`, string `button`, and numeric `modifiers`, without exposing mutable stored entries.
- [x] Replace the active profile through `SetProfileByInfo`, use it for binding/effective-button queries, and restore defaults through `ResetCurrentProfile`.
- [x] Let unchanged Blizzard `SecureUnitButton_OnClick` target player and party units through the default profile.
- [x] Keep unmodeled `ExecuteBinding` inert without preventing interaction clicks.

## How it works

- See [Lua API](../lua-api.md) for environment initialization and C API boundaries.

## Implementation inventory

- `src/c_api/c_click_bindings.rs` — per-environment interaction profile and queries.
- `src/lua_api/env_init/mod.rs` — registers the modeled profile.
- `src/lua_api/workarounds/temporary/click_bindings_defaults.rs` — unrelated temporary click-binding gaps and modifier helpers.

## Tests asserting this spec

- `tests/click_targeting.rs` — profile round-trip, secure vendor handler, inert execution.

## Known gaps (current cycle)

- [ ] Modifier assignments beyond the unmodified default are simulator-defined; native modifier semantics are unverified.

## Out of scope

- Spell, macro, and pet-action execution; full click-casting UI; persistence and native default-profile conformance.
