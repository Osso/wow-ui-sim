# Aura expiration-time query — B86

`C_UnitAuras.DoesAuraHaveExpirationTime(auraInstanceUnit, auraInstanceID)` reports whether a stored aura record has an expiration time. The 12.0.5 [GlobalAPI source](../../data/patch-api/sources/12.0.5-api-changes.txt) lines 364–365 change its argument policy: `# SecretArguments AllowedWhenTainted -> AllowedWhenUntainted`. Source ID: `global api-C_UnitAuras-DoesAuraHaveExpirationTime-365`.

Cached retail `Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua` lines 107–125 declare nonnil `auraInstanceUnit: UnitToken`, nonnil `auraInstanceID: number`, one nonnil `hasExpirationTime: bool`, `SecretArguments = "AllowedWhenUntainted"`, `RequiresUnitAuraAccess`, `RequiresValidUnitAuraInstance` and `SecretWhenUnitAuraRestricted`. Contract context, not native execution evidence.

## What it must do

### Stored result

- [ ] Return exactly one public boolean: true when the matched stored aura has a nonzero `expiration_time`, false when it is zero.
- [ ] Resolve helpful and harmful player records and party buffs/debuffs through the existing public aura enumeration; records resolve only on their own unit and instance ID. That enumeration drops instances listed in `C_UnitAuras._blockedAuras`, so a blocked record reports false; `target` resolves a fixed fixture list, not per-environment state. Neither is asserted by this spec's tests.
- [ ] Read stored state live; queries never mutate aura records. Environments are isolated.
- [ ] Unknown unit, unknown instance and nil unit return one public false. Inferred miss policy, not native-verified.
- [ ] A unit that is neither string nor nil, or an instance ID that is not a finite number, errors before lookup. Inferred representation policy.

### Argument policy under `retail-12-0-5`

- [ ] Untainted callers may pass an authentic secret unit, a secret instance ID, or both; the query resolves them like their public values and returns the public boolean.
- [ ] Tainted callers passing either argument as a secret are denied before validation or lookup, including for unknown units and when the other argument is a malformed public value: both arguments are authenticated before either is validated.
- [ ] Denial and success leave caller taint unchanged and never declassify the inputs; tainted callers keep working with public arguments.
- [ ] Rooted secret arguments survive full garbage collection with identity and secrecy intact.

## How it works

- [Lua API](../lua-api.md)
- [Audit page](../wiki/investigations/patch-12-0-5-api-audit.md)

## Implementation inventory

- `src/c_api/aura_duration.rs` — registration, argument reading and stored-record lookup shared with the duration queries.

## Tests asserting this spec

- `tests/aura_expiration_time.rs` — nine cases: stored result, party isolation, live reads, miss policy, malformed arguments, untainted secret acceptance, tainted denial, GC rooting, immutability and environment isolation.

## Development proof — 2026-10-03

| Stage | Revision | Result |
|---|---|---|
| Inputs RED | `a342c6eec` | 6 PASS / 3 FAIL: the three secret-argument cases fail with `secret unit access is not modeled`. |
| First producer | `0d2a98596` | 27/27 (9 new + 18 `aura_refresh_duration::` controls). Independent GPT-6.1-sol review **REJECTED** it: the unit was validated before the instance ID was authenticated, so a tainted caller's `(12, secretID)` got the unit type error instead of the denial; no secret payload was exposed. It also found the `cfg` annotations and non-12.0.5 reader dead, because the module is already feature-gated. |
| Ordering RED | `08c9e8bc8` | 8 PASS / 1 FAIL at `secret denial precedes unit validation`. |
| Corrected producer | `d1bbdc8e8` | 27/27, cargo exit0, no warnings; `cargo fmt --check` exit0. |

Startup `lua-errors` returned `[]` at `0d2a98596`; not rerun after the reader reorder. Logs live in a session scratchpad and are not durable.

## Known gaps (current cycle)

- [ ] Independent re-verification of `d1bbdc8e8` and source accounting pending. Not asserted by tests: blocked or `target` records, full-record and party immutability beyond player `(instance, expiration)` pairs, arguments past the second (ignored).

## Out of scope

- `RequiresUnitAuraAccess`, `RequiresValidUnitAuraInstance` and `SecretWhenUnitAuraRestricted`: unmodeled. The result is always public; no aura-access restriction state exists for this query.
- Native miss/nil/type behavior and error wording: unverified.
- `GetAuraBaseDuration` and `GetRefreshExtendedDuration` keep their existing conservative rejection of every secret argument; their cached policy is `AllowedWhenTainted` and is not part of row 365.
- Profiles below `retail-12-0-5`: `src/c_api/aura_duration.rs` is compiled and registered only under that feature, so the query does not exist there. The earlier `AllowedWhenTainted` policy is not modeled.
