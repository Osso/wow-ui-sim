# Aura expiration-time query — B86

`C_UnitAuras.DoesAuraHaveExpirationTime(auraInstanceUnit, auraInstanceID)` reports whether a stored aura record has an expiration time. The 12.0.5 [GlobalAPI source](../../data/patch-api/sources/12.0.5-api-changes.txt) lines 364–365 change its argument policy: `# SecretArguments AllowedWhenTainted -> AllowedWhenUntainted`. Source ID: `global api-C_UnitAuras-DoesAuraHaveExpirationTime-365`.

Cached retail `Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua` lines 107–125 declare nonnil `auraInstanceUnit: UnitToken`, nonnil `auraInstanceID: number`, one nonnil `hasExpirationTime: bool`, `SecretArguments = "AllowedWhenUntainted"`, `RequiresUnitAuraAccess`, `RequiresValidUnitAuraInstance` and `SecretWhenUnitAuraRestricted`. Contract context, not native execution evidence.

## What it must do

### Stored result

- [ ] Return exactly one public boolean: true when the matched stored aura has a nonzero `expiration_time`, false when it is zero.
- [ ] Resolve helpful and harmful player records and party buffs/debuffs through the existing public aura enumeration; records resolve only on their own unit and instance ID.
- [ ] Read stored state live; queries never mutate aura records. Environments are isolated.
- [ ] Unknown unit, unknown instance and nil unit return one public false. Inferred miss policy, not native-verified.
- [ ] A unit that is neither string nor nil, or an instance ID that is not a finite number, errors before lookup. Inferred representation policy.

### Argument policy under `retail-12-0-5`

- [ ] Untainted callers may pass an authentic secret unit, a secret instance ID, or both; the query resolves them like their public values and returns the public boolean.
- [ ] Tainted callers passing either argument as a secret are denied before validation or lookup, including for unknown units.
- [ ] Denial and success leave caller taint unchanged and never declassify the inputs; tainted callers keep working with public arguments.
- [ ] Rooted secret arguments survive full garbage collection with identity and secrecy intact.

## How it works

- [Lua API](../lua-api.md)
- [Audit page](../wiki/investigations/patch-12-0-5-api-audit.md)

## Implementation inventory

- `src/c_api/aura_duration.rs` — registration, argument reading and stored-record lookup shared with the duration queries.

## Tests asserting this spec

- `tests/aura_expiration_time.rs` — nine cases: stored result, party isolation, live reads, miss policy, malformed arguments, untainted secret acceptance, tainted denial, GC rooting, immutability and environment isolation.

## Known gaps (current cycle)

- [ ] Inputs only: no compiled RED, producer or GREEN recorded yet.

## Out of scope

- `RequiresUnitAuraAccess`, `RequiresValidUnitAuraInstance` and `SecretWhenUnitAuraRestricted`: unmodeled. The result is always public; no aura-access restriction state exists for this query.
- Native miss/nil/type behavior and error wording: unverified.
- `GetAuraBaseDuration` and `GetRefreshExtendedDuration` keep their existing conservative rejection of every secret argument; their cached policy is `AllowedWhenTainted` and is not part of row 365.
- Profiles below `retail-12-0-5`: the earlier `AllowedWhenTainted` policy is not modeled; those builds keep rejecting secret arguments.
