# PTR/Forever bank reagents and action-button visualization

`ReagentsFromBankAllowed` and `C_ActionBar.EnterActionButton` / `LeaveActionButton`
implement bounded native decisions over supplied state. The producers live in
`src/lua_api/globals/real/bank_reagents.rs` and `src/c_api/action_button_hover.rs`.
See [Lua API architecture](../lua-api.md) for runtime state and API registration.

## Evidence authority

WM92 (`WM92-ptr-actionbar-reagents.md`) is **PTR 12.1.7.70668** evidence applied
to this PTR profile, whose UI inputs are pinned to **12.1.7.70493**. It does **not**
prove these behaviors on 70493. WM92 used no running client; its contract is partial.
WM103b (`WM103b-enter-action-button-closed.md`, same image) closes the entry state
model: three standalone static values, exactly one writer pair and one builder
reader, no protected wrapper, no combat-lockdown check, no events and no tooltip
operation. The native-name absence comparison is Retail **12.1.5.70688**. No
equivalent Retail native behavior or default is inferred from absence.

Pinned PTR declarations: PlayerScriptDocumentation.lua:1611–1618 and
ActionBarFrameDocumentation.lua:35–44,960–969. Original consumers:
Blizzard_FrameXMLUtil/ItemUtil.lua:315–320 and
Blizzard_ActionBar/Shared/ActionButton.lua:1501,1512. WM92 identifies rule 204 as
ActionCombatSpellHitIndicationEnabled and rule 211 as CraftFromBankRestricted;
ActionButton.lua:941,951 also uses the first name. Their PTR enum entries must
refer to the shared C_GameRules rule store.

R39.2: Forever 1.60.1.70707 and PTR 12.1.7.70668 use the same decision over `CraftFromBankRestricted` and a player permission bit. The universal initial value and permission-bit producer remain UNKNOWN.

## What it must do

### Bank reagents

- [ ] Publish the global on PTR and through Forever's Game installer; return exactly one Boolean. Forever calls read no arguments.
- [ ] No player object returns false. A present player returns true when
  CraftFromBankRestricted is absent/inactive; with that rule active it returns
  the supplied permission bit. Do not infer bank-open, ownership, location or CVar state.
- [ ] The Retail/PTR/Forever original recipe-source consumer reaches ItemUtil.lua:315 and completes its supplied source-text tooltip. Its fixture explicitly supplies a present player with permission bit false and no restrictive rule.

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
- [x] Original OnEnter and OnLeave handlers execute without swallowed errors;
  two distinct supplied action records drive two distinct values.
- [x] Expose the builder read as the named `range_visualization_override` seam:
  flag zero returns no override; a zero value returns no override even with the
  flag set; a set flag and non-zero value return the value. The stored slot is
  not an input. No cursor, reticle, action-button or tooltip effect is modeled.

## How it works

- [Lua API architecture](../lua-api.md)
- [Frame data flow](../frame-data-flow.md)

## Implementation inventory

- `src/c_api/action_button_hover.rs`: supplied visualization records, local stores,
  the builder-read seam, parser, PTR registrations and two game-rule enum entries.
- `src/c_api/action_button_hover/unknown_policy.rs`: named UNKNOWN execution-policy seam.
- `src/lua_api/globals/real/bank_reagents.rs`: PTR/Forever Game global, supplied player/rule query and named UNKNOWN default/publication seams.
- `src/lua_api/state/sim_state.rs`, `src/lua_api/state.rs`: per-environment inputs.
- `src/lua_api/env_init/mod.rs`, `src/lua_api/globals/action_bar_api/registration.rs`:
  registration entry points; active-profile checks reside in the producers.

## Tests asserting this spec

- `tests/ptr_tooltip_residue.rs`: reagent matrix, ItemUtil consumer, parser/arity,
  lookup gates, secret-policy distinction, range-visualization builder read,
  original action handlers, Retail absence.
- `tests/ptr_tooltip_hover_leave.rs`: independent original OnLeave consumer RED/GREEN.
- `tests/blizzard_professions_templates_loads.rs`: Retail/PTR/Forever original recipe-source tooltip.
- `tests/forever_bank_reagents.rs`: Forever player/rule matrix without asserting a universal default or event producer.
- `tests/tooltip_hover.rs`: original action hover retains spell tooltip postcall identity.

## Known gaps (current cycle)

- [ ] Native `GAME_RULES_CHANGED` fires on whole game-rule collection replacement. The simulator has no such host input or event producer. A single-entry `A_Admin.SetGameRule` must not synthesize it.

The original rule-enabled OnEnter boundary is closed by the separate
[WM93 counted channel](ptr-action-combat-range-check.md), not by a callback stub
or a changed assertion. Current profile proof belongs to that follow-up contract.
The earlier WM92-only failed selector and unbound-source library results remain
historical handoff evidence, not the current profile result.

- [ ] Same-build 70493 native evidence remains unavailable; retain the 70668
  evidence scope and UNKNOWN native security, geometry and scheduling limits.

## Out of scope

- **UNKNOWN universal character default:** `unknown_character_default` is one
  named EMULATED scenario seam: player present, restrictive rule inactive. The shared
  rule store starts empty. This is not an unconditional native true/false default;
  fixtures supply player/rule state explicitly.
- **UNKNOWN execution/publication policy:** `unknown_policy::read_accessible_argument`
  uses the existing AllowedWhenUntainted runtime access policy as EMULATED behavior;
  full native secret/combat contexts and taint status are not established. The
  reagent `publish_unknown_native_policy` emits an ordinary Boolean with current
  execution provenance, not a claimed always-secret or always-unmarked native value.
- **UNKNOWN downstream effects:** no bank-query or action-button event, tooltip
  ownership, combat policy, cursor change or taint reset is synthesized. The event
  for permission-bit changes is UNKNOWN. Rendering geometry/reticle appearance and
  indirect native object operations are not modeled. Missing supplied records stay
  unavailable, not fabricated.
- **UNKNOWN builder gameplay effect:** `range_visualization_override` is the only
  named seam between the stored flag/value and the range-visualization model. The
  verified read is modeled; the effect of the resolved spell-range-visualization
  record on the cursor, reticle, action button or tooltip is not.
