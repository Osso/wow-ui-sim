# Spectating UI scenario state

This simulator scenario supplies the booleans read by `C_SpectatingUI.IsSpectating` and `C_Commentator.IsSpectating`. The `C_Commentator` state mapping is INFERRED (R88ek; native evidence pending); the native defaults are unknown. Generated Retail and PTR API documentation has no SpectatingUI entry.

## What it must do

- [ ] Publish `IsSpectating()` on Retail, PTR, and WowForever, whose pinned UI calls it.
- [x] Return the environment-local supplied boolean from `C_SpectatingUI.IsSpectating`. The initial `false` is a user-authorized simulator scenario default, not a native observation.
- [x] Publish `C_Commentator.IsSpectating` on Retail and WowForever from the supplied boolean; verify with the original `TimerTracker` event handler.
- [x] The original Retail game-menu Esc handler consumes both supplied values.

## How it works

- [Spectating UI scenario](../wiki/systems/spectating-ui.md)

## Implementation inventory

- `src/c_api/c_spectating_ui.rs`: `C_SpectatingUI` query and profile publication.
- `src/c_api/c_commentator.rs`: `C_Commentator.IsSpectating` query and `A_Admin.SetSpectating` input.
- `src/lua_api/state/sim_state.rs`: scenario fixture input.
- `src/lua_api/state.rs`: scenario default.

## Tests asserting this spec

`tests/chat_enter_binding.rs` exercises the original game-menu handler with an explicit Enter binding. `tests/commentator_spectating.rs` checks the original timer frame's parent for both supplied states.

## Known gaps (current cycle)

- [x] WowForever, Mists, Wrath, and Retail `--lib --test integration --no-run` builds pass with zero warnings.

## Out of scope

Spectating lifecycle, other `C_SpectatingUI` members, and native default or policy are not modeled.
