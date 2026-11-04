# PTR action-combat range monitoring

`C_ActionBar.EnableActionCombatActionRangeCheck(actionID, enable)` manages the
separate counted action-combat channel in `src/c_api/action_combat_range_check.rs`.
Its monitor accepts explicit simulator results, not invented geometry or targets.
See [PTR tooltip investigation](../wiki/investigations/ptr-tooltip-residue.md).

WM93 is bounded **PTR 12.1.7.70668** evidence applied to **70493 pinned UI inputs**,
not same-build proof. The Retail absence comparison is **12.1.5.70688**. No client
was run. The [WM92 contract](ptr-bank-reagents-action-hover.md#evidence-authority)
shares this evidence limitation and the slot reader. Pinned PTR declarations:
ActionBarFrameDocumentation.lua:11–20 and :1088–1100. Original ActionButton.lua:433
stores the callback used by :403 via rule-enabled :942 and OnEnter:1492.

## What it must do

- [x] Publish the API on PTR only; Retail must reject the event registration and
  expose neither an ordinary nor raw API member. PTR's existing event table
  admits ACTION_COMBAT_ACTION_RANGE_CHECK_UPDATE; no Retail event is added.
- [x] Read numeric/numeric-string action identifiers with WM92's unsigned-number
  validation and subtract-before-truncate conversion. Unlike Enter/Leave, retain
  converted keys outside 0–407: ordinary slot 0 registers key -1; 409 registers 408.
- [x] Require non-nil enable, then use accessible Lua truthiness: false disables;
  zero, empty string and all other non-nil values enable. Ignore extra ordinary
  arguments. Missing/nil usage errors identify the argument number; ordinary
  slot-range errors remain distinct from secret-access errors.
- [x] Under rule 204 only, enable increments and disable decrements the per-key
  32-bit reference count. Multiple enables require balanced disables. Exactly zero
  removes the registration and its cached result. Do not change WM92 hover stores.
- [x] Registration, partial disable and final disable emit no update event.
  Ordinary permitted calls return zero values (**INFERRED** native return contract).
- [x] The explicit monitor initializes from supplied results, then emits on first
  evaluation, a changed available range result, availability change or explicit
  force. Unchanged initialized results do not emit. Removing the registration
  discards initialization; a later re-enable can produce an initial event again.
- [x] Publish `(oneBasedSlot, inRangeBoolean, availableBoolean)`. When unavailable,
  retain the previous cached range Boolean; it is not a valid standalone answer.
  Converted keys outside 0–407 are unavailable even when the supplied snapshot
  requests availability. Never replace either Boolean with nil.
- [x] The unchanged rule-enabled original OnEnter/OnLeave selector passes without
  a callback stub, registration bypass, removed texture or weakened assertion.

## How it works

- [PTR tooltip investigation](../wiki/investigations/ptr-tooltip-residue.md)
- [Event architecture](../event-system.md)

## Implementation inventory

- `src/c_api/action_combat_range_check.rs`: counts, cached results, PTR API/admin registration.
- `src/c_api/action_combat_range_check/unknown_policy.rs`: named UNKNOWN security,
  exceptional native-diagnostic handling and explicit scheduling/target seam.
- `src/c_api/action_button_hover.rs`: shared WM92/WM93 one-based reader.
- `src/c_api/action_button_hover/unknown_policy.rs`: shared accessible-argument and
  extreme signed-conversion seams.
- `src/lua_api/state/sim_state.rs`, `src/lua_api/state.rs`: distinct per-environment
  counted channel and explicit supplied results.
- `src/lua_api/env_init/mod.rs`: registration after admin namespace initialization.

## Tests asserting this spec

- `tests/ptr_tooltip_combat_range.rs`: parser/truthiness/counts, rule gate, hover
  independence, secret-access distinction, monitor notifications and Retail absence.
- `tests/ptr_tooltip_residue.rs`: unchanged rule-enabled original action consumer.

## Known gaps (current cycle)

All five new direct selectors and the unchanged original OnEnter/OnLeave
selector were observed RED before the producer and GREEN after it. Current
full-filter proof (all enumerated selectors, source-bound test units):

| Scope | Result |
|---|---|
| PTR `tooltip_` | 528 passed, 0 failed, 1 ignored |
| PTR lib | 2194 passed, 3 accepted baseline failures, 1 ignored |
| Retail `tooltip_` | 527 passed, 0 failed, 1 ignored, including raw/ordinary API and event absence |

The three remaining library selectors are pinned PTR identity, edit-mode
loss-of-control enum and hyperlink tooltip data. No other failure or incomplete
result remains in this run. These are results on the lane's source, not a claim
about unmerged later master tests or a fully passing PTR library profile.

## Out of scope

- **UNKNOWN scheduling and targets:** `A_Admin.SweepActionCombatRangeChecks(force)`
  is an explicit **EMULATED** scheduling seam using the existing synchronous
  dispatcher, not a native polling interval or listener-delivery guarantee.
  `supplied_results` is the explicit target/geometry evaluation seam. No supplied
  snapshot means no evaluation. The first snapshot supplies both Booleans,
  including the cached Boolean when initially unavailable; subsequent unavailable
  snapshots retain the previous cached range. No target is selected or inferred.
- **UNKNOWN security:** native complete protected/combat/taint contexts are not
  established. The named policy reuses runtime AllowedWhenUntainted argument
  access as EMULATED behavior, without combat lockdown or taint resets.
- **UNKNOWN native diagnostics/traps:** inactive-rule and negative-count paths
  have native diagnostics, not established ordinary Lua errors. The simulator's
  named EMULATED diagnostic policy reports to stderr and preserves the store
  instead of simulating a native trap or fabricating a Lua argument error.
  Both monitor/observer callbacks are provided by the simulator adapter; native
  callback installation/lifetime is not reproduced.
- **UNKNOWN extreme conversion consequences:** accepted unsigned values outside
  signed-32 representation use a named EMULATED narrowing seam, not a claim about
  all protected native execution paths. Ordinary keys -1, 0–407 and 408 are covered.
- Cursor changes, tooltip ownership/content, actual geometric eligibility and
  automatic destruction cleanup are not supplied by this registration API.
