# Cooldown paused state

Pinned `FrameAPICooldownDocumentation.lua` declares `SetPaused(bool)` with no returns.

## Contract

- Retail 12.0.0 and later: explicit true/false updates existing cooldown paused state observed through `IsPaused()`.
- Setter returns zero values and preserves other instances and immediate cooldown timing values.
- Shared `SetPaused` dispatch must preserve existing ModelScene paused behavior.
- Existing boolean parsing policy is retained; native coercion and errors are unverified.

## Evidence

Tests through `34f3cf876` reproduce the shared-dispatch boundary: cooldown state and isolation fail before the correction, while zero return arity and immediate timing preservation are separately observed. They also control that `ModelScene:SetPaused` continues to update `GetPaused()` independently. Runtime `218ba9977` routes Cooldown receivers through the existing `cooldown_paused` state while retaining ModelScene state handling. Post-change proof is pending; this is not audit credit.

## Gaps

Elapsed-time freezing, rebasing, resume lifecycle, events, rendering, native validation and earlier-profile behavior remain unproven. Secret/taint enforcement is deferred. State tests do not establish clock behavior.
