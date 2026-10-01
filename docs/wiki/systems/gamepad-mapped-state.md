# Forever mapped-stick queries

`f774ce454` adds a bounded environment-local input and two Forever queries; `f442b0913` adds the separate free-look-hover policy. [Contract and inference limits](../../specs/gamepad-mapped-state.md) own requirements, unknown native fields/signatures and incomplete acceptance.

## Backing representation

`c_api::c_game_pad::MappedStickSnapshot` contains an ordered `Vec<MappedStick>`; each entry contains a configuration name and x/y model inputs. `SimState.gamepad_mapped_sticks` is optional, initially None. It is separate from logical input style and widget gamepad-enable flags. No physical device record or full mapped DTO exists.

The getter clones the optional input and drops the state borrow before Lua allocation. It roots the result on the stack, attaches the stick array, then attaches each child before field-key interning. Selector lookup clones the chosen name before Lua string allocation. Neither path unwraps secrets or changes caller taint.

## Consumer and evidence boundary

Actual cached `Blizzard_GamepadSharedUtility/InputBindingStack/InputAxisBinding.lua` eagerly computes both `inputBindingAxisListener` centering fields during TOC loading. `tests/gamepad_mapped_state.rs` installs concrete input before loading that unchanged TOC's dependency closure; it asserts these source-owned fields as well as query results. No vendor overrides or callback bypasses.

Current `hover-green-ledger.json` and sibling stdout/stderr at source-scoped `a956dfdd3`, rilua `6044544b`, record 75 cases, 71 pass / 4 fail. All eleven mapped/hover cases pass: three direct-model, three unchanged cached initializer and five hover cases. Initializer fixtures observe absent snapshots centering both sides, nonzero Movement/zero Camera selecting left/right, and reordered names selecting configured sides. Four AutoRoll cases stop before local lifecycle decisions at `Blizzard_StaticPopup_Game` requirements for `C_Club.GetInvitationCandidates` and `C_GameRules.IsHardcoreActive`; classification complete; fixture cleanliness scope remains user-owned. No masking, exemption or native conclusion follows. Earlier namespace-gap and `model-v2` hover-dependency failures remain historical.

The [comparison evidence](../investigations/forever-addon-comparison.md#mapped-stick-and-autoroll--current-pin-boundary) records 36 prior messaging/module/restricted/BugCapture passes and 24 loot/instance control passes on the current pin. Independent historical audit `20366` is complete: `independent-report.md` in the same proof directory confirms 63/70 and recorded cache/CVar/vendor/binary attribution. Default fmt/check evidence in `/tmp/patch-12.0.5-batch11-rust-gates.json` is historical; changed current inputs prevent blanket current-source acceptance. Bounded hover/initializer GREEN is observed; independent audits complete; scoped default gates recorded in the contract, overall FAIL. Native hover/controller and Reveal remain unmodeled. The retained SharedXML Reveal diagnostic changes debug setup, not native parity. No archive/matrix counter credit or overall completion follows.

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
