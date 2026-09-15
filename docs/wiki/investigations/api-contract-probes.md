# Native API contract probes

The scalar curve point and `UnitSexBase` investigations share one manual [ApiContractProbe](../../addons/ApiContractProbe/README.md). The existing [AuraDispelCurveProbe](../../addons/AuraDispelCurveProbe/README.md) covers dispel-ID input. These initial recorder experiments are prepared; none was installed or executed in a native client by this task. They do **not** cover the user's request for all remaining probe needs.

## Evidence boundaries

Scalar point declarations specify vectors but do not establish indexing, ordering or returned-object identity. The recorder observes empty/populated curves, explicit index queries, accessible fields/GetXY, repeated equality and an isolated mutation attempt. Cached `Blizzard_SharedXML/Vector2D.lua` provides vector fields but does not prove curve return behavior.

`UnitSexBase` returns `Enum.UnitSex`; legacy `UnitSex` numbering cannot be copied blindly. The recorder stores both raw returns and named enum values for real existing units, with scenario labels for independently observed transformations. It does not infer what “Base” means.

Dispel curve numeric IDs remain pending the separate prepared capture. See [[aura-dispel-curve-probe]].

## Local proof

Tests `e2adbb57d` initially failed because the new addon did not exist. Runtime `a055b98c9`: 8/8 fixtures pass; independent verification reuses exact hashes and passes 14/14 supplemental checks. Actual TOC/slash/SavedVariables wiring, bounded captures, raw values, inaccessible-value redaction, opaque errors, missing APIs and read-only/invalid-index results were checked. Proof: `/tmp/verify-api-contract-probe-ledger.json`.

Fixture outputs are not native contracts. No audit credits or totals changed. Native capture remains required before assigning compatibility evidence.

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
