# UnitPowerPercent curves

`UnitPowerPercent(unit, powerType, unmodified, curve)` evaluates supplied scalar/color curves from retail 12.0.0 and on Forever. Source: `src/lua_api/globals/utility_system_spell/spell_api.rs`. See [Lua API architecture](../lua-api.md) and [curve objects](curve-objects.md).

## What it must do

- [x] Evaluate argument 4 using current modeled power percentage and return exactly one scalar/color result.
- [x] Reflect live primary power updates and explicit secondary player power selection.
- [x] Preserve omitted/nil-curve numeric behavior and one-result arity.
- [x] Reject unsupported non-nil curve values through the existing evaluator.
- [x] Verify retail 12.0.0/12.0.5/12.0.7 behavior.
- [x] Enable Forever argument-4 evaluation with normalized `current / max` curve input, including explicit secondary combo-point power.

Retail retains its existing `current / max * 100` curve-input policy. Forever uses normalized `current / max` input, inferred from unchanged DinoUnitFrames `8936218` `basecombopoints.lua` thresholds `id / 5` and cached Blizzard `CurveConstants.ScaleTo100` points `(0, 0)` and `(1, 100)`. Neither convention is native-runtime verification. Omitted/nil curves retain numeric `current / max * 100` results on every profile. Zero maximum retains zero ratio/input (a supplied curve may map zero to a nonzero result). `unmodified` remains unmodeled. Other profiles retain existing behavior; earlier-profile curve semantics remain unverified.

## How it works

- [Lua API architecture](../lua-api.md)
- [Curve object contract](curve-objects.md)

## Implementation inventory

- `src/lua_api/globals/utility_system_spell/spell_api.rs`: existing power lookup and optional curve evaluation.
- `src/c_api/c_curve_util.rs`: scalar/color validation and evaluation.

## Tests asserting this spec

`tests/admin_health_power_api.rs`: `unit_power_percent_` tests cover scalar/color evaluation, primary/secondary updates, arity, nil/omitted queries and invalid curve rejection.

Tests `eb495dbc7` reached RED: two ordinary-query passes and four supplied-curve failures. Runtime `2ba41cda2` evaluates non-nil argument 4 through the existing evaluator. Independent proof `/tmp/verify-unit-power-percent-curves-ledger.json` passes six power tests plus five health regressions (11/11) on each retail 12.0.0/12.0.5/12.0.7; fmt/check/default binary build/startup passed and startup returned `[]`. Metadata `f846069c9` records 64 renewals, seven additions and one credit. Correction `9c75403d2` renewed one stale prior health-spec reference; final snapshot binding `52297be78` proves **15,070 fresh / zero stale**, validator exit 0 and all 3,410 rows matching. Totals **2346 / 1062 / 2**; snapshot **1,088 / 282**.

Forever regressions in the same grouped integration target (`admin_health_power_api::forever_percent`) cover live primary scalar power, Dino-style secondary combo-point color `GetRGBA`, one-result arity, omitted/nil numeric queries, zero maximum, and invalid curves. Development evidence is recorded separately in `/tmp/forever-addon-audit/forever-percent-development-ledger.json`; independent verification and unchanged-addon replay belong to the parent integration slice.

## Known gaps (current cycle)

- [ ] Earlier-profile curve behavior remains unverified.

## Out of scope

Native percentage scale, validation/coercion, unknown-unit semantics, `unmodified` scaling, zero-max defaults, secrets/security and full-LoD availability remain unverified. No interpolation, vendor or unit-lookup changes.
