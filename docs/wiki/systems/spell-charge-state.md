# Spell charge state

Bounded explicit input for shared spell/action/book charge queries. Contract and native-probe caveats live in [spell charge state](../../specs/spell-charge-state.md); this page records representation and evidence, not native progression.

## Input and resolution

`SimState.spell_charges` is an initially empty `HashMap<u32, SpellChargeState>`. The public row stores current/max counts, recharge start/base duration and rate. Seed fixtures through `env.state().borrow_mut().spell_charges.insert(spell_id, row)`. An ordinary cooldown, GCD or action assignment is not charge input.

Grouped fixtures bind spell 19750 to action 17 and seeded alias `fixture heal`; existing player spellbook slot 5/bank 0 resolves that spell. `SimState.start_time` provides the clock: `Instant::now() - Duration::from_secs(20)` gives the concrete initial observation. Varied snapshots remap alias/action to spell 54321 and move the clock to 50 without mutating old duration objects.

## Shared producers and provider evidence

Actual no-data RED at `e0a46d691` passes the spell-duration nil control and fails three fabricated action/table cases. The passing spell control is the lazy `__wow_namespace_mt.__index` nil-returning closure (`runtime_surface_bootstrap.lua:65–76`), not a modeled charge producer. The exact static `C_Spell.GetSpellCharges` table fallback is in `workarounds/temporary/spell_static_defaults.rs`. `13af6a6b2` installs authoritative charge registrations and deletes the exact static charge-table fallback and action empty-object producer. Unrelated namespace/default behavior remains untouched.

`d1f2474e5` adds type/input and concrete fixtures, `91a0bfb55` strengthens nonnil checks, and `75434f23d` adds direct zero-span/zero-maximum cases plus epoch-aware duration-core controls. Actual concrete RED at `eac08bda3` is 1/10 pass, 9 expected failures, including the shared zero-span failure (`/tmp/patch-12.0.5-batch6-charge-red.log`). `13af6a6b2` then wires all three producers to the same explicit map and existing identity paths. Table queries and the already-published action duration query are globally modeled; new spell/book duration registrations and max-charge zero-span behavior require cumulative `retail-12-0-5`. Earlier maximum-charge action duration has no active interval (inference), not an empty fallback.

Rows with zero maximum are not configured charge spells. Below maximum, the shared selector snapshots configured start/base duration/rate; at maximum on 12.0.5+, it uses query time and zero base duration. The rate-aware constructor calls existing `SetTimeFromStart` validation. No charge transitions are synthesized. Shared core zero-span `HasExpired` and elapsed fraction change under 12.0.5 only; Started/Active remain unchanged. Parent-batched GREEN and final gates are pending; no Cargo ran in the implementation child.

## Sources

- [Charge contract](../../specs/spell-charge-state.md) — requirements, publication gates, inferences and proof.
- [Duration core contract](../../specs/duration-core.md) — existing validation/time/rate and 12.0.5 zero-span policy.

## See Also

- [[duration-core]] — shared duration snapshot semantics.
- [[patch-12-0-0-api-audit]] — historical placeholder audit, not current producer proof.
