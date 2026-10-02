# Aura dispel color arguments

Batch45 bounded producer contract for **exact Retail 12.0.5 source/register row378**: `C_UnitAuras.GetAuraDispelTypeColor`, `SecretArguments AllowedWhenTainted -> AllowedWhenUntainted`. Scope: sole epoch-owned C API producer and registration, preserving existing lookup/state/curves and fixtures. No Cargo target or accounting change. Requirements below remain unchecked pending parent GREEN and independent acceptance; implemented code alone is not execution proof.

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

- `src/c_api/c_unit_aura_dispel_color.rs`: sole `retail-12-0-5` producer; upfront authentication of all three arguments, strict validation, typed mapping, real curve evaluation and authentic wrapped-color output.
- `src/c_api/mod.rs`, `src/lua_api/globals/register.rs`: epoch-gated module/registration after existing aura namespace initialization.
- `src/lua_api/globals/auras.rs`: unchanged blocked-inclusive typed lookup; old getter/import/registration now earlier-epoch-only, not a runtime fallback.
- `src/c_api/c_curve_util.rs`: existing native-identity color-curve factory, registry validation and evaluation helper; unchanged.
- `tests/aura_dispel_color_arguments.rs`: sixteen fixtures for the chosen row378 contract.
- `tests/c_unit_auras_admin.rs`: two existing fixtures corrected to use required real color curves.

## Tests asserting this spec

`tests/aura_dispel_color_arguments.rs`: **16** `retail-12-0-5`-gated fixtures, auto-grouped by existing integration harness. No Cargo/new-target edits.

| Capability | Fixtures | Proof level |
| --- | ---: | --- |
| Step mapping, numeric linear x, custom nondispel/Magic colors, player/party lookup, signed endpoints | 5 | Implemented; saved RED; parent GREEN pending; mapping/representations INFERRED |
| Required unit/ID, malformed UTF-8/numeric bounds, native curve identity, missing/unknown stored data | 5 | Implemented; saved RED; parent GREEN pending; error/representation policies INFERRED |
| Secure each/mixed secrets, authenticated payload validation, tainted guard before miss, secret output/GC/recovery | 4 | Implemented using actual VM authentication/wrapping; saved RED; parent GREEN pending; output policy INFERRED |
| Curve/point/result/DTO/record/block/provider immutability and two-environment isolation | 2 | Existing models preserved by producer; saved RED; parent GREEN pending |

`tests/c_unit_auras_admin.rs`: two existing tests now supply required real step curves with x0 transparent white and x1 Magic `(0.2, 0.6, 1, 1)`. Existing alpha/type/icon/RGBA assertions are retained, not weakened. Their historical fixed-palette successes did not demonstrate curve evaluation.

Producer slice runs **no tests, builds, checks, readability gates, delegation or operations**. Only the four producer-owned Rust files are formatted with `rustfmt --edition 2024 --config skip_children=true`; unchanged fixtures are not reformatted. Coherent producer/spec/system-doc changes commit before parent GREEN. Parent owns subsequent GREEN and independent acceptance; no pass, native permission/output, profile-wide, consumer or row-accounting credit follows implementation. No disputed aura-duration, batch44 spec, wiki/index/log, data or PLAN edits.

## Saved parent RED and producer rooting

Fixture revision `9a2f15ac878a5b2c804d63867d7ddd92f7d6313a`: `/tmp/patch-12.0.5-batch45-red-build-result.json` records `cargo test --test integration --no-run --message-format=json`, exit0/99.040s. `/tmp/patch-12.0.5-batch45-red-run.json` records the compiled `aura_dispel_color_arguments::` selection, exit101/2.556s, **0 PASS / 16 FAIL**. Integration binary SHA256 `376ea4d00dacdca8c3ead28dbfd058e2d7e85dc5a6db002778809231231dc174`; full `.stdout`/`.stderr` artifacts share the run prefix. Evidence includes preserved unowned dirty source, not clean-revision proof. Every failure reaches custom-curve/recovery RGBA mismatch because the old getter ignores argument3; these are **not sixteen independent security RED claims**.

The producer inspects original curve-wrapper identity and runs actual VM `unwrap_secret` on all three positions before any representation/identity/miss validation. Thus wrong public unit plus secret ID, or missing ID plus secret curve, reaches the actual untainted-caller guard first. Authenticated unit/ID/curve values are pushed as explicit stack roots before registry/key allocations and helper calls. The evaluated ColorMixin table is rooted before actual `wrap_secret` allocates a wrapper; the wrapper is immediately rooted too. Temporary stack top restores on success/error, with no GC safe point between restoration and pushing the successful result. Existing VM userdata marking traverses generic secret payloads; no new traversal or declassification is needed. Only an authentic secret curve wrapper propagates output secrecy under the inferred policy; secret unit/ID alone do not.

## Corrected mutation fixtures — 2026-10-02

First producer run at `5fb039e8f`: compile0/233.147s, **14 PASS / 2 FAIL**, exit101/10.141s; startup0 `[]`. Saved `/tmp/patch-12.0.5-batch45-green-{build-result,runs,startup-run}.json` and full outputs. Two failures reached incidental `SetRGBA` calls after successful curve/secret-output checks. Simulator color tables expose mutable `r/g/b/a` and getter methods, not that setter; existing curve snapshot fixtures use field mutation. Corrected only those three mutation sites, asserting changed RGBA before unchanged subsequent evaluations. No production or native setter behavior changed; ColorMixin method completeness remains unproved. Original sixteen-failure RED and first producer run remain historical; corrected fixture GREEN/acceptance pending.

## Known gaps (current cycle)

- [x] Saved parent compiled RED for exact fixtures, with shared custom-curve failure boundary and dirty-source provenance recorded above.
- [x] Curve-evaluating producer and upfront VM authentication implemented; owned formatting only, no producer execution claim.
- [ ] Parent GREEN and independent acceptance before any bounded row378 credit.

## Out of scope

Native access/valid-instance enforcement, restricted-unit output secrecy, native secrecy from secret curve points, exact native mapping/error/output parity, user-field curve overrides and all other source rows. [Native recorder](aura-dispel-curve-probe.md) remains observation preparation, not evidence establishing these unknowns.
