# Class-trial character state

The simulator models the `C_ClassTrial` character flag and logout deadline from host-supplied state. The deadline contract is INFERRED from the Retail/Forever timer consumer; native evidence is pending R88en.

## What it must do

### API surface
- [ ] INFERRED (R88en pending): `C_ClassTrial.IsClassTrialCharacter()` returns the supplied class-trial flag.
- [ ] INFERRED (R88en pending): `C_ClassTrial.GetClassTrialLogoutTimeSeconds()` returns the supplied deadline in the `GetTime()` clock domain.
- [ ] `A_Admin.SetClassTrialState(isTrialCharacter, logoutDeadline)` supplies both API inputs for a simulation.

### Timer consumer
- [ ] INFERRED (R88en pending): Retail's loaded `ClassTrialTimerDisplay` stays hidden when its supplied deadline is zero, shows formatted remaining time for a positive deadline, and updates as the simulation clock advances.

## How it works

- [C API boundary](../lua-api.md)
- [Manual simulation clock](manual-simulation-clock.md)

## Implementation inventory

- `src/c_api/c_class_trial.rs` — class-trial API functions and host setter.
- `src/lua_api/state/sim_state.rs` — per-environment class-trial input.
- `src/lua_api/globals/missing_surface.rs` — C API namespace registration.
- `src/lua_api/globals/admin.rs` — A_Admin setter registration.
- `src/lua_api/workarounds/temporary/class_trial_defaults.rs` — unchanged fallback for profiles without the modeled surface.

## Tests asserting this spec

- `tests/c_class_trial.rs`
- `tests/blizzard_class_trial_loads.rs`

## Known gaps (current cycle)

- [ ] Confirm the native return semantics and deadline clock through R88en.
- [ ] Class-trial timer UI is not runtime-selected on WoW Forever; only its C API state surface is tested there.

## Out of scope

- Other class-trial dialog, store, and boost behavior. Their backing state is not part of this timer model.
- Changing fallback registrations on PTR or other profiles.
