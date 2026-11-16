# Mists R88ah C APIs

This spec covers Mists-only `C_CombatAudioAlert.IsEnabled` and `C_Spell.GetVisibilityInfo` in `src/c_api/mists_r88ah.rs`. The native answer is `~/.local/state/wow-ui-sim/handoff/pr46-49-decomp-answers.md`, section `R88ah`.

## What it must do

- [ ] **VERIFIED (R88ah, Mists 5.5.4.70631):** `IsEnabled()` returns exactly one boolean. It reads `CAAEnabled`, whose registered default is `0`, and returns false when the internal alert-state gate is off.
- [ ] **VERIFIED (R88ah, Mists 5.5.4.70631):** `GetVisibilityInfo(spellID, visibilityType)` requires two non-nil arguments, accepts number-like strings, truncates fractions toward zero, and reduces the second value modulo 256 before accepting only values `0`, `1`, and `2`.
- [ ] **VERIFIED (R88ah, Mists 5.5.4.70631):** with an active player, `GetVisibilityInfo` returns three booleans; a missing spell/type record returns `false, false, false`. Without an active player it returns zero values.
- [ ] **VERIFIED (R88ah, Mists 5.5.4.70631):** usage, range, integer-overflow, argument, and secret-access failures use the answer's exact messages.
- [ ] **INFERRED (70631 evidence applied to the 70268 Mists cache):** publish these APIs for the Mists profile because the cached declarations and visibility enum values match. The 70268 native behavior was not verified.

## How it works

- [C API boundary and Lua API](../lua-api.md)
- [Client-profile selection](../wiki/systems/client-profiles.md)

## Implementation inventory

- `src/c_api/mists_r88ah.rs` — Mists API registration and behavior.
- `src/lua_api/state/sim_state.rs` — alert-gate input with a neutral enabled default.
- `src/lua_api/state.rs` — initializes the alert-gate input.

## Tests asserting this spec

- `tests/mists_r88ah.rs` — original Mists consumers, CVar results, visibility rows, absent player/spell, conversions, and argument errors.

## Known gaps (current cycle)

- [ ] **UNKNOWN (R88ah):** how audio-manager failures change the internal alert-state gate. The simulator keeps the gate on by default and does not assert failure mapping.
- [ ] **INFERRED:** native behavior on Mists 5.5.4.70268; evidence covers 5.5.4.70631.

## Out of scope

- Reconstructing visibility rows or their symbolic field names. This API reads supplied visibility results.
- Mapping audio-device or audio-manager failures to the internal gate.
