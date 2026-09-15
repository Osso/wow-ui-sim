# Native API contract probes

The scalar curve point and `UnitSexBase` investigations share one manual [ApiContractProbe](../../addons/ApiContractProbe/README.md). The existing [AuraDispelCurveProbe](../../addons/AuraDispelCurveProbe/README.md) covers dispel-ID input. All three requested probes are prepared; none was installed or executed in a native client by this task.

## Evidence boundaries

Scalar point declarations specify vectors but do not establish indexing, ordering or returned-object identity. The recorder observes empty/populated curves, explicit index queries, accessible fields/GetXY, repeated equality and an isolated mutation attempt. Cached `Blizzard_SharedXML/Vector2D.lua` provides vector fields but does not prove curve return behavior.

`UnitSexBase` returns `Enum.UnitSex`; legacy `UnitSex` numbering cannot be copied blindly. The recorder stores both raw returns and named enum values for real existing units, with scenario labels for independently observed transformations. It does not infer what “Base” means.

Dispel curve numeric IDs remain pending the separate prepared capture. See [[aura-dispel-curve-probe]].

## Local proof

Tests `e2adbb57d` initially failed because the new addon did not exist. Runtime `a055b98c9`: 8/8 fixtures pass; independent verification reuses exact hashes and passes 14/14 supplemental checks. Actual TOC/slash/SavedVariables wiring, bounded captures, raw values, inaccessible-value redaction, opaque errors, missing APIs and read-only/invalid-index results were checked. Proof: `/tmp/verify-api-contract-probe-ledger.json`.

Fixture outputs are not native contracts. No audit credits or totals changed. Native capture remains required before assigning compatibility evidence.

## Sources

- [Recorder spec](../../specs/api-contract-probe.md)
- [Manual capture protocol](../../addons/ApiContractProbe/README.md)
- [Dispel probe spec](../../specs/aura-dispel-curve-probe.md)

## See Also

- [[patch-12-0-0-api-audit]] — unresolved native contracts
- [[aura-dispel-curve-probe]] — companion experiment
