# Aura expiration-time query — B86

`C_UnitAuras.DoesAuraHaveExpirationTime(auraInstanceUnit, auraInstanceID)` reports whether a stored aura record has an expiration time. The 12.0.5 [GlobalAPI source](../../data/patch-api/sources/12.0.5-api-changes.txt) lines 364–365 change its argument policy: `# SecretArguments AllowedWhenTainted -> AllowedWhenUntainted`. Source ID: `global api-C_UnitAuras-DoesAuraHaveExpirationTime-365`.

Cached retail `Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua` lines 107–125 declare nonnil `auraInstanceUnit: UnitToken`, nonnil `auraInstanceID: number`, one nonnil `hasExpirationTime: bool`, `SecretArguments = "AllowedWhenUntainted"`, `RequiresUnitAuraAccess`, `RequiresValidUnitAuraInstance` and `SecretWhenUnitAuraRestricted`. Contract context, not native execution evidence.

## What it must do

### Stored result

- [x] Return exactly one public boolean: true when the matched stored aura has a nonzero `expiration_time`, false when it is zero.
- [ ] Resolve helpful and harmful player records and party buffs/debuffs through the existing public aura enumeration; records resolve only on their own unit and instance ID. That enumeration drops instances listed in `C_UnitAuras._blockedAuras`, so a blocked record reports false; `target` resolves a fixed fixture list, not per-environment state. Neither is asserted by this spec's tests.
- [ ] Read stored state live; queries never mutate aura records. Environments are isolated.
- [x] Unknown unit, unknown instance and nil unit return one public false. Inferred miss policy, not native-verified.
- [x] A unit that is neither string nor nil, or an instance ID that is not a finite number, errors before lookup. Inferred representation policy.

### Argument policy under `retail-12-0-5`

- [x] Untainted callers may pass an authentic secret unit, a secret instance ID, or both; the query resolves them like their public values and returns the public boolean.
- [x] Tainted callers passing either argument as a secret are denied before validation or lookup, including for unknown units and when the other argument is a malformed public value: both arguments are authenticated before either is validated.
- [x] Denial and success leave caller taint unchanged and never declassify the inputs; tainted callers keep working with public arguments.
- [x] Rooted secret arguments survive full garbage collection with identity and secrecy intact.

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

## Independent bounded acceptance — 2026-10-03

Main accepts independent GPT-6.1-sol re-verification of `d1bbdc8e8`: **ACCEPT WITH QUALIFICATIONS**. Both `unwrap_secret` calls precede validation, conversion and lookup; the ordering assertion fails against the first producer and passes now; the GC case asserts unit and numeric wrapper identity; the duration queries are unchanged. It ran the prebuilt integration binary itself: **9/9 `aura_expiration_time::` and 18/18 `aura_refresh_duration::`, both exit0**. First review SHA256 `3d2c3a16f934ca5d81b538c98dc540432ee6e84507b766e3e35a27e7aebb50ee`, re-verification SHA256 `c4e407866ef9c5d70e8e0b653cb38e2026456496db4f531c4970007a06f886f4`; both scratchpad-only.

Seven requirements are checked as bounded simulator proof. Two stay unchecked because parts are unasserted: blocked and `target` records, and immutability beyond player `(instance, expiration)` pairs. Checked means bounded development proof on the tested fixtures, not native parity.

[Page accounting](../../data/patch-api/sources/12.0.5-page-coverage.json): row365 `bounded-coverage` under new capability `aura-expiration-time-arguments`; **91 capabilities/362 ordered IDs; 150 pending/194 bounded/11 partial/7 metadata**. Startup after the reorder, `cargo check`, older profiles, aura access and restricted output remain unverified or unmodeled.

## Known gaps (current cycle)

- [ ] Not asserted by tests: blocked or `target` records, full-record and party immutability beyond player `(instance, expiration)` pairs, arguments past the second (ignored).

## Out of scope

- `RequiresUnitAuraAccess`, `RequiresValidUnitAuraInstance` and `SecretWhenUnitAuraRestricted`: unmodeled. The result is always public; no aura-access restriction state exists for this query.
- Native miss/nil/type behavior and error wording: unverified.
- `GetAuraBaseDuration` and `GetRefreshExtendedDuration` keep their existing conservative rejection of every secret argument; their cached policy is `AllowedWhenTainted` and is not part of row 365.
- Profiles below `retail-12-0-5`: `src/c_api/aura_duration.rs` is compiled and registered only under that feature, so the query does not exist there. The earlier `AllowedWhenTainted` policy is not modeled.
