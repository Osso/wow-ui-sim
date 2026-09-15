# API contract probe

`docs/addons/ApiContractProbe/` prepares both requested native investigations: scalar curve point returns and `UnitSexBase` comparison. It shares one manual recorder; the existing [dispel probe](aura-dispel-curve-probe.md) remains separate. See [capture protocol](../addons/ApiContractProbe/README.md).

## What it must do

- [x] Record build provenance and manual scenario labels without automatic capture or gameplay changes.
- [x] Observe scalar curve empty/populated returns, explicit index queries, fields/GetXY, repeated identity and an isolated returned-point mutation attempt without assuming native outcomes.
- [x] Preserve raw legacy/base sex values and arity for existing units, alongside named enum values; never convert numbering.
- [x] Redact inaccessible/secret values before comparison or saving, and record opaque failures without stringifying errors.
- [x] Bound capture count, inspected results, arrays, depth and strings.
- [x] Exercise actual addon loading and slash commands through local behavioral fixtures.

## How it works

- [Capture protocol](../addons/ApiContractProbe/README.md)

## Implementation inventory

- `docs/addons/ApiContractProbe/ApiContractProbe.lua`: shared observation and two experiments.
- `docs/addons/ApiContractProbe/ApiContractProbe.toc`: manual addon and SavedVariables registration.

## Tests asserting this spec

`docs/addons/ApiContractProbe/tests/harness.lua`. Initial RED at `e2adbb57d`: addon not yet present; exit 1. Logs `/tmp/api-contract-probe-red.stdout` and `/tmp/api-contract-probe-red.stderr`. Runtime `a055b98c9` passes 8/8 local fixtures. Independent `/tmp/verify-api-contract-probe-ledger.json` reuses exact-hash fixture proof and passes 14/14 supplemental checks, including invalid indices, read-only points, raw values, access filtering and real addon wiring. This proves the recorder, not native API semantics.

## Known gaps (current cycle)

- [ ] Native captures on a matching client.

## Out of scope

Installation, native execution, inferred numeric mappings or vector contracts, secret access bypass, arbitrary object serialization and gameplay manipulation. No API audit credit from local recorder fixtures.
