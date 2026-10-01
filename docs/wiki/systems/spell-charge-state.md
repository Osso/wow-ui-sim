# Spell charge state

Bounded explicit input for shared spell/action/book charge queries. Contract and native-probe caveats live in [spell charge state](../../specs/spell-charge-state.md); this page records representation and evidence, not native progression.

## Input and resolution

`SimState.spell_charges` is an initially empty `HashMap<u32, SpellChargeState>`. The public row stores current/max counts, recharge start/base duration and rate. Seed fixtures through `env.state().borrow_mut().spell_charges.insert(spell_id, row)`. An ordinary cooldown, GCD or action assignment is not charge input.

Grouped fixtures bind spell 19750 to action 17 and seeded alias `fixture heal`; existing player spellbook slot 5/bank 0 resolves that spell. `SimState.start_time` provides the clock: `Instant::now() - Duration::from_secs(20)` gives the concrete initial observation. Varied snapshots remap alias/action to spell 54321 and move the clock to 50 without mutating old duration objects.

## Provider evidence and pending wiring

Actual no-data RED at `e0a46d691` passes the spell-duration nil control and fails three fabricated action/table cases. The passing spell control is the lazy `__wow_namespace_mt.__index` nil-returning closure (`runtime_surface_bootstrap.lua:65–76`), not a modeled charge producer. The exact static `C_Spell.GetSpellCharges` table fallback is in `workarounds/temporary/spell_static_defaults.rs`. Authoritative registrations must replace these exact charge providers; unrelated namespace/default behavior remains untouched.

`d1f2474e5` adds type/input and concrete fixtures, `91a0bfb55` strengthens nonnil checks, and `75434f23d` adds direct zero-span/zero-maximum cases plus epoch-aware duration-core controls. Concrete fixture RED and producers are pending parent batching. No Cargo ran in the implementation child.

## Sources

- [Charge contract](../../specs/spell-charge-state.md) — requirements, publication gates, inferences and proof.
- [Duration core contract](../../specs/duration-core.md) — existing validation/time/rate and 12.0.5 zero-span policy.

## See Also

- [[duration-core]] — shared duration snapshot semantics.
- [[patch-12-0-0-api-audit]] — historical placeholder audit, not current producer proof.
