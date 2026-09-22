# UnitHealthPercent curves

`UnitHealthPercent` evaluates supplied scalar and color curves from retail 12.0.0 and on Forever. Source: `src/lua_api/globals/utility_system_spell/spell_api.rs`. See [curve objects](curve-objects.md) and [Lua API architecture](../lua-api.md).

## What it must do

- [x] Evaluate a supplied scalar curve using current modeled health percentage; return exactly one numeric result.
- [x] Evaluate a supplied color curve using current modeled health percentage; return exactly one color result.
- [x] Reflect explicit player and target health updates on subsequent queries.
- [x] Preserve omitted/nil-curve numeric results and one-result arity.
- [x] Reject non-nil values that are not supported curve objects with the existing evaluator error.
- [x] Verify retail 12.0.0/12.0.5/12.0.7 behavior.
- [x] Enable Forever argument-3 evaluation with normalized `health / healthMax` curve input while preserving ordinary 0–100 results.

Retail retains its existing `health / healthMax * 100` curve-input policy. Forever uses normalized `health / healthMax` input, inferred from unchanged DinoUnitFrames `8936218` power-curve thresholds `id / 5` and cached Blizzard `CurveConstants.ScaleTo100` points `(0, 0)` and `(1, 100)`, applied to the analogous documented health contract. This is not native-verified scale evidence. Omitted/nil curves still return `health / healthMax * 100` on every profile. A zero maximum retains zero ratio/input (a supplied curve may map zero to a nonzero result). `usePredicted` remains unmodeled and uses current health; other profiles retain their prior behavior.

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

Committed curve tests `13bf5219f` reached RED: two curve failures, one nil-preservation pass. Invalid-curve test `b31004297` reached RED because invalid values were accepted. Runtime `92b4f8c4f` passes all five `unit_health_percent_` tests on retail 12.0.0/12.0.5/12.0.7; 12.0.0 exact-byte proof was reused and later profiles were fresh. Independent fmt/check/build/startup proof passed; startup returned `[]`. Full proof: `/tmp/verify-unit-health-percent-curves-ledger.json`. Metadata proof `68ad13265`: 15,063 fresh hashes, zero stale, 59 renewals, seven additions, one credit, validator exit 0 and all 3,410 rows matching. Totals **2345 / 1063 / 2**; snapshot **1,089 / 282**.

Forever regressions in the same grouped integration target (`admin_health_power_api::forever_percent`) cover live scalar health, target color `GetRGBA`, one-result arity, omitted/nil numeric queries, zero maximum, and unsupported curves. Development evidence is recorded separately in `/tmp/forever-addon-audit/forever-percent-development-ledger.json`; independent verification and unchanged-addon replay belong to the parent integration slice.

## Known gaps (current cycle)

- [ ] Earlier-profile curve behavior remains unverified.
- [ ] Native curve input scale and default/no-curve units remain unverified.

## Out of scope

- Predicted-health modeling, supported/unknown-unit semantics and native validation fidelity: no new backing behavior in this slice.
- Secrets/security and full-LoD availability: deferred audit obligations, not established by ordinary curve tests.
- Interpolation algorithms, extrapolation, security and vendor code are unchanged. The shared Forever power integration is specified in [UnitPowerPercent curves](unit-power-percent-curves.md).
