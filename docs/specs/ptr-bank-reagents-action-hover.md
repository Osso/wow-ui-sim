# PTR bank reagents and action-button visualization

`ReagentsFromBankAllowed` and `C_ActionBar.EnterActionButton` / `LeaveActionButton`
implement bounded native decisions over supplied state. The producers live in
`src/lua_api/globals/real/bank_reagents.rs` and `src/c_api/action_button_hover.rs`.
See [Lua API architecture](../lua-api.md) for runtime state and API registration.

## Evidence authority

WM92 (`WM92-ptr-actionbar-reagents.md`) is **PTR 12.1.7.70668** evidence applied
to this PTR profile, whose UI inputs are pinned to **12.1.7.70493**. It does **not**
prove these behaviors on 70493. WM92 used no running client; its contract is partial.
The native-name absence comparison is Retail **12.1.5.70688**. No equivalent Retail
native behavior or default is inferred from absence.

Pinned PTR declarations: PlayerScriptDocumentation.lua:1611–1618 and
ActionBarFrameDocumentation.lua:35–44,960–969. Original consumers:
Blizzard_FrameXMLUtil/ItemUtil.lua:315–320 and
Blizzard_ActionBar/Shared/ActionButton.lua:1501,1512. WM92 identifies rule 204 as
ActionCombatSpellHitIndicationEnabled and rule 211 as CraftFromBankRestricted;
ActionButton.lua:941,951 also uses the first name. Their PTR enum entries must
refer to the shared C_GameRules rule store.

## What it must do

### Bank reagents

- [x] Publish the global on PTR only; return exactly one Boolean and ignore extra ordinary arguments.
- [x] Missing shared object returns false. A present object returns true when
  CraftFromBankRestricted is absent/inactive; with that rule active it returns
  the supplied object flag. Do not infer bank-open, ownership, location or CVar state.
- [x] Original recipe-source consumer reaches ItemUtil.lua:315 and completes
  its supplied source-text tooltip. Its fixture explicitly supplies a present
  object with flag false and no restrictive rule.

### Action-button visualization

- [x] Publish both namespace members on PTR only, leaving both ordinary and raw
  Retail lookup nil.
- [x] Accept ordinary numbers and convertible numeric strings in finite
  unsigned-number domain 0–4294967295. Subtract one before truncation toward
  zero. Only converted indices 0–407 perform action work. Thus integer 0/409
  return zero values; 0.25 selects index 0 and 408.9 selects index 407.
- [x] Missing/nil uses the member's Usage text. Conversion/domain failure has
  `outside of expected range 0 to 4294967295 - Usage: ...`; argument access
  errors remain distinct from ordinary parser errors. Successful/unusable calls
  return no values, not a Boolean.
- [x] Enter updates only with ActionCombatSpellHitIndicationEnabled active,
  an action in the shared action bar, a supplied action record and installed
  object, unequal supplied identities and a successful visualization lookup.
  Failure of any gate preserves all local stores. Successful lookup stores the
  converted slot, supplied SpellRangeVisualization value and enabled=true.
- [x] Leave clears enabled and value only for a matching converted slot. It
  preserves the stored slot, is not gated by rule 204 and preserves state for
  other or unusable slots.
- [ ] Original OnEnter and OnLeave handlers execute without swallowed errors;
  two distinct supplied action records drive two distinct values.

## How it works

- [Lua API architecture](../lua-api.md)
- [Frame data flow](../frame-data-flow.md)

## Implementation inventory

- `src/c_api/action_button_hover.rs`: supplied visualization records, local stores,
  parser, PTR registrations and two game-rule enum entries.
- `src/c_api/action_button_hover/unknown_policy.rs`: named UNKNOWN execution-policy seam.
- `src/lua_api/globals/real/bank_reagents.rs`: modeled non-C global and named
  UNKNOWN character-default/publication seams.
- `src/lua_api/state/sim_state.rs`, `src/lua_api/state.rs`: per-environment inputs.
- `src/lua_api/env_init/mod.rs`, `src/lua_api/globals/action_bar_api/registration.rs`:
  registration entry points; active-profile checks reside in the producers.

## Tests asserting this spec

- `tests/ptr_tooltip_residue.rs`: reagent matrix, ItemUtil consumer, parser/arity,
  lookup gates, secret-policy distinction, original action handlers, Retail absence.
- `tests/ptr_tooltip_hover_leave.rs`: independent original OnLeave consumer RED/GREEN.
- `tests/blizzard_professions_templates_loads.rs`: original recipe-source tooltip.
- `tests/tooltip_hover.rs`: original action hover retains spell tooltip postcall identity.

## Known gaps (current cycle)

- [ ] Rule-enabled original OnEnter is blocked by missing
  `C_ActionBar.EnableActionCombatActionRangeCheck`: original ActionButton.lua:433
  stores it as a callback; :1492 → :570 → :942 → :403 invokes nil before Enter.
  Its detailed native contract is UNKNOWN, pending WM93. Keep the selector
  visibly failing; do not bypass registration, remove the texture, or add a stub.

The full requested filters were enumerated and run in bounded partitions. PTR
`tooltip_`: 522 passed, 1 failed (WM93), 1 ignored; Retail: 526 passed, 0 failed,
1 ignored, including raw and ordinary API absence. PTR library: 2192 passed,
4 failed, 1 ignored, and one process-exit case with no completed result. Three
library failures match the earlier tooltip-data report's selectors (identity,
edit-mode enum and hyperlink); the exclusive-addon-root texture failure and
`addon_tests::tests::synchronous_tests_reset_addon_instruction_usage` exit 1 are
additional observed results, not newly attributed regressions. No source-matched
RED library comparison was run. These do not establish a passing PTR profile.

## Out of scope

- **UNKNOWN universal character default:** `unknown_character_default` is one
  named EMULATED scenario seam: object present, restrictive flag clear. The shared
  rule store starts empty. This is not an unconditional native true/false default;
  fixtures supply object/rule state explicitly.
- **UNKNOWN execution/publication policy:** `unknown_policy::read_accessible_argument`
  uses the existing AllowedWhenUntainted runtime access policy as EMULATED behavior;
  full native secret/combat contexts and taint status are not established. The
  reagent `publish_unknown_native_policy` emits an ordinary Boolean with current
  execution provenance, not a claimed always-secret or always-unmarked native value.
- **UNKNOWN downstream effects:** no event emission (including CURSOR_CHANGED),
  tooltip ownership, combat policy, cursor changes or taint reset is synthesized.
  Rendering geometry/reticle appearance and indirect native object operations are
  not modeled. Missing supplied records remain unavailable, not fabricated.
