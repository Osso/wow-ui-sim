# Forever mapped-stick queries

Forever exposes the two `C_GamePad` queries consumed by cached `Blizzard_GamepadSharedUtility/InputBindingStack/InputAxisBinding.lua`. The backing model lives in `src/c_api/c_game_pad.rs`; [system notes](../wiki/systems/gamepad-mapped-state.md) describe its bounded implementation. This is a **source-observed subset**, not a native mapped-state DTO or hardware model.

## What it must do

- [x] Forever direct calls to `GetDeviceMappedState()` and `StickIndexToConfigName(index)` expose the modeled queries.
- [ ] Publication is Forever-only (source-gated; no executed other-profile absence proof). The actual cached initializer calls the getter eagerly twice, without arguments, and passes ordinary zero-based numbers to the selector.
- [x] No configured snapshot returns nil. Defaulting to no configured snapshot is simulator policy, not observed hardware detection.
- [ ] Actual cached initialization marks both listener sides centered.
- [x] A configured ordered stick list publishes coherent `stickCount`, one-based `sticks`, and per-stick `len`. Selector index zero names the first configured stick. The actual cached initializer compares names with `Movement`/`Camera` and checks `len == 0`.
- [ ] Concrete nonzero Movement/zero Camera initializes left uncentered/right centered. Reordered Camera/Movement inputs initialize the opposite sides from names rather than fixed positions.
- [x] **Inferred simulator policy:** finite coordinate fixtures produce Euclidean length; Rust x/y are input fields, not claims about native Lua fields. Each getter returns independent tables; replacement, empty snapshots and clearing are visible on subsequent queries.
- [x] Each environment owns its optional snapshot independently of logical UI input style and frame gamepad flags.
- [x] **Inferred simulator policy:** absent/out-of-range/negative/fractional numeric selectors return nil; string and opaque secret selectors error, with opaque input remaining secret.
- [ ] Other nonnumeric selectors and payload-free error reporting lack observable assertions. No native coercion or secret contract is claimed.
- [x] Ordinary addon-tainted calls preserve caller taint; opaque selector input remains secret after rejection.

Checked bullets denote bounded observable direct-model proof only, not overall acceptance or native parity. Current evidence is `/home/osso/.local/state/wow-ui-sim-proof/forever-auto-roll-2026-10-01/model-v2-ledger.json` with `model-v2.stdout` / `model-v2.stderr`: build `5e15752` reconciled to run `74e6c8845`, rilua `6044544b`, 70 cases / 63 pass / 7 fail (exit 101). Three direct mapped-model cases pass; three cached initializer and four AutoRoll cases fail before local addon/decision assertions on `C_GamePad.SetAllowHoverEventsWithFreeLook`, `FrameControlsManager.lua:830`. This supersedes the earlier namespace-gap boundary at `1fbea8e70`; neither failure establishes AutoRoll decision behavior.

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

- [ ] Cached initializer GREEN and AutoRoll workflow proof remain blocked at the hover-policy dependency.
- [ ] Independent audit agent `20366` remains pending; overall acceptance is open.

Default fmt/check passed in `/tmp/patch-12.0.5-batch11-rust-gates.json` with unchanged recorded source/config inputs; no rerun by this documentation audit. New hover-policy tests have implementation pending and no outcome is inferred. Native hover/controller and Reveal behavior remain unmodeled.

## Out of scope

Full native fields, optional getter arguments, selector coercion/secrecy parity, physical devices, enumeration/configuration/vibration, additional `C_GamePad` methods, host events, Admin producers, Retail, Reveal modeling and AutoRoll workflow integration. Source only establishes the consumed fields and call forms; native probes are unavailable. No vendor edits, no-op masks, fallback paths or diagnostic-classification patches.
