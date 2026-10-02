# Aura dispel color arguments

Batch45 bounded producer contract for **exact Retail 12.0.5 source/register row378**: `C_UnitAuras.GetAuraDispelTypeColor`, `SecretArguments AllowedWhenTainted -> AllowedWhenUntainted`. Scope: sole epoch-owned C API producer and registration, preserving existing lookup/state/curves and fixtures. No Cargo target or accounting change. Requirements below remain unchecked pending independent acceptance; saved parent GREEN is reconciled below, not native parity.

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
| Step mapping, numeric linear x, custom nondispel/Magic colors, player/party lookup, signed endpoints | 5 | Implemented; saved RED; saved combined GREEN PASS; independent acceptance pending; mapping/representations INFERRED |
| Required unit/ID, malformed UTF-8/numeric bounds, native curve identity, missing/unknown stored data | 5 | Implemented; saved RED; saved combined GREEN PASS; independent acceptance pending; error/representation policies INFERRED |
| Secure each/mixed secrets, authenticated payload validation, tainted guard before miss, secret output/GC/recovery | 4 | Implemented using actual VM authentication/wrapping; saved RED; saved combined GREEN PASS; independent acceptance pending; output policy INFERRED |
| Curve/point/result/DTO/record/block/provider immutability and two-environment isolation | 2 | Existing models preserved by producer; saved RED; saved combined GREEN PASS; independent acceptance pending |

`tests/c_unit_auras_admin.rs`: two existing tests now supply required real step curves with x0 transparent white and x1 Magic `(0.2, 0.6, 1, 1)`. Existing alpha/type/icon/RGBA assertions are retained, not weakened. Their historical fixed-palette successes did not demonstrate curve evaluation.

Producer slice runs **no tests, builds, checks, readability gates, delegation or operations**. Only the four producer-owned Rust files are formatted with `rustfmt --edition 2024 --config skip_children=true`; unchanged fixtures are not reformatted. Coherent producer/spec/system-doc changes commit before parent GREEN. Parent owns subsequent GREEN and independent acceptance; no pass, native permission/output, profile-wide, consumer or row-accounting credit follows implementation. No disputed aura-duration, batch44 spec, wiki/index/log, data or PLAN edits.

## Saved parent RED and producer rooting

Fixture revision `9a2f15ac878a5b2c804d63867d7ddd92f7d6313a`: `/tmp/patch-12.0.5-batch45-red-build-result.json` records `cargo test --test integration --no-run --message-format=json`, exit0/99.040s. `/tmp/patch-12.0.5-batch45-red-run.json` records the compiled `aura_dispel_color_arguments::` selection, exit101/2.556s, **0 PASS / 16 FAIL**. Integration binary SHA256 `376ea4d00dacdca8c3ead28dbfd058e2d7e85dc5a6db002778809231231dc174`; full `.stdout`/`.stderr` artifacts share the run prefix. Evidence includes preserved unowned dirty source, not clean-revision proof. Every failure reaches custom-curve/recovery RGBA mismatch because the old getter ignores argument3; these are **not sixteen independent security RED claims**.

The producer inspects original curve-wrapper identity and runs actual VM `unwrap_secret` on all three positions before any representation/identity/miss validation. Thus wrong public unit plus secret ID, or missing ID plus secret curve, reaches the actual untainted-caller guard first. Authenticated unit/ID/curve values are pushed as explicit stack roots before registry/key allocations and helper calls. The evaluated ColorMixin table is rooted before actual `wrap_secret` allocates a wrapper; the wrapper is immediately rooted too. Temporary stack top restores on success/error, with no GC safe point between restoration and pushing the successful result. Existing VM userdata marking traverses generic secret payloads; no new traversal or declassification is needed. Only an authentic secret curve wrapper propagates output secrecy under the inferred policy; secret unit/ID alone do not.

## Corrected mutation fixtures — 2026-10-02

First producer run at `5fb039e8f`: compile0/233.147s, **14 PASS / 2 FAIL**, exit101/10.141s; startup0 `[]`. Saved `/tmp/patch-12.0.5-batch45-green-{build-result,runs,startup-run}.json` and full outputs. Two failures reached incidental `SetRGBA` calls after successful curve/secret-output checks. Simulator color tables expose mutable `r/g/b/a` and getter methods, not that setter; existing curve snapshot fixtures use field mutation. Corrected only those three mutation sites, asserting changed RGBA before unchanged subsequent evaluations. No production or native setter behavior changed; ColorMixin method completeness remains unproved. Original sixteen-failure RED and first producer run remain historical; corrected fixture saved GREEN PASS; independent acceptance pending.

## Known gaps (current cycle)

- [x] Saved parent compiled RED for exact fixtures, with shared custom-curve failure boundary and dirty-source provenance recorded above.
- [x] Curve-evaluating producer and upfront VM authentication implemented; owned formatting only, no producer execution claim.
- [x] Saved combined parent GREEN: all16 corrected fixtures PASS; see reconciliation below.
- [ ] Independent acceptance before any bounded row378 credit.

## Out of scope

Native access/valid-instance enforcement, restricted-unit output secrecy, native secrecy from secret curve points, exact native mapping/error/output parity, user-field curve overrides and all other source rows. [Native recorder](aura-dispel-curve-probe.md) remains observation preparation, not evidence establishing these unknowns.

## Reconciled combined batch45/46 parent GREEN — 2026-10-01

Saved parent evidence, not independent acceptance. Batch45 producer `2c7cbdc1c37ea71653cfd06c68dda844c9df10c5`, original fixtures `9a2f15ac878a5b2c804d63867d7ddd92f7d6313a`, corrected fixtures `8db0ed1a2a1b0bb1fe339ba50cbd10df07d474ce`; batch46 fixtures `5fb039e8f0dc5cd9e688553fb94b1cd939f0537b`, producer/build revision `f11e8d151cc638227e8a85083d5ceac329d02c6b`.

`/tmp/patch-12.0.5-batch45-46-green-build-result.json` records `cargo test --test integration --no-run --message-format=json`: exit0, **119.52380113198888s**. Integration executable `/syncthing/Sync/Projects/wow/wow-ui-sim/target/debug/deps/integration-a11e89d240f9bd0c`, SHA256 `64791d2cfc776b9bc4bfb0e2e1035f39303f73a607abc8231986c709a37a0f3b`. Every run binds that revision/binary plus preserved unowned dirty duration source diff SHA256 `6967f0b47312d926c2359bd29bc1c26d4d1d523522abf8d87e067104da170a1a`: **dirty-source-bound evidence, not clean-revision proof**. Diff artifact referenced, not read or changed in this docs slice.

`/tmp/patch-12.0.5-batch45-46-green-runs.json` owns thirteen ordered selections. Exact command template: `timeout 90 /syncthing/Sync/Projects/wow/wow-ui-sim/target/debug/deps/integration-a11e89d240f9bd0c FILTER --test-threads=1`. All exits0; **16 color +12 enumeration +168 controls =196 unique PASS**, zero failures. Full outputs: `/tmp/patch-12.0.5-batch45-46-green-run-{0..12}.{stdout,stderr}`; build `.jsonl`/`.log` share the combined prefix.

| Run | FILTER | PASS | Manifest seconds |
| --- | --- | ---: | ---: |
| 0 | `aura_dispel_color_arguments::` | 16 | 4.619249168084934 |
| 1 | `unit_aura_slot_enumeration_arguments::` | 12 | 1.7190134720876813 |
| 2 | `c_unit_auras_admin::` | 14 | 1.747257960960269 |
| 3 | `userdata_proxy::color_curve_` | 18 | 2.424104623030871 |
| 4 | `unit_aura_slot_secret_arguments::` | 12 | 1.4354590170551091 |
| 5 | `aura_application_display_count::` | 14 | 2.075528427027166 |
| 6 | `next125aura::` | 12 | 1.5566162089817226 |
| 7 | `unit_aura_filter_query::` | 14 | 1.8908739370526746 |
| 8 | `aura_table_shape::` | 7 | 0.9552202279446647 |
| 9 | `aura_api::` | 29 | 3.6104179460089654 |
| 10 | `admin_buff_api::` | 18 | 1.8691707890247926 |
| 11 | `aura_refresh_duration::` | 18 | 2.5241610950324684 |
| 12 | `aura_spell_identifier::` | 12 | 1.7038781450828537 |

Warm runtime partition sums to **28.130951017374173s**, below60s target despite196 batched cases; startup adds3.39698344306089s (31.527934460435063s combined runtime). Retain actual duration: no padding, repeat or final whole-goal pass claim.

`/tmp/patch-12.0.5-batch45-46-green-startup-run.json` records `timeout 90 /syncthing/Sync/Projects/wow/wow-ui-sim/target/debug/wow-sim --no-addons --no-saved-vars lua-errors`: exit0, `[]`, **3.39698344306089s**. Binary SHA256 `5358650bde853e3de017824cb32f4f9ff697c164da0f83756461afb837026588`; full `-startup.stdout`/`-startup.stderr` retained.

Original color RED0PASS/16FAIL shares custom-curve/recovery RGBA boundary; not sixteen independent security failures. First producer14PASS/2FAIL reached incidental unsupported `SetRGBA`; correction preserves mutation assertions through direct `r/g/b/a` fields. All sixteen corrected fixtures now saved PASS; full ColorMixin setter completeness remains unproved. Enumeration RED4PASS/8FAIL in `/tmp/patch-12.0.5-batch46-red-run.json` used that same first-attempt compiled binary/build, **no additional RED build**.

Combined independent verifier active; Rust/security/readability and parent acceptance pending. Exact rows378/382 remain uncredited; **241 pending/107 bounded/14 partial =362**, unchanged. Parent owns accounting/gates. Native parity, permission/restriction/access enforcement, secret POINTS propagation, complete ColorMixin setters, all-profile proof and native pagination remain excluded. Earlier pending-GREEN checkpoints are historical; this docs reconciliation runs no build/check/test or independent acceptance.
