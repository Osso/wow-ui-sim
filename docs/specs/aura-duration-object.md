# Aura duration object query — B93

`C_UnitAuras.GetAuraDuration(auraInstanceUnit, auraInstanceID)` returns a duration object for a stored aura. The 12.0.5 [GlobalAPI source](../../data/patch-api/sources/12.0.5-api-changes.txt) lines 379–380 change its argument policy: `# SecretArguments AllowedWhenTainted -> AllowedWhenUntainted`. Source ID: `global api-C_UnitAuras-GetAuraDuration-380`. Before this work the simulator did not register the function at all.

Cached retail `UnitAuraDocumentation.lua` lines 265–282 declare nonnil `auraInstanceUnit: UnitToken`, nonnil `auraInstanceID: number`, one nonnil `duration: LuaDurationObject`, `SecretArguments = "AllowedWhenUntainted"`, `RequiresUnitAuraAccess` and `RequiresValidUnitAuraInstance`. No cached Blizzard UI file calls it. Contract context, not native execution evidence.

## What it must do

### Snapshot

- [ ] Return exactly one new duration object whose start is the stored `expiration_time - duration`, whose span is the stored `duration`, at rate 1, for helpful and harmful player records and party records resolved through the existing public aura enumeration.
- [ ] A permanent record (zero duration and expiration) yields a zero-span object. Inferred policy.
- [ ] Each call reads stored state live and returns an independent object: earlier results keep their values, and mutating a result changes neither stored state nor later results.
- [ ] An unknown unit, an unknown or cross-unit instance, a nil or missing unit, or a missing, non-numeric or non-finite instance ID raises an error; there is no nil result. Inferred from the nonnil return and `RequiresValidUnitAuraInstance`.

### Argument policy

- [ ] Untainted callers may pass an authentic secret unit, a secret instance ID, or both.
- [ ] Tainted callers passing either argument as a secret are denied before validation or lookup, including for unknown units and with a malformed public unit; both arguments are authenticated before either is validated. Caller taint and input secrecy are unchanged; tainted callers keep working with public arguments.

## How it works

- [Aura expiration-time query](aura-expiration-time.md) — the shared two-argument reader.
- [Duration core](duration-core.md) — the object returned.

## Implementation inventory

- `src/c_api/aura_duration.rs` — registration and producer, compiled only under `retail-12-0-5`.
- `src/lua_api/globals/lua_duration_object.rs` — `push_timed_duration_object`.

## Tests asserting this spec

- `tests/aura_duration_object.rs` — six cases: timed spans, permanent zero span, independent live snapshots, invalid arguments, untainted secret acceptance, tainted denial ordering.

## Known gaps (current cycle)

- [ ] Development proof: inputs `b9b9eeec8` RED 0 PASS / 6 FAIL (the name resolved to a nil-returning placeholder); producer GREEN 33/33 across `aura_duration_object::`, `aura_expiration_time::`, `aura_refresh_duration::`, cargo exit0, no warnings, `cargo fmt --check` exit0. Independent verification and accounting pending.

## Out of scope

- `RequiresUnitAuraAccess` and any restricted or secret duration output: unmodeled; the object is always public.
- Blocked records and `target`: the public enumeration drops blocked instances, so they error here; `target` resolves a fixed fixture list. Neither is asserted.
- Rate modifiers, haste-scaled durations, native error wording and behavior for invalid instances: unverified.
