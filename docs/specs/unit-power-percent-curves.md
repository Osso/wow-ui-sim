# UnitPowerPercent curves

`UnitPowerPercent(unit, powerType, unmodified, curve)` evaluates supplied scalar/color curves from retail 12.0.0. Source: `src/lua_api/globals/utility_system_spell/spell_api.rs`. See [Lua API architecture](../lua-api.md) and [curve objects](curve-objects.md).

## What it must do

- [ ] Evaluate argument 4 using current modeled power percentage and return exactly one scalar/color result.
- [ ] Reflect live primary power updates and explicit secondary player power selection.
- [ ] Preserve omitted/nil-curve numeric behavior and one-result arity.
- [ ] Reject unsupported non-nil curve values through the existing evaluator.
- [ ] Verify retail 12.0.0/12.0.5/12.0.7 behavior.

The existing `current / max * 100` input is simulator policy, not native-scale evidence. Zero maximum retains existing zero input/result policy. `unmodified` remains unmodeled. Earlier profiles retain existing behavior; their curve semantics remain unverified.

## How it works

- [Lua API architecture](../lua-api.md)
- [Curve object contract](curve-objects.md)

## Implementation inventory

- `src/lua_api/globals/utility_system_spell/spell_api.rs`: existing power lookup and optional curve evaluation.
- `src/c_api/c_curve_util.rs`: scalar/color validation and evaluation.

## Tests asserting this spec

`tests/admin_health_power_api.rs`: `unit_power_percent_` tests cover scalar/color evaluation, primary/secondary updates, arity, nil/omitted queries and invalid curve rejection.

Tests `eb495dbc7` reached RED: two ordinary-query passes and four supplied-curve failures. Runtime `2ba41cda2` evaluates non-nil argument 4 through the existing evaluator; verification remains pending. Proof: `/tmp/unit-power-percent-curves-red-ledger.json`.

## Known gaps (current cycle)

- [ ] Independent runtime/profile verification and bounded audit provenance.

## Out of scope

Native percentage scale, validation/coercion, unknown-unit semantics, `unmodified` scaling, zero-max defaults, secrets/security and full-LoD availability remain unverified. No interpolation, vendor or unit-lookup changes.
