# Forever unit speed

Cached Forever `Blizzard_APIDocumentationGenerated/UnitDocumentation.lua:335–351` requires a unit token and four numeric results: current, run, flight, swim. Camelot `PaperDollFrame.lua:1410` consumes the global. This specification models only the player's speed; native Forever probes are unavailable.

## Behavior

- [x] Register `GetUnitSpeed(unit)` only for Forever; other profiles retain their existing surfaces.
- [x] Resolve unit identity through existing unit snapshots: `player`, `self`, and aliases actually resolving to the player GUID read player speed. A different player's target, pet, party member, or unknown token never inherits the player's speed.
- [x] Return four numbers. Player capabilities default to run `7`, flight `7`, swim `4.722222` yards/s; these are explicit simulator assumptions inherited from existing Mists values, **not native-verified defaults**. Configured player capabilities replace them.
- [x] Current speed is zero while stationary; otherwise swimming takes precedence over flying, then running. Existing movement verbs and admin setters control those flags.
- [x] Policy for resolved nonplayer and unknown tokens: return `0, 0, 0, 0`, because no nonplayer movement-speed model exists. This policy is a simulator assumption, not documented native behavior. Missing or non-string unit arguments fail existing string argument conversion.
- [x] Post-cleanup global restoration retains the registered query and modeled state.

## Proof and limits

`tests/wowforever_unit_interactions.rs` contains focused Lua behavior tests in the grouped `integration` target. The cached character-panel failure is reproduced independently by the main session; this slice does not claim full panel replay, native semantics, persistence, or other-profile runtime proof.
