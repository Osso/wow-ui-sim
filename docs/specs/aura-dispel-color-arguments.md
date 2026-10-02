# Aura dispel color arguments

Batch45 fixture/spec contract for **exact Retail 12.0.5 source/register row378**: `C_UnitAuras.GetAuraDispelTypeColor`, `SecretArguments AllowedWhenTainted -> AllowedWhenUntainted`. No producer, registration, Cargo target or accounting change belongs to this slice.

## Grounding and evidence limits

Actual retail runtime documentation: `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua:244–266`. It declares required `auraInstanceUnit: UnitToken`, `auraInstanceID: number`, and `curve: LuaColorCurveObject`; all three are nonnil, under `AllowedWhenUntainted`. Documentation says: “with the dispel type ID used as the 'x' value.” Return: exactly one nonnil `colorRGBA` with `ColorMixin`.

The same declaration sets `RequiresUnitAuraAccess`, `RequiresValidUnitAuraInstance`, `SecretWhenUnitAuraRestricted`, and `SecretWhenCurveSecret`. These annotations establish obligations, **not proof that simulator native access/restricted-output policies exist**. This slice has no native-client capture or parity credit.

Exact provenance: [plaintext source](../../data/patch-api/sources/12.0.5-api-changes.txt), line378; [register](../../data/patch-api/sources/12.0.5-register.json), `global api-C_UnitAuras-GetAuraDispelTypeColor-378`.

Observed old backing contract: `src/lua_api/globals/auras.rs:454–464` ignores argument3. It looks up the typed aura, selects built-in debuff RGB, otherwise returns transparent white. **It does not evaluate a curve.** `/tmp/patch-12.0.5-next-registered-aura-delta-map.md` is not authoritative for this behavior. `src/c_api/c_curve_util.rs` supplies real color-curve userdata, private registry/kind validation and callable `evaluate_curve_value`; fixtures use actual known curve methods, not producer/vendor replacements or method overrides. Existing modeled lookup/state mechanics: [Lua API system](../wiki/systems/lua-api.md).

## What it must do

### Evaluation and typed lookup

- [ ] Require all three documented arguments, authenticating secret inputs before representation validation or missing-record lookup.
- [ ] Return one ColorMixin-compatible RGBA value obtained by evaluating the supplied real color curve at numeric dispel x. Step/linear fixtures expose distinct x values in all channels; fixed palette/transparent fallback is insufficient.
- [ ] Read concrete `AuraInfo.dispel_type: Option<String>` in player and seeded party helpful/harmful stores, including blocked records. Preserve existing blocked-inclusive instance lookup rather than borrowing from another unit or inventing a catalog.
- [ ] Use chosen **INFERRED** mapping: `None -> 0`, `Magic -> 1`, `Curse -> 2`, `Disease -> 3`, `Poison -> 4`, `Enrage -> 9`. Nondispellable x0 must evaluate the caller's curve, including nontransparent x0 colors.
- [ ] Unknown stored strings (including empty/lowercase variants) error under chosen **INFERRED** policy, not silently map to None or an invented catalog.
- [ ] Missing unit or instance errors under chosen **INFERRED** policy motivated by valid-instance/nonnil declarations. No fabricated transparent-color fallback; native error semantics/wording remain unknown.

### Authentication and strict inputs

- [ ] Secure callers accept authentic secret STRING unit, NUMBER ID and generic-secret-wrapped native color-curve userdata independently and all mixed combinations. Authentication must preserve original wrappers and caller taint, not generically declassify inputs.
- [ ] Tainted callers deny each/mixed secret argument before miss/type validation, including secret missing unit, secret ID with public missing unit, secret curve with missing unit/ID, and mixed wrong public types. Fixtures distinguish the actual VM untainted-caller guard category from ordinary query errors; this is not native error-message parity.
- [ ] Require actual UTF-8 STRING unit and actual finite integral signed-i32 NUMBER ID (**INFERRED representation policy**). Reject omitted/nil inputs, coercible numeric strings, malformed bytes, fractions, nonfinite/out-of-range numbers and other actual types. Zero, negative and signed endpoints accept when records exist.
- [ ] Require actual native `LuaColorCurveObject` identity and private registry kind. Reject nil/omission, plain `CreateColor`, table/spoofed curve methods, numeric `LuaCurveObject`, arbitrary userdata/frame, scalar and wrong authenticated payloads. Wrapping an invalid object cannot turn it into a color curve.
- [ ] Across success/denial, rooted GC preserves argument identity, input-slot taint/secrecy and caller taint. Public recovery occurs in the same executing helper/tainted probe closure, without clearing taint; secure reentry still accepts original secrets.

### Secret curve output and immutable state

- [ ] For secure evaluation using an authentic secret-wrapped native curve, return an authentic **secret-wrapped color result** under chosen **INFERRED wrapper-output policy**. Secure unwrap exposes actual evaluated RGBA; tainted unwrap/access to original curve wrapper or output denies. Root native curve before wrapping; root wrapper before global insertion/GC; preserve identity through GC and secure/tainted/public/secure sequences.
- [ ] Treat secrecy of a generic curve wrapper separately from native curve secrecy propagated from **secret POINTS**. Secret-points propagation is unmodeled and untested here; wrapper-output tests do not establish native `SecretWhenCurveSecret` or `SecretWhenUnitAuraRestricted` parity.
- [ ] Queries leave input colors, curve type/count/points, prior results, typed aura fields/order, DTO snapshots, block state and provider selection unchanged. Returned colors and point snapshots remain independent: mutating them cannot change later curve evaluation.
- [ ] Two environments isolate records, real curves, blocked visibility and provider selection.

## How it works

- [Lua API system](../wiki/systems/lua-api.md).
- [Curve objects contract](curve-objects.md).
- [Aura instance filter query](unit-aura-filter-query.md).

## Implementation inventory

- `src/lua_api/globals/auras.rs`: existing blocked-inclusive typed lookup and old fixed-palette getter; unchanged.
- `src/c_api/c_curve_util.rs`: existing native-identity color-curve factory, registry validation and evaluation helper; unchanged.
- `tests/aura_dispel_color_arguments.rs`: sixteen fixtures for the chosen row378 contract.
- `tests/c_unit_auras_admin.rs`: two existing fixtures corrected to use required real color curves.

## Tests asserting this spec

`tests/aura_dispel_color_arguments.rs`: **16** `retail-12-0-5`-gated fixtures, auto-grouped by existing integration harness. No Cargo/new-target edits.

| Capability | Fixtures | Proof level |
| --- | ---: | --- |
| Step mapping, numeric linear x, custom nondispel/Magic colors, player/party lookup, signed endpoints | 5 | Authored; mapping/representations INFERRED; parent compiled RED pending |
| Required unit/ID, malformed UTF-8/numeric bounds, native curve identity, missing/unknown stored data | 5 | Authored; error/representation policies INFERRED; parent compiled RED pending |
| Secure each/mixed secrets, authenticated payload validation, tainted guard before miss, secret output/GC/recovery | 4 | Authored against actual VM wrapping/guard contract; output policy INFERRED; parent compiled RED pending |
| Curve/point/result/DTO/record/block/provider immutability and two-environment isolation | 2 | Authored against existing typed lookup/curve model; parent compiled RED pending |

`tests/c_unit_auras_admin.rs`: two existing tests now supply required real step curves with x0 transparent white and x1 Magic `(0.2, 0.6, 1, 1)`. Existing alpha/type/icon/RGBA assertions are retained, not weakened. Their historical fixed-palette successes did not demonstrate curve evaluation.

Fixture slice runs **no tests, builds, checks, readability gates, delegation or operations**. Only the two owned Rust files are formatted with `rustfmt --edition 2024 --config skip_children=true`. Parent owns compiled RED and subsequent producer/acceptance proof; no pass, native permission/output, profile-wide, consumer or row-accounting credit follows authored fixtures. No disputed aura-duration, batch44 spec, wiki/index/log, data or PLAN edits.

## Known gaps (current cycle)

- [ ] Parent compiled RED for these exact fixtures; no execution proof recorded here.
- [ ] Curve-evaluating producer and argument authentication, owned by parent after RED.
- [ ] Parent GREEN and independent acceptance before any bounded row378 credit.

## Out of scope

Native access/valid-instance enforcement, restricted-unit output secrecy, native secrecy from secret curve points, exact native mapping/error/output parity, user-field curve overrides, producer/registration changes and all other source rows. [Native recorder](aura-dispel-curve-probe.md) remains observation preparation, not evidence establishing these unknowns.
