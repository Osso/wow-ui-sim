# Cooldown paused state

Pinned `FrameAPICooldownDocumentation.lua` declares `SetPaused(bool)` with no returns.

## Contract

- Retail 12.0.0 and later: explicit true/false updates existing cooldown paused state observed through `IsPaused()`.
- Setter returns zero values and preserves other instances and immediate cooldown timing values.
- Shared `SetPaused` dispatch must preserve existing ModelScene paused behavior.
- Existing boolean parsing policy is retained; native coercion and errors are unverified.

## Evidence

Tests through `34f3cf876` reproduce the shared-dispatch boundary: cooldown state and isolation fail before the correction, while zero return arity and immediate timing preservation are separately observed. They also control that `ModelScene:SetPaused` continues to update `GetPaused()` independently. Runtime `218ba9977` routes Cooldown receivers through the existing `cooldown_paused` state while retaining ModelScene state handling. Independent proof at `34f3cf876` passes 5/5 each on retail 12.0.0/12.0.5/12.0.7; fmt/check/build/startup (`[]`)/readability passed. Final metadata verification at `37caff416` reuses exact bytes, validates all 3,410 rows, and records 14,903 fresh hashes, zero stale, 15 renewals, and six additions. One bounded credit; totals **2313 / 1095 / 2**.

## Gaps

Elapsed-time freezing, rebasing, resume lifecycle, events, rendering, native validation and earlier-profile behavior remain unproven. Secret/taint enforcement is deferred. State tests do not establish clock behavior.
