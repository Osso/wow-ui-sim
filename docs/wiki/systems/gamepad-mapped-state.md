# Forever mapped-stick queries

`f774ce454` adds a bounded environment-local input and two Forever queries; `f442b0913` adds the separate free-look-hover policy. [Contract and inference limits](../../specs/gamepad-mapped-state.md) own requirements, unknown native fields/signatures and incomplete acceptance.

## Backing representation

`c_api::c_game_pad::MappedStickSnapshot` contains an ordered `Vec<MappedStick>`; each entry contains a configuration name and x/y model inputs. `SimState.gamepad_mapped_sticks` is optional, initially None. It is separate from logical input style and widget gamepad-enable flags. No physical device record or full mapped DTO exists.

The getter clones the optional input and drops the state borrow before Lua allocation. It roots the result on the stack, attaches the stick array, then attaches each child before field-key interning. Selector lookup clones the chosen name before Lua string allocation. Neither path unwraps secrets or changes caller taint.

## Consumer and evidence boundary

Actual cached `Blizzard_GamepadSharedUtility/InputBindingStack/InputAxisBinding.lua` eagerly computes both `inputBindingAxisListener` centering fields during TOC loading. `tests/gamepad_mapped_state.rs` installs concrete input before loading that unchanged TOC's dependency closure; it asserts these source-owned fields as well as query results. No vendor overrides or callback bypasses.

Current `hover-green-ledger.json` and sibling stdout/stderr at source-scoped `a956dfdd3`, rilua `6044544b`, record 75 cases, 71 pass / 4 fail. All eleven mapped/hover cases pass: three direct-model, three unchanged cached initializer and five hover cases. Initializer fixtures observe absent snapshots centering both sides, nonzero Movement/zero Camera selecting left/right, and reordered names selecting configured sides. Four AutoRoll cases stop before local lifecycle decisions at `Blizzard_StaticPopup_Game` requirements for `C_Club.GetInvitationCandidates` and `C_GameRules.IsHardcoreActive`; classification complete; user approved consumed-path-only testing. The [AutoRoll fixture contract](../../specs/forever-auto-roll-fixture.md) retains exact setup diagnostics and records the unexecuted calibrated observer and full-history guards. No executed isolation, AutoRoll outcome or native conclusion follows. Earlier namespace-gap and `model-v2` hover-dependency failures remain historical.

The [comparison evidence](../investigations/forever-addon-comparison.md#mapped-stick-and-autoroll--current-pin-boundary) records 36 prior messaging/module/restricted/BugCapture passes and 24 loot/instance control passes on the current pin. Independent historical audit `20366` is complete: `independent-report.md` in the same proof directory confirms 63/70 and recorded cache/CVar/vendor/binary attribution. Default fmt/check evidence in `/tmp/patch-12.0.5-batch11-rust-gates.json` is historical; changed current inputs prevent blanket current-source acceptance. Bounded hover/initializer GREEN is observed; independent audits complete; scoped default gates recorded in the contract, overall FAIL. Native hover/controller and Reveal remain unmodeled. The retained SharedXML Reveal diagnostic changes debug setup, not native parity. No archive/matrix counter credit or overall completion follows.

## Explicit enabled capability

[Gamepad query contract](../../specs/forever-gamepad-enabled.md) covers the Forever enabled capability and the Retail addon-compatibility queries. `C_GamePad.IsEnabled` reads `SimState.cvars.get_bool("GamePadEnable")`; the existing default is `"0"`, and an explicit stored `"1"` enables it. This is simulator policy, not native detection or proof of native CVar/device equivalence.

Forever 1.60.1.70532 `GamePadDocumentation.lua:222-229` declares `IsEnabled` with no arguments and one non-nil bool. Retail `GamePadDocumentation.lua:126-131,174-185,213-219` declares `GetAllDeviceIDs`, `GetDeviceRawState` and `IsEnabled`. **INFERRED (simulator mapping):** Retail has no physical-device input, so the first returns an empty table and the second nil; `IsEnabled` reads the existing CVar. PTR `IsEnabled` remains absent. Mapped sticks, input-interface style and hover policy remain separate inputs.

`tests/forever_gamepad_enabled.rs` selects the original Game-screen restricted addon and verifies the wrapper file's secure TOC annotation. It obtains `IsGamePadEnabled` through original `CallRestrictedClosure`, verifies its source, secure environment, and inner `func` at `RestrictedEnvironment.lua:176`, then invokes the original restricted call across stored CVar `0/1/1/0/0` with absent/nonzero/cleared stick snapshots. Test-first `00e3c265a` fails at the original line177 nil `IsEnabled`; producer `5efd665d9` passes. Test bytes remain identical. Forever stock remains 26 keys/26 occurrences, zero additions/removals; this reached fixture gap was not a stock failure. Retail/PTR exact raw-publication controls pass. Their stock maps match master exactly: Retail 16 keys/18 occurrences, PTR 296/410, zero additions/removals/count changes. All-target checks including tools pass on Forever, Retail and PTR; Retail has zero warnings, the other two retain only the three approved pre-existing fast-handler dead-code warnings. Formatting passes. Native detection is still unknown, and startup is not clean.

## Retail device queries

Retail registers the declared `C_GamePad` namespace members used by DialogueUI. There is no simulated device store: **INFERRED (simulator mapping)** `GetAllDeviceIDs()` returns an empty table, `GetDeviceRawState(deviceID)` returns nil, and `IsEnabled()` uses the existing CVar. This does not model physical hardware or connection events; see the [query contract](../../specs/forever-gamepad-enabled.md).

## Bounded free-look-hover policy

Cached `GamePadDocumentation.lua:135-141,231-238,305-314` documents a non-nil bool getter, bool setter with `AllowedWhenUntainted`, and synchronous `GAME_PAD_ALLOW_HOVER_EVENTS_WITH_FREE_LOOK_CHANGED` carrying one bool. `f442b0913` publishes the getter/setter only for Forever and registers exactly that event. The setter uses VM secret access checks, requires a boolean, stores the value, releases the state borrow, then dispatches through `fire_named_event_state`. Optional mapped-stick state remains separate.

Initial false, change-only emission and state-before-callback are explicit simulator guesses, not native-verified semantics. No physical cursor/hover filtering is implemented.

Five tests committed in `0ff9d1c5a` have actual RED: all stop registering the unknown event, before getter/transition/security assertions. The combined RED records 3 pass / 12 fail, not getter-behavior failures. Collect-all diagnostics at `ee3e27172` report only the setter as unexpected within the failed GamepadSharedUtility TOC in all seven cached cases; later dependency closure remains unknown. The exact Reveal setup exception remains retained, not generalized or suppressed. No cached initializer or AutoRoll decision credit follows.

Post-implementation runtime GREEN covers the five hover cases, not native semantics. Original GREEN build runner was interrupted by Pi restart and proven a terminated zombie with no surviving compiler; lost-output proof remains retained. `hover-green-retry-build-ledger.json` records retry exit 0 and unchanged inputs at `a956dfdd3`; runtime artifacts are named `hover-green-*`, not `hover-green-retry-*`. Runtime ledger records unchanged binary/addon/host CVar hashes. Later source commits are not blanket-covered. Full native DTO, hardware/physical hover, Reveal and executed other-profile absence remain unproved; no AutoRoll or inventory credit.

## Sources

- [Mapped-stick contract](../../specs/gamepad-mapped-state.md) — requirements, proof artifact location, policies and exclusions.
- [Logical input style](../../specs/forever-input-interface-style.md) — separate current-style behavior.
- `src/c_api/c_game_pad.rs`, `tests/gamepad_mapped_state.rs` — committed implementation and behavioral fixtures; all eleven mapped/hover cases have bounded GREEN in the current cited run.

## See Also

- [[lua-api]] — runtime surface.
- [[forever-addon-comparison]] — separate addon workflow evidence.
