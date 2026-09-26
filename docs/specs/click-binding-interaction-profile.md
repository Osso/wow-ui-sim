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

## Tested scope

- Commit `c00f44d0e`: profile queries, replacement, copied results, effective-button mapping, and reset: 2/2; unchanged Blizzard `SecureUnitButton_OnClick` targets `player` and `party1`: 1/1.
- Commit `dfcacefbe` replaces the handpicked fixture startup list with production Blizzard Game-screen discovery/loading. Independent inspection of `/tmp/wow-unit-frame-bug/click-production-fixture.log` records `blizzard_full_ui_click_chain_targets_and_casts` PASS 1/1: PlayerFrame and PartyFrame target their units, and spell/action paths cast Flash of Light. The PlayerFrame assertion has no direct `TargetUnit` fallback.
- The owned prior-release probe `/tmp/wow-unit-frame-bug/aura-global/probe.txt` records PlayerFrame `target` and `menu` attributes. Those attributes are supporting configuration evidence, not separate GUI or native proof.
- Final bounded local Forever GUI evidence is recorded in [[final-unit-frame-click-aura-proof]]: clicking PlayerFrame selected Uther with the matching player GUID and zero hook/final errors. This is simulator-local GUI evidence, not native proof or broad hit-testing coverage.
- Retail `/tmp/retail-regression/native-input-app.json` uses application debug-key and internal hit-tested-mouse endpoints, not native physical input. It opens/closes/reopens Character, opens/closes Social and Talents, opens Talents through the PlayerSpells microbutton, and shows Head/Shoulder tooltips. `mainhand-input-probe.json` leaves MainHand blocked by hit `#68088`; diagnosis remains pending. Neither GUI log reports Lua errors.

## Known gaps (current cycle)

- [ ] Modifier assignments beyond the unmodified default are simulator-defined; native modifier semantics are unverified.
- [ ] Broader GUI hit-testing coverage remains unverified: current retail app-endpoint evidence is not native physical input, and MainHand tooltip behavior is blocked by `#68088`.
- [ ] The target-aura `OnUpdate` root cause is outside the click-chain contract; see [[target-aura-private-count]].

## Out of scope

- Spell, macro, and pet-action execution; full click-casting UI; persistence and native default-profile conformance.
