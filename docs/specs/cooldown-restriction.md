# Cooldown restriction input and charge-table secrecy

An explicit `SimState.cooldowns_restricted` input controls the zero-argument `C_Secrets.ShouldCooldownsBeSecret()` predicate and numeric secrecy on three charge tables: `C_Spell.GetSpellCharges`, `C_ActionBar.GetActionCharges`, and `C_SpellBook.GetSpellBookItemCharges`. This slice adds input scaffolding and unexecuted tests only; predicate registration, secret-output production, and the absent book-table producer remain parent-owned after actual RED. Existing five-field charge input stays unchanged. See [charge model](../wiki/systems/spell-charge-state.md).

## What it must do

- [ ] Default `cooldowns_restricted` to false. Read changes live, independently of combat and `unit_stats_restricted`; the predicate returns exactly one ordinary boolean.
- [ ] Resolve existing public spell/action/player-book selectors to the same explicit `SpellChargeState`. Fixture spell 19750, action 17, player book slot 5/bank 0 supplies current 1, max 3, recharge start 12, base duration 40, rate 2.
- [ ] With restriction false, all five numeric fields are ordinary and match input. With restriction true, `currentCharges`, `cooldownStartTime`, `cooldownDuration`, and `chargeModRate` are opaque native secret numbers with unchanged payloads; `maxCharges` remains public.
- [ ] Subsequent table queries follow false → true → false changes without modifying charge input. No spell/action/book charge record is fabricated for missing input or unresolved identity, under either policy.
- [ ] Tainted callers using public selectors receive opaque restricted fields, cannot unwrap or perform arithmetic on them, and retain their original taint. A secure host may inspect known payloads only through the existing guarded VM helper.
- [ ] Secret action/book selectors resolve for untainted callers and reject tainted callers. Exercise action, book slot, book bank, and both book selectors separately from output restriction. These tests specify the documented argument policy; native acceptance/error details remain unverified.
- [ ] Charge duration objects and timing remain nonsecret with restriction true or false, including calls from tainted public-selector consumers. Existing rate semantics preserve start 12, total 20 (base 40), rate 2, end 32. This is bounded exclusion policy from absent return-secret annotations, not native parity.

### Source grounding

Retained [12.0.5 register](../../data/patch-api/sources/12.0.5-register.json) IDs:

- `global api-C_ActionBar-GetActionCharges-231`
- `global api-C_Spell-GetSpellCharges-303`
- `global api-C_SpellBook-GetSpellBookItemCharges-320`

Each changes the former action/spell restriction annotation to `SecretWhenCooldownsRestricted`. The predicate is also retained in [12.0.0 register](../../data/patch-api/sources/12.0.0-register.json), symbol `C_Secrets.ShouldCooldownsBeSecret` (added API, null inputs, one boolean output); its [audit identifier](../../data/patch-api/12.0.0.json) is `added:C_Secrets.ShouldCooldownsBeSecret`.

Exact local source directory: `/home/osso/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/`:

- `SecretPredicateAPIDocumentation.lua:129–131`: predicate name/function/documentation; following Returns declares one nonnil boolean and no Arguments block.
- `SecretPredicatesDocumentation.lua:89–92`: restriction annotation includes combat/encounter/challenge/PvP contexts and individual spell exceptions. This slice exposes an explicit input, not automatic native restriction hooks or exception inference.
- `SpellSharedDocumentation.lua:6–15`: numeric charge fields; `maxCharges` and the newer `isActive` are `NeverSecret`. `isActive` is deliberately not added to this existing five-field model.
- `SpellDocumentation.lua:250–254`, `ActionBarFrameDocumentation.lua:138–142`, `SpellBookDocumentation.lua:196–200`: all three queries have restricted output; spell secret arguments are `AllowedWhenTainted`, action/book are `AllowedWhenUntainted`.

## How it works

- [Shared charge model](../wiki/systems/spell-charge-state.md)
- [Duration core](duration-core.md)
- [Existing secret-value boundary](retail-secret-values.md)

## Implementation inventory

- `src/lua_api/state/sim_state.rs`: public explicit boolean input.
- `src/lua_api/state.rs`: false default, no derived policy.
- `src/c_api/charge_state.rs`: existing public five-field row and plain table producer; unchanged by this slice.
- `tests/cooldown_restriction.rs`: grouped actual-query fixtures; no replaced API providers.

## Tests asserting this spec

`tests/cooldown_restriction.rs`: eight cases, filter `cooldown_restriction::`, automatically included in the existing `integration` harness; no Cargo target added. Gate: cumulative `retail-12-0-5` and mainline (`profile-retail` or `client-ptr`). Public model/environment exports and existing VM helpers are used directly.

No build, test, check, or actual RED executed in this input-only slice. Expected failure boundaries are missing predicate, ordinary spell/action restricted fields, absent configured book charge table, and currently unsupported secret action/book selector handling. Duration/no-data controls may already pass; no pass count is claimed. Parent must observe actual RED before producer changes. Earlier charge-model 10/10 GREEN does not establish this secrecy contract.

## Known gaps (current cycle)

- [ ] Parent-owned actual RED for these eight cases, followed by scoped predicate/table/book producer implementation and GREEN.
- [ ] Secret spell identifier `AllowedWhenTainted` remains unresolved and untested here. It may require a separate opaque identifier operation; guarded generic unwrap does not solve it. No broad unsafe unwrap, caller-taint clearing, or invented bypass is authorized.
- [ ] Final compilation, format/check and independent verification remain parent-owned; this commit promises input/test scaffolding, not working secrecy.

## Out of scope

- Other cooldown/cast-count/aura APIs, new `isActive` output, per-spell exceptions, automatic native restriction hooks, spending/replenishment, and pet/macro mappings: separate evidence/model scope.
- Full secret-argument matrix and native probes: native provenance/acceptance is unknown; do not claim this slice solves spell opaque identifiers.
- Vendor/XML changes, provider replacement in tests, production API-handler edits, push, deployment, and native-client parity.
