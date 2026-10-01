# Forever mapped-stick queries

Forever exposes the two `C_GamePad` queries consumed by cached `Blizzard_GamepadSharedUtility/InputBindingStack/InputAxisBinding.lua`. The backing model lives in `src/c_api/c_game_pad.rs`; [system notes](../wiki/systems/gamepad-mapped-state.md) describe its bounded implementation. This is a **source-observed subset**, not a native mapped-state DTO or hardware model.

## What it must do

- [ ] Register `GetDeviceMappedState()` and `StickIndexToConfigName(index)` only for Forever. The actual cached initializer calls the getter eagerly twice, without arguments, and passes ordinary zero-based numbers to the selector.
- [ ] No configured snapshot returns nil; actual cached initialization consequently marks both listener sides centered. Defaulting to no configured snapshot is simulator policy, not observed hardware detection.
- [ ] A configured ordered stick list publishes coherent `stickCount`, one-based `sticks`, and per-stick `len`. Selector index zero names the first configured stick. The actual cached initializer compares names with `Movement`/`Camera` and checks `len == 0`.
- [ ] Concrete nonzero Movement/zero Camera initializes left uncentered/right centered. Reordered Camera/Movement inputs initialize the opposite sides from names rather than fixed positions.
- [ ] **Inferred simulator policy:** finite coordinate fixtures produce Euclidean length; Rust x/y are input fields, not claims about native Lua fields. Each getter returns independent tables; replacement, empty snapshots and clearing are visible on subsequent queries.
- [ ] Each environment owns its optional snapshot independently of logical UI input style and frame gamepad flags.
- [ ] **Inferred simulator policy:** absent/out-of-range/negative/fractional numeric selectors return nil; nonnumeric and opaque secret selectors error without decoding or reporting their payload. No native coercion or secret contract is claimed.
- [ ] Ordinary addon-tainted calls preserve caller taint. Neither query implicitly declassifies values.

All bullets remain unchecked until parent-owned focused GREEN and independent verification. Existing RED is the pre-addon namespace failure at `1fbea8e70`, not an AutoRoll decision failure; the retained proof is `/home/osso/.local/state/wow-ui-sim-proof/forever-auto-roll-2026-10-01/corrected-ledger.json` with sibling stdout/stderr.

## How it works

- [Environment-local input and staged result tables](../wiki/systems/gamepad-mapped-state.md)
- [Separate logical input style](forever-input-interface-style.md)

## Implementation inventory

- `src/c_api/c_game_pad.rs` — public concrete input types and the two state-backed queries.
- `src/c_api/mod.rs`, `src/c_api/registration.rs` — Forever-only publication.
- `src/lua_api/state/sim_state.rs`, `src/lua_api/state.rs` — optional environment field and None default.

## Tests asserting this spec

`tests/gamepad_mapped_state.rs` is autodiscovered inside the existing grouped integration target. Three cases load the actual cached GamepadSharedUtility TOC dependency closure without overrides or callback bypasses, with fixture state installed before initialization. Other cases cover replacements/counts/names/table independence, environment isolation and taint/opaque-selector policy. The exact existing SharedXML `C_Reveal` debug-setup diagnostic is retained and reported; its truthy stub changes setup behavior and is not native absence or an innocent probe.

## Known gaps (current cycle)

- [ ] Parent must compile and run focused tests against the approved rilua pin, then perform independent final verification. No Cargo or verification command was run by this author.

## Out of scope

Full native fields, optional getter arguments, selector coercion/secrecy parity, physical devices, enumeration/configuration/vibration, additional `C_GamePad` methods, host events, Admin producers, Retail, Reveal modeling and AutoRoll workflow integration. Source only establishes the consumed fields and call forms; native probes are unavailable. No vendor edits, no-op masks, fallback paths or diagnostic-classification patches.
