# Action cooldown duration

`C_ActionBar.GetActionCooldownDuration(actionID)` exposes existing simulator action-slot cooldown state as a duration object. Its pinned 12.0.0 declaration promises a duration object; the mappings below are simulator policies, not native-client evidence.

## What it must do

- [x] Resolve an assigned action slot through its spell cooldown and active GCD, selecting the existing model's later-ending interval, consistently with `C_ActionBar.GetActionCooldown`.
- [x] Return a duration object with the selected start, duration, and rate `1`; elapsed and remaining queries use the simulator clock.
- [x] Return zero timing for empty valid slots or assigned slots without an active cooldown/GCD.
- [x] Snapshot timing at query time: later slot/cooldown changes affect a new query, not the earlier object's configured interval.
- [ ] On Forever and Retail-family 12.1+ builds, `C_ActionBar.GetActionCooldown` includes non-nil `isActive`, true when the selected cooldown is enabled and both start and duration are nonzero. Clear/expired intervals return false. Earlier profiles retain their existing four-field payload; no whole Retail epoch is enabled for Forever. Implementation committed with targeted GREEN pending integration.

## How it works

- [Duration core](duration-core.md)
- [12.0.0 audit](../wiki/investigations/patch-12-0-0-api-audit.md)

## Implementation inventory

- `src/c_api/c_action_bar.rs`: shared action cooldown lookup, cooldown-info table, and duration-object producer.
- `src/lua_api/globals/action_bar_api.rs`: existing namespace registration owner and spell/GCD interval selection.
- `src/lua_api/globals/lua_duration_object.rs`: shared timed-duration construction and registered setters/queries.

## Tests asserting this spec

`tests/cooldown_probes.rs::get_action_cooldown_duration_*` covers active state, runtime clock bounds, later-ending GCD, snapshot independence, and empty/inactive/expired state. Initial RED at `8afb0edfd`: three failures and one passing zero-state control; GREEN at `eb9383afb`: 4/4, exit 0 in `/tmp/action-cooldown-duration-green-ledger.json`. `/tmp/verify-action-cooldown-duration-reconciled-ledger.json` reuses matching 12.0.0 4/4 and 12.0.5/12.0.7/Mists 14/14 proof, plus format, readability, binaries, startup `[]`, metadata validation, and 14,808 fresh references. It records the corrected warning-free default `cargo check` at `6987cbf21`: exit 0 with no warnings or errors.

### Active-field contract and regression

Pinned Forever `ActionBarFrameDocumentation.lua:164–180` and current Retail/PTR declarations return `SpellCooldownInfo`. `SpellSharedDocumentation.lua:19–30` requires `isActive` and defines disabled/zero-start/zero-duration cases as inactive. The committed 12.0.0 source register instead describes four-field `C_ActionBar.ActionBarCooldownInfo`. The bounded gate covers the inspected current profiles and preserves older epochs; it does not establish the historical introduction version.

`tests/cooldown_probes.rs::get_action_cooldown_active_*` covers actual admin assignment of slot 1 to spell 19750 with a five-second cooldown, clearing, expiration, zero start, and earlier-profile absence of the field. No addon or acceptance assertion is changed. Runtime RED: `/tmp/ellesmere-forever/runtime-data-trace-ledger.json` records action start `43.98364306`, duration `5`, enabled `true`, but absent `isActive` after roughly `0.665` seconds; the spell query reports active. Compiled GREEN remains integration-owned.

## Known gaps (current cycle)

- [ ] Native snapshot/lifetime, invalid-slot/coercion/error, GCD and identity semantics remain unverified.
- [ ] No direct cached Blizzard consumer was found; API-state tests do not establish real-consumer acceptance.

## Out of scope

- Later `ignoreGCD` option, secrets/security, item/charge/loss-of-control producers, non-default rates, and cooldown-engine redesign.
