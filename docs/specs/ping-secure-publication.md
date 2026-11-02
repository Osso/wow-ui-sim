# Retail secure ping publication

Retail 12.1.5 documents `C_PingSecure` as `SecureOnly` in `Blizzard_APIDocumentationGenerated/PingManagerSecureDocumentation.lua:3–6`. The pinned `Blizzard_PingUI/Blizzard_PingUI.toc` loads PingUI into the secure environment. Build 70688 WM38 registration evidence supports a separate secure namespace, not an ordinary public-global installation.

The simulator publishes the existing `C_PingSecure` namespace in the secure environment and removes its public-global binding for Retail 12.1.5. Earlier Retail epochs and other profiles are unchanged. This publication contract does not by itself claim that every member's native behavior is modeled. Other client code could install an alias; such aliases are not modeled here.

## Retail 12.1.5 restrictions and modeled members

Every modeled member checks the caller's taint provenance first. A tainted caller is refused before any argument is read: the member returns no values and the shared reason-0 reporter fires `ADDON_ACTION_FORBIDDEN(source, functionName)`. `functionName` is the live Lua debug name of the call plus `()`: the member name for dotted access, the alias name for local, global or upvalue aliases, and `UNKNOWN()` when no name exists, such as a call through `pcall`. It is cut to 1023 bytes, so a long name can lose its suffix. A refusal inside a running report is not reported again.

The eight `Set*Callback` members replace their slot with a required callback; the secret-aware argument is unwrapped first. `ClearPendingPingOffScreenCallback` is not declared for Retail 12.1.5 and is not published there.

`CreateFrame()` returns no values. The first secure call creates one unnamed native `Frame` without a parent, with its `CENTER` anchored to `WorldFrame`'s `BOTTOMLEFT` at zero offsets, `MEDIUM` strata and frame level 0. Its size is 10 × 10 on every screen. It stores the frame and synchronously calls the installed radial-wheel-created callback with that frame only. Later calls change nothing and do not notify again, so a callback installed after creation is never told about the frame. Errors raised by the callback go to the error handler instead of the caller.

The size follows from three findings: WM47 Q10 (as corrected), WM51-F5 and WM51-F6. The native side is (10/1024)/S × M, with S = 0.75 × W/H and M = W/√(W² + H²). Native coordinates map to `GetCursorPosition` coordinates by the factor S × 1024/M. The side is therefore exactly 10 `GetCursorPosition` units for every screen width and height. **INFERRED:** the simulator's `GetCursorPosition` space is its layout canvas, with Y counted from the bottom.

`DisplayError(error)` requires a string or number; a number is converted with Lua number formatting. Empty text does nothing. Other text fires `PING_SYSTEM_ERROR(text)`, which Blizzard routes to `UIErrorsFrame` (`EventRouting.lua:73`, `EventImplementation.lua:167–168`). It returns no values.

## Sends and pending hit-test information

The five send members return a table with a numeric `result` (`Enum.PingResult`) and, only when a type is selected, a numeric `type` (`Enum.PingSubjectType`). An optional type argument is absent when nil or missing. Other values convert to an integer (0 when conversion fails) and narrow to one byte, so 256 becomes 0; a byte above 9 is an argument error.

Every send starts with the shared eligibility check, in this order:

1. `enablePings` set to 0 gives `FailedDisabledBySettings` (4). An absent CVar counts as enabled.
2. An active cooldown gives `FailedSpamming` (2).
3. In a group, a squelched player (`C_SocialRestrictions.IsSquelched`) gives `FailedSquelched` (6).
4. In a group, the `C_PartyInfo` ping restriction can give `FailedDisabledByLeader` (3). The group leader always passes. Otherwise `Lead` denies everyone, `Assist` denies everyone except assistants, and `TankHealer` denies the damage role.

The clock is the cached millisecond clock that `GetTime` reads. The burst state holds a count, a limit, a window, the window start, the last submission time and the cooldown pair that `C_Ping.GetCooldownInfo` reads:

- While a cooldown is active, a send is refused until the clock reaches the pair's end. At or after the end, the cooldown ends and the pair becomes (0, 0); that check does no idle reset.
- Without a cooldown, 3000 ms or more since the last submission resets the count to 0, the limit to 2 and the window to 3000 ms, and starts the window now.
- A submission first resets the count to 0 when the count is at least the limit and the window has elapsed since its start. It then increments the count.
- A count above the limit stores the pair {now, now + 3000} and starts the cooldown. It then calls the cooldown-started callback with one new `{startTimeMs, endTimeMs}` table. The callback runs synchronously, before the send returns; its errors go to the error handler.
- The last submission time is recorded after the callback. The crossing send itself succeeds.

`SendUnitPing(target [, type, isPlayerResource])` requires a GUID string. After eligibility, an empty GUID gives `FailedUnspecified` (7), and a GUID that resolves to no simulator unit gives `FailedOutOfPingArea` (5). The local player's GUID succeeds. Without a type argument it selects 4 when the target can attack the player with reaction at most 1, and 5 otherwise; for the player this is 5.

`SendHitTestPing([type])` takes the pending target. A pending GUID is sent as a unit ping. Otherwise, if any position component has an absolute value of at least 2⁻²² (equality qualifies), the point sender runs its own eligibility check. With neither, the result is `FailedGeneric` (1). The pending information is cleared in every case, including a refused send. `ClearHitTestPingInfo()` clears it and returns no values. The simulator has no world hit selection, so hosts supply the pending target in `SimState.ping_pending`.

`SendPlayerItemPing(itemID)`, `SendPlayerSpellPing(spellID)` and `SendPlayerSpellCategoryPing(spellCategoryID)` convert their ID from a number or numeric string, truncating toward zero within the signed 32-bit range. After eligibility, ID 0 gives `FailedUnspecified` (7). A spell category other than 4, 30 or 1711 gives `FailedGeneric` (1).

## UI receivers and world targets

`GetTargetPingReceiver(mousePosX, mousePosY)` takes `GetCursorPosition` coordinates and returns one value: the receiving frame, or nil. It walks the frame list in creation order and keeps the eligible frame that draws on top: higher strata wins, then higher frame level; a tie keeps the first frame. A candidate must:

- be a frame, not a texture, font string or line;
- be effectively visible, with effective alpha above 0;
- contain the point in its rendered rectangle minus its hit-rect insets.

It does not need mouse input. A top-level frame qualifies unless `ping-top-level-pass-through` is set. Any other frame needs `ping-receiver` and ignores pass-through. An attribute counts as set for any non-nil value, including `false` and 0, and names match without regard to case. Retail 12.1.5 publishes `Enum.PingSetTargetState` (`PingConstantsDocumentation.lua:37–48`).

The simulator has no world geometry, so every world query misses:

- `SetHitTestPingTarget(mousePosX, mousePosY [, forcePointPing])` returns `Failed` (1). It stores nothing and keeps the old pending information.
- `SetHitTestTargetAndSendPing()` returns `{result = 1}` without a type. It skips the pending send, eligibility and cooldown bookkeeping.

Basis: Retail 12.1.5 build 70688 WM38, WM38F restriction reporting, WM38F-B CreateFrame, WM47 Q1–Q10 and WM51 F1–F7. **INFERRED:**

- Only functions convert as setter callbacks; missing, nil and other values are argument errors.
- The burst state starts with the idle-reset values.
- A type byte above 9 and a non-string GUID are argument errors. `enablePings` text that is not a number counts as 0.
- `SendUnitPing`'s contextual type uses the simulator's `UnitCanAttack` and `UnitReaction` models.
- Group rank 2 is the leader and rank 1 an assistant. The role masks 2, 4 and 8 are tank, healer and damage.
- The frame list is in creation order. `GetCursorPosition` space is the layout canvas. Hit-rect edges are half-open.

**Known gaps:**

- The macro-origin `MACRO_ACTION_FORBIDDEN(functionName)` variant (WM47 Q11 UNKNOWN), the method-call name prefix, and the callback error copy and rethrow path.
- In eligibility, the player's own squelch flag (mask 64) and the secondary role-flag remap (WM51-F2). Both count as clear.
- GUID text parsing, and the resolver's range and visibility policy (WM51-F3). Resolution covers only the simulator's token-backed units.
- Non-self unit suitability flags and redirection (WM47 Q7, WM51-F3). These raise a "not modeled" error.
- The point sender's distance check against the player position after eligibility (WM51-F7). The simulator has no player world position, so this raises a "not modeled" error.
- Past the zero-ID check, these raise a "not modeled" error: item toy ownership and use predicates (WM47 Q5), the spell effect-aura predicate and spell metadata (WM47 Q3, Q6), and accepted spell categories, which need the category cooldown query (WM47 Q3).
- In receiver selection: the rendering-priority flag, raised frame levels, parent-group comparisons, and the script-binding, access and execution-permission checks (WM47 Q8).
