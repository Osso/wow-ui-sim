# API contract probe

`docs/addons/ApiContractProbe/` prepares native investigations of scalar curve point returns, `UnitSexBase` comparison, unit name/realm returns and finite numeric formatting. It shares one manual recorder; the existing [dispel probe](aura-dispel-curve-probe.md) remains separate. See [capture protocol](../addons/ApiContractProbe/README.md).

## What it must do

- [x] Record build provenance and manual scenario labels without automatic capture or gameplay changes.
- [x] Observe scalar curve empty/populated returns, explicit index queries, fields/GetXY, repeated identity and an isolated returned-point mutation attempt without assuming native outcomes.
- [x] Preserve raw legacy/base sex values and arity for existing units, alongside named enum values; never convert numbering.
- [x] Capture `UnitName` and `UnitNameUnmodified` with `/apicontract names <label>` and `all` for fixed tokens `player`, `party1`–`party4`, `target`, `nonexistent`, `invalid-unit-token` and the empty string. Never skip queries based on `UnitExists`.
- [x] Preserve name/realm positional returns and exact arity, including zero returns, nil slots and empty realm strings, under existing accessibility-first redaction and opaque-error rules. Same/cross-realm fixture identity comes from manual labels, not inferred return values.
- [x] Capture `/apicontract numbers <label>` and `all` numeric samples for `C_StringUtil.FloorToNearestString` and `RoundToNearestString`, with `GetLocale` provenance, build and label. Use exactly the 25 finite inputs listed in the capture protocol: signed ties and their neighbors, integers, fractions and large finite magnitudes.
- [x] Preserve numeric helper result bytes, positional nils and exact arity through the existing bounded observer without guessing rounding or grouping. Missing APIs and failed access checks fail closed; errors remain opaque.
- [x] Redact inaccessible/secret values before comparison or saving, and record opaque failures without stringifying errors.
- [x] Bound capture count, inspected results, arrays, depth and strings.
- [x] Exercise actual addon loading and slash commands through local behavioral fixtures.

## Publication and event recording

- [x] Capture raw and ordinary lookup observations for configured enum, constant and API paths, distinguishing missing parents from missing members.
- [x] Capture configured CVars through `C_CVar.GetCVar` and `GetCVarDefault` without writes.
- [x] Register configured events only on explicit `events-start`; retain registration errors and bounded positional payloads with nil slots.
- [x] Stop listeners with `events-stop`; retain build/source provenance, scenario labels and overflow counts.

`AuditTargets.lua` contains 199 publication/CVar targets and 88 event-registration targets from the blocker snapshot. These are recording capabilities, not 287 complete behavioral probes. Native transitions, earlier builds, load phases, producers and restricted payloads remain unresolved. Configuration processing is bounded to 256 targets per category; events to 256 and payloads to eight values. Passive event payloads use raw table inspection without invoking their methods or `__index`.

Runtime `d959372a3` passes 10/10 fixtures (exact-hash reuse) plus 21/21 independent supplemental checks. Proof: `/tmp/verify-all-probes-recorders-corrected-ledger.json`. The initial passive-inspection bug is preserved by regression `5870cfa46`; no native API credits follow from recorder tests.

## How it works

- [Capture protocol](../addons/ApiContractProbe/README.md)

## Implementation inventory

- `docs/addons/ApiContractProbe/ApiContractProbe.lua`: shared observation and manual curve, sex, name, numeric, publication and event captures.
- `docs/addons/ApiContractProbe/ApiContractProbe.toc`: manual addon and SavedVariables registration.
- `docs/addons/ApiContractProbe/AuditTargets.lua`: exact snapshot-derived capture targets and source hash.

## Tests asserting this spec

`docs/addons/ApiContractProbe/tests/harness.lua`. Initial RED at `e2adbb57d`: addon not yet present; exit 1. Logs `/tmp/api-contract-probe-red.stdout` and `/tmp/api-contract-probe-red.stderr`. Runtime `a055b98c9` passes 8/8 local fixtures. Independent `/tmp/verify-api-contract-probe-ledger.json` reuses exact-hash fixture proof and passes 14/14 supplemental checks, including invalid indices, read-only points, raw values, access filtering and real addon wiring. This proves the recorder, not native API semantics.

Name fixtures exercise differing modified/unmodified names, nil/empty/explicit realms, positional nils and zero returns, all fixed tokens without existence gating, redacted secrets/inaccessible values, opaque errors, missing APIs and `all` inclusion. These are recorder tests, not native name/realm expectations.

Numeric fixtures assert the literal 25-input corpus against deliberately different fake clients, locale bytes, multiple/nil/zero returns, redacted results, opaque errors and missing/failed access APIs. They do not establish native rounding rules.

## Known gaps (current cycle)

- [ ] Native numeric formatting corpora on matching builds/locales; nonfinite/coercion/security behavior and values outside the finite corpus remain outside this bounded recorder.
- [ ] Native captures on a matching client, including independently identified same-realm and cross-realm party fixtures for names.

## Out of scope

Installation, native execution, inferred numeric mappings or vector contracts, secret access bypass, arbitrary object serialization and gameplay manipulation. No API audit credit from local recorder fixtures.
