# UnitHealthPercent curves

`UnitHealthPercent` evaluates supplied scalar and color curves from retail 12.0.0. Source: `src/lua_api/globals/utility_system_spell/spell_api.rs`. See [curve objects](curve-objects.md) and [Lua API architecture](../lua-api.md).

## What it must do

- [x] Evaluate a supplied scalar curve using current modeled health percentage; return exactly one numeric result.
- [x] Evaluate a supplied color curve using current modeled health percentage; return exactly one color result.
- [x] Reflect explicit player and target health updates on subsequent queries.
- [x] Preserve omitted/nil-curve numeric results and one-result arity.
- [x] Reject non-nil values that are not supported curve objects with the existing evaluator error.
- [ ] Preserve earlier-profile behavior; curve evaluation starts at `retail-12-0-0`.

The existing `health / healthMax * 100` scale is simulator policy for both ordinary results and curve input, not native-scale evidence. A zero maximum retains the existing zero result/input policy. `usePredicted` remains unmodeled and uses current health.

## How it works

- [Lua API architecture](../lua-api.md)
- [Curve object contract](curve-objects.md)

## Implementation inventory

- `src/lua_api/globals/utility_system_spell/spell_api.rs`: health lookup and optional curve evaluation.
- `src/c_api/c_curve_util.rs`: existing scalar/color validation and evaluation.

## Tests asserting this spec

`tests/admin_health_power_api.rs`:

- `unit_health_percent_supplied_scalar_curve_tracks_live_health`
- `unit_health_percent_supplied_color_curve_tracks_live_target_health`
- `unit_health_percent_supplied_nil_preserves_uncurved_queries`
- `unit_health_percent_rejects_invalid_curves`
- `unit_health_percent_uses_player_health_values`
- `unit_health_percent_ignores_legacy_truthy_curve_argument` (earlier profiles only)

Committed curve tests `13bf5219f` reached RED: two curve failures, one nil-preservation pass. Invalid-curve test `b31004297` reached RED because invalid values were accepted. Runtime `92b4f8c4f` passes all five `unit_health_percent_` tests on retail 12.0.0 (exit 0; six existing warnings). Full outputs, command and exact source hashes: `/tmp/unit-health-percent-curves-green-ledger.json`. Earlier-profile behavior remains unverified.

## Known gaps (current cycle)

- [ ] Independent historical-profile and earlier-profile verification.
- [ ] Native curve input scale and default/no-curve units remain unverified.

## Out of scope

- Predicted-health modeling, supported/unknown-unit semantics and native validation fidelity: no new backing behavior in this slice.
- Secrets/security and full-LoD availability: deferred audit obligations, not established by ordinary curve tests.
- Changes to `UnitPowerPercent`, interpolation algorithms or vendor code: unrelated to this integration.
