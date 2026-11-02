# Retail secure ping publication

Retail 12.1.5 documents `C_PingSecure` as `SecureOnly` in `Blizzard_APIDocumentationGenerated/PingManagerSecureDocumentation.lua:3–6`. The pinned `Blizzard_PingUI/Blizzard_PingUI.toc` loads PingUI into the secure environment. Build 70688 WM38 registration evidence supports a separate secure namespace, not an ordinary public-global installation.

The simulator publishes the existing `C_PingSecure` namespace in the secure environment and removes its public-global binding for Retail 12.1.5. Earlier Retail epochs and other profiles are unchanged. This publication contract does not by itself claim that every member's native behavior is modeled. Other client code could install an alias; such aliases are not modeled here.

## Retail 12.1.5 restrictions and modeled members

Every modeled member checks the caller's taint provenance first. A tainted caller is refused before any argument is read: the member returns no values and the shared reason-0 reporter fires `ADDON_ACTION_FORBIDDEN(source, functionName)`. `functionName` is the live Lua debug name of the call plus `()`: the member name for dotted access, the alias name for local, global or upvalue aliases, and `UNKNOWN()` when no name exists, such as a call through `pcall`. It is cut to 1023 bytes, so a long name can lose its suffix. A refusal inside a running report is not reported again.

The eight `Set*Callback` members replace their slot with a required callback; the secret-aware argument is unwrapped first. `ClearPendingPingOffScreenCallback` is not declared for Retail 12.1.5 and is not published there.

`CreateFrame()` returns no values. The first secure call creates one unnamed native `Frame` without a parent, with its `CENTER` anchored to `WorldFrame`'s `BOTTOMLEFT` at zero offsets, `MEDIUM` strata and frame level 0. It stores the frame and synchronously calls the installed radial-wheel-created callback with that frame only. Later calls change nothing and do not notify again, so a callback installed after creation is never told about the frame. Errors raised by the callback go to the error handler instead of the caller.

`DisplayError(error)` requires a string or number; a number is converted with Lua number formatting. Empty text does nothing. Other text fires `PING_SYSTEM_ERROR(text)`, which Blizzard routes to `UIErrorsFrame` (`EventRouting.lua:73`, `EventImplementation.lua:167–168`). It returns no values.

## Sends and pending hit-test information

The five send members return a table with a numeric `result` (`Enum.PingResult`) and, only when a type is selected, a numeric `type` (`Enum.PingSubjectType`). An optional type argument is absent when nil or missing. Other values convert to an integer (0 when conversion fails) and narrow to one byte, so 256 becomes 0; a byte above 9 is an argument error.

Every send starts with the shared eligibility check. The clock is the cached millisecond clock that `GetTime` reads. The burst state holds a count, a limit, a window, the window start, the last submission time and an optional cooldown deadline:

- While a cooldown is active, a send is refused with `FailedSpamming` (2) until the clock reaches the deadline. At or after the deadline the cooldown ends, and that check does no idle reset.
- Without a cooldown, 3000 ms or more since the last submission resets the count to 0, the limit to 2 and the window to 3000 ms, and starts the window now.
- A submission first resets the count to 0 when the count is at least the limit and the window has elapsed since its start. It then increments the count and records the submission time. A count above the limit starts a cooldown with deadline now + 3000 ms. The crossing send itself succeeds.

`SendUnitPing(target [, type, isPlayerResource])` requires a GUID string. After eligibility, an empty GUID gives `FailedUnspecified` (7), and a GUID that resolves to no simulator unit gives `FailedOutOfPingArea` (5). The local player's GUID succeeds. Without a type argument it selects 4 when the target can attack the player with reaction at most 1, and 5 otherwise; for the player this is 5.

`SendHitTestPing([type])` takes the pending target. A pending GUID is sent as a unit ping. Otherwise, if any position component has an absolute value of at least 2⁻²² (equality qualifies), the point sender runs its own eligibility check. With neither, the result is `FailedGeneric` (1). The pending information is cleared in every case, including a refused send. `ClearHitTestPingInfo()` clears it and returns no values. The simulator has no world hit selection, so hosts supply the pending target in `SimState.ping_pending`.

`SendPlayerItemPing(itemID)`, `SendPlayerSpellPing(spellID)` and `SendPlayerSpellCategoryPing(spellCategoryID)` convert their ID from a number or numeric string, truncating toward zero within the signed 32-bit range. After eligibility, ID 0 gives `FailedUnspecified` (7). A spell category other than 4, 30 or 1711 gives `FailedGeneric` (1).

Basis: Retail 12.1.5 build 70688 WM38, WM38F restriction reporting, WM38F-B CreateFrame and WM47 Q1–Q10. **INFERRED:**

- Only functions convert as setter callbacks; missing, nil and other values are argument errors.
- The burst state starts with the idle-reset values.
- `SendUnitPing` runs eligibility before the empty-GUID check.
- A type byte above 9 and a non-string GUID are argument errors.
- `SendUnitPing`'s contextual type uses the simulator's `UnitCanAttack` and `UnitReaction` models.

**Known gaps:**

- The macro-origin `MACRO_ACTION_FORBIDDEN(functionName)` variant (WM47 Q11 UNKNOWN) and the method-call name prefix.
- The radial frame's size (WM51-F6) and the callback error copy and rethrow path.
- The cooldown-started callback (WM51-F1) and the eligibility results 4, 3 and 6 (WM51-F2).
- GUID text parsing and non-self unit flags and redirection (WM51-F3). These raise a "not modeled" error.
- The point sender's distance check after eligibility (WM51-F7), which raises a "not modeled" error.
- Past the zero-ID check, these raise a "not modeled" error: item toy ownership and use predicates (WM47 Q5), the spell effect-aura predicate and spell metadata (WM47 Q3, Q6), and accepted spell categories, which need the category cooldown query (WM47 Q3).
- `GetTargetPingReceiver` (WM51-F5), `SetHitTestPingTarget` and `SetHitTestTargetAndSendPing` (WM51-F4) are not published.
