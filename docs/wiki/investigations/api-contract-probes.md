# Native API contract probes

Scalar curve point, `UnitSexBase`, unit-name/realm, and finite numeric-formatting investigations share one manual [ApiContractProbe](../../addons/ApiContractProbe/README.md). The existing [AuraDispelCurveProbe](../../addons/AuraDispelCurveProbe/README.md) covers dispel-ID input. Runtime `3f7934e27` adds `names`; `41bcd89f0` adds numeric capture; both are included in `all`. These recorder experiments are prepared; none was installed or executed in a native client by this task. They do **not** cover all remaining probe needs.

## Evidence boundaries

Scalar point declarations specify vectors but do not establish indexing, ordering or returned-object identity. The recorder observes empty/populated curves, explicit index queries, accessible fields/GetXY, repeated equality and an isolated mutation attempt. Cached `Blizzard_SharedXML/Vector2D.lua` provides vector fields but does not prove curve return behavior.

`UnitSexBase` returns `Enum.UnitSex`; legacy `UnitSex` numbering cannot be copied blindly. The recorder stores both raw returns and named enum values for real existing units, with scenario labels for independently observed transformations. It does not infer what “Base” means.

Name/realm capture invokes `UnitName` and `UnitNameUnmodified` without existence gating for fixed player, party, target, unknown and invalid-token inputs. It preserves exact arity and positional nils, so nil and empty realms remain distinct from explicit realm strings. Scenario labels identify independently assembled same-realm and cross-realm party slots; returned names never classify the session. Native same-realm and cross-realm sessions remain pending.

`/apicontract numbers <label>` records raw `GetLocale` provenance plus exact arity, positional nils, and bounded accessible result bytes from both `C_StringUtil.FloorToNearestString` and `C_StringUtil.RoundToNearestString`. Its ordered 25 finite inputs cover signed ties and close neighbors, integers, fractions, and large finite magnitudes. It records literal observations only; helper names do not establish rounding or grouping semantics. Nonfinite inputs, coercion, security behavior, values outside this corpus, and actual build/locale captures remain unknown.

Dispel curve numeric IDs remain pending the separate prepared capture. See [[aura-dispel-curve-probe]].

## Local proof

Tests `e2adbb57d` initially failed because the new addon did not exist. The scoped recorder proof now has 16 local fixtures: curve, sex, name/realm, and numeric recording behavior, including deliberately different fake locale bytes, non-rounding strings, multiple/nil/zero returns, restricted/error results, modified/unmodified differences, nil/empty/explicit realms, fixed unknown/invalid tokens, redaction, opaque errors, and `all` inclusion. Actual TOC/slash/SavedVariables wiring remains locally exercised.

The name recorder's prior 13 fixtures plus 12 supplemental checks remain valid within their earlier scoped commit. The numeric fixtures prove literal recording only, not native formatting. Fixture outputs are not native contracts. No audit credits or totals changed. Actual native captures still require matching-client sessions with independently established same-realm and cross-realm party members, plus matching build/locale numeric sessions, before assigning compatibility evidence.

## Complete preparation inventory

[Preparation inventory](../../baselines/native-probe-preparation.json) retains all 282 snapshot plans: 275 active, covering 1,083 rows, and seven without active rows. Snapshot membership excludes best-effort residual gaps and exceptions; their separate reconciliation remains pending.

At `3538b5dc6`, 112 plans have shared publication/event recording components, three have partially reviewed dedicated observations, 158 still need design/recorder review, and two are explicitly deferred security plans. No whole-plan completion is claimed from these counts. Existing older probes may supply more coverage but must be inspected before credit.

`AuditTargets.lua` supplies 199 publication/CVar and 88 event-registration targets. These capture parent/member/CVar observations and real event tuples, not missing producers, historical phases, transitions or semantic contracts. Runtime `e2af1e8e1` initially invoked payload methods during passive event inspection. Regression `5870cfa46` reproduces that error; `d959372a3` uses non-executing raw table inspection for passive payloads. Independent proof reuses 10/10 main fixtures and passes 21/21 supplemental checks, including zero payload `GetXY`/`__index` calls. Ledger: `/tmp/verify-all-probes-recorders-corrected-ledger.json`.

All-probe preparation remains open. No installation or native execution is authorized by recording-code preparation.

## Sources

- [Recorder spec](../../specs/api-contract-probe.md)
- [Manual capture protocol](../../addons/ApiContractProbe/README.md)
- [Dispel probe spec](../../specs/aura-dispel-curve-probe.md)

## See Also

- [[patch-12-0-0-api-audit]] — unresolved native contracts
- [[aura-dispel-curve-probe]] — companion experiment
