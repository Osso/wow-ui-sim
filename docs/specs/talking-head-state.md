# Retail TalkingHead state

Retail exposes `C_TalkingHead` state to the Blizzard talking-head and splash-screen consumers. The simulator uses explicit empty-default host input because native return values remain unknown (R88bf pending). See [Lua API](../lua-api.md).

## What it must do

- [ ] Publish `GetConversationsDeferred`, `GetCurrentLineAnimationInfo`, `GetCurrentLineInfo`, `IgnoreCurrentTalkingHead`, `IsCurrentTalkingHeadIgnored`, and `SetConversationsDeferred` on Retail only.
- [ ] With no host line, both current-line queries return no values. With a line, return ten line fields and four animation fields in the order used by `Blizzard_FrameXML/TalkingHeadUI.lua:143,275`.
- [ ] Read and write the conversation-deferred and ignored flags from empty-default host state.
- [ ] Original `Blizzard_FrameXML` and `Blizzard_SplashFrame` consumers run against the modeled state without errors.
- **INFERRED (simulator mapping; native returns UNKNOWN, R88bf pending):** no current line means no current-line values; the host supplies all line and animation fields; ignore marks the current line; deferred and ignored queries return their stored Boolean inputs.

## How it works

- [Lua API](../lua-api.md) — C API boundary and modeled state.

## Implementation inventory

- `src/c_api/c_talking_head.rs` — Retail registration, host state and Lua return projection.
- `src/c_api/registration.rs` — Retail registration wiring.
- `src/lua_api/state/sim_state.rs` — environment-local talking-head input.

## Tests asserting this spec

- `tests/c_talking_head.rs` — API field projection and original Blizzard consumer behavior.

## Known gaps (current cycle)

- [ ] Native return values, state producers, and lifecycle semantics remain UNKNOWN pending R88bf evidence.

## Out of scope

- Network/client talking-head production and 3D model rendering; the host input is explicit and empty by default.
