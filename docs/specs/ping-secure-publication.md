# Retail secure ping publication

Retail 12.1.5 documents `C_PingSecure` as `SecureOnly` in `Blizzard_APIDocumentationGenerated/PingManagerSecureDocumentation.lua:3–6`. The pinned `Blizzard_PingUI/Blizzard_PingUI.toc` loads PingUI into the secure environment. Build 70688 WM38 registration evidence supports a separate secure namespace, not an ordinary public-global installation.

The simulator publishes the existing `C_PingSecure` namespace in the secure environment and removes its public-global binding for Retail 12.1.5. Earlier Retail epochs and other profiles are unchanged. This publication contract does not by itself claim that every member's native behavior is modeled. Other client code could install an alias; such aliases are not modeled here.

## Retail 12.1.5 restrictions and modeled members

Every modeled member checks the caller's taint provenance first. A tainted caller is refused before any argument is read: the member returns no values and the shared reason-0 reporter fires `ADDON_ACTION_FORBIDDEN(source, functionName)`. `functionName` is the live Lua debug name of the call plus `()`: the member name for dotted access, the alias name for local, global or upvalue aliases, and `UNKNOWN()` when no name exists, such as a call through `pcall`. It is cut to 1023 bytes, so a long name can lose its suffix. A refusal inside a running report is not reported again.

The eight `Set*Callback` members replace their slot with a required callback; the secret-aware argument is unwrapped first. `ClearPendingPingOffScreenCallback` is not declared for Retail 12.1.5 and is not published there.

`CreateFrame()` returns no values. The first secure call creates one unnamed native `Frame`, stores it, and synchronously calls the installed radial-wheel-created callback with that frame only. Later calls change nothing and do not notify again, so a callback installed after creation is never told about the frame. Errors raised by the callback go to the error handler instead of the caller.

`DisplayError(error)` requires a string or number; a number is converted with Lua number formatting. Empty text does nothing. Other text fires `PING_SYSTEM_ERROR(text)`, which Blizzard routes to `UIErrorsFrame` (`EventRouting.lua:73`, `EventImplementation.lua:167–168`). It returns no values.

Basis: Retail 12.1.5 build 70688 WM38, WM38F restriction reporting and WM38F-B CreateFrame. **INFERRED:** only functions convert as setter callbacks; missing, nil and other values are argument errors. **Known gaps:** the macro-origin `MACRO_ACTION_FORBIDDEN(functionName)` variant (the simulator has no macro-origin source), the method-call name prefix, the radial frame's parent, anchoring and size, the callback error copy and rethrow path, `ClearHitTestPingInfo`, `GetTargetPingReceiver`, `SetHitTestPingTarget`, `SetHitTestTargetAndSendPing` and the five send members (pending WM47 native evidence).
