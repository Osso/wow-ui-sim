# Cooldown paused state

Pinned `FrameAPICooldownDocumentation.lua` declares `SetPaused(bool)` with no returns.

## Contract

- Retail 12.0.0 and later: explicit true/false updates existing cooldown paused state observed through `IsPaused()`.
- Setter returns zero values and preserves other instances and immediate cooldown timing values.
- Shared `SetPaused` dispatch must preserve existing ModelScene paused behavior.
- Existing boolean parsing policy is retained; native coercion and errors are unverified.

## Evidence

Focused tests in the existing grouped integration target reached RED 2/4: state and isolation failed while arity and immediate timing preservation passed. The shared method registry resolves `SetPaused` to the ModelScene handler, which previously wrote only model state. Runtime correction dispatches Cooldown receivers to their existing paused field. Post-change proof pending.

## Gaps

Elapsed-time freezing, rebasing, resume lifecycle, events, rendering, native validation and earlier-profile behavior remain unproven. Secret/taint enforcement is deferred. State tests do not establish clock behavior.
