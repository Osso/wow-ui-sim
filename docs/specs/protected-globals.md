# Protected global calls

The simulator applies a best-guess protection policy to registered globals and C namespace functions. The policy is implemented in `src/lua_api/globals/real/protected_globals.rs`.

## Best-guess basis

Native evidence establishes the diagnostic events and that a refused call returns without raising. It does not establish the protected function set. The membership list uses the Warcraft Wiki `World_of_Warcraft_API` API tags captured on 2026-10-31. Protection membership and combat policy remain best guesses, not native-verified facts.

## What it must do

- [x] An insecure `TargetUnit` call returns no values, leaves the target unchanged, and emits `ADDON_ACTION_FORBIDDEN` with the addon origin and `TargetUnit()`.
- [x] The Blizzard `GameEvent` consumer shows `ADDON_ACTION_FORBIDDEN` for that event.
- [x] A secure `TargetUnit` call executes normally.
- [x] Insecure direct and callback-mediated `securecall` invocations cannot launder protected calls.
- [x] Insecure `SetBinding` executes outside combat, but in combat returns no values, leaves the binding unchanged, and emits `ADDON_ACTION_BLOCKED` with origin and `SetBinding()`.
- [ ] Apply the same policy to the other registered names in the community list, without publishing missing functions.

## How it works

- See [Lua API](../wiki/systems/lua-api.md).

## Implementation inventory

- `src/lua_api/globals/real/protected_globals.rs`: policy names and call protection.
- `src/lua_api/env_init/mod.rs`: installs the policy during environment initialization.
- `src/lua_api/globals/real/mod.rs`: registers the module.

## Tests asserting this spec

- `tests/protected_globals.rs`: loader-tainted addon, original Blizzard popup consumer, secure calls and combat binding behavior on Retail and PTR.

## Known gaps (current cycle)

- [ ] Exact native protection membership and event payload text remain unknown.
- [ ] Protection lists can differ between client builds and profiles; the community list is not native proof for each build.
- [ ] Test the remaining registered names and profile-specific exposure.

## Out of scope

- Hardware-event restrictions: the simulator has no hardware-event model for this policy.
