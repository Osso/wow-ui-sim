# Addon restriction state

Retail and PTR expose `C_RestrictedActions` state queries backed by six independent simulator restriction states. The model is in `src/c_api/restricted_actions.rs`.

## What it must do

- [x] A new environment starts with types 0–5 inactive. The getter returns the state and the predicate is true exactly when that state is Active (2).
- [x] Activation sets Activating (1), synchronously emits `ADDON_RESTRICTION_STATE_CHANGED(type, 1)`, then sets Active (2). The predicate for the changing type is false inside the event.
- [x] Deactivation sets Inactive (0) before synchronously emitting `ADDON_RESTRICTION_STATE_CHANGED(type, 0)`.
- [x] Combat follows `A_Admin.SetInCombat`; the other types are independently supplied with `A_Admin.SetAddOnRestrictionActive(type, active)`.
- [x] Encounter and PvP lifecycle events and challenge-run start/clear drive their respective types. These drivers are best guesses, not verified native triggers.
- [x] Missing, nil, negative, out-of-range and nonnumeric type arguments fail. This validation is a best guess.
- [ ] Challenge-run completion deactivates the challenge restriction. This path is implemented but not separately tested here.

## Best-guess basis

**Best-guess, native UNKNOWN:** the initial inactive state, input validation, simulator drivers, exact activation-state storage and event ordering beyond the documented before-active/after-inactive ordering. The generated Retail and PTR `Blizzard_APIDocumentationGenerated/RestrictedActionsDocumentation.lua:29–41,55–67,91–101` declare the queries and synchronous event. `RestrictedActionsConstantsDocumentation.lua:6–32` declares the state and type values. The 12.1.5 native evidence verifies that the getter reads a per-index byte and the predicate compares it with 2; it does not establish writers or triggers. The simulator uses the documented enum values as indices.

## How it works

- [Event system](../event-system.md)

## Implementation inventory

- `src/c_api/restricted_actions.rs`: state queries, transitions and supplied input.
- `src/lua_api/state/sim_state.rs`: per-environment state storage.
- `src/lua_api/globals/admin.rs`: combat and supplied-state inputs.
- `src/c_api/challenge_run/inputs.rs`: challenge-run transitions.
- `src/lua_api/globals/state_backed_queries.rs`, `src/lua_api/env_events.rs`, `src/lua_api/loader_env.rs`: host lifecycle event inputs.
- `src/lua_api/workarounds/temporary/restricted_actions_protected.rs`: unchanged permissive protected-function permission placeholder.

## Tests asserting this spec

- `tests/restricted_actions_state.rs`: combat, supplied states, lifecycle inputs and invalid arguments.

## Known gaps (current cycle)

- [ ] Native writers, trigger conditions and validation details remain unknown.
- [ ] The documentation says the predicate is always false during event dispatch; behavior when a *different* restriction is already active is not established by these tests. The implementation compares the queried state to 2.
- [ ] Native relationships between `SecretWhen*` categories and restriction types are unknown; this model does not wire them together.
- [ ] Protected-function permission enforcement remains outside this change.

## Out of scope

Secret-category mappings and changes to `CheckAllowProtectedFunctions` are excluded from this lane.
