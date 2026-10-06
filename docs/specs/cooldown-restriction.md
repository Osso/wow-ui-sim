# Cooldown restriction input and charge-table secrecy

An explicit `SimState.cooldowns_restricted` input controls the zero-argument `C_Secrets.ShouldCooldownsBeSecret()` predicate and numeric secrecy on three charge tables: `C_Spell.GetSpellCharges`, `C_ActionBar.GetActionCharges`, and `C_SpellBook.GetSpellBookItemCharges`. Policy and new book-table registration require both `retail-12-0-5` and mainline (`profile-retail` or `client-ptr`). Older queries/profile defaults remain unchanged. Existing five-field charge input stays unchanged. See [charge model](../wiki/systems/spell-charge-state.md).

## What it must do

Implemented below; bounded batch11 execution and independent default-profile gates are recorded below. Broader compatibility remains unproven.

- [x] Default `cooldowns_restricted` to false. Read changes live, independently of combat and `unit_stats_restricted`; the predicate returns exactly one ordinary boolean.
- [x] Resolve existing public spell/action/player-book selectors to the same explicit `SpellChargeState`. Fixture spell 19750, action 17, player book slot 5/bank 0 supplies current 1, max 3, recharge start 12, base duration 40, rate 2.
- [x] With restriction false, all five numeric fields are ordinary and match input. With restriction true, wrap only concrete Rust `currentCharges`, `cooldownStartTime`, `cooldownDuration`, and `chargeModRate` numbers with `wrap_host_secret_number`; preserve payloads. `maxCharges` and the table itself remain public. Root the table before wrapper allocations. The 12.0.1 extract follow-up adds public `isActive` under the later supported `retail-12-0-5` gate.
- [x] Subsequent table queries follow false → true → false changes without modifying charge input. Retain nil for missing input, unresolved identity, or zero maximum charges; fabricate no records.
- [x] Accept tainted callers using public selectors without clearing caller taint. Restricted fields remain opaque; guarded VM inspection is available only to untainted callers.
- [x] Authenticate secret action/book selectors using VM `unwrap_secret`, even for secure callers, before exact numeric identity resolution. Tainted public selectors are accepted; tainted secret selectors are rejected. Authenticate both book selectors before resolving identity, including invalid slot/bank combinations.
- [x] Leave charge duration objects and timing unchanged and nonsecret under either restriction input, including tainted public-selector consumers. Existing rate semantics preserve start 12, total 20 (base 40), rate 2, end 32. This bounded exclusion follows absent return-secret annotations, not native parity.

### Source grounding

Retained [12.0.5 register](../../data/patch-api/sources/12.0.5-register.json) IDs:

- `global api-C_ActionBar-GetActionCharges-231`
- `global api-C_Spell-GetSpellCharges-303`
- `global api-C_SpellBook-GetSpellBookItemCharges-320`

Each changes the former action/spell restriction annotation to `SecretWhenCooldownsRestricted`. The predicate is also retained in [12.0.0 register](../../data/patch-api/sources/12.0.0-register.json), symbol `C_Secrets.ShouldCooldownsBeSecret` (added API, null inputs, one boolean output); its [audit identifier](../../data/patch-api/12.0.0.json) is `added:C_Secrets.ShouldCooldownsBeSecret`.

Exact local source directory: `/home/osso/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/`:

- `SecretPredicateAPIDocumentation.lua:129–131`: predicate name/function/documentation; following Returns declares one nonnil boolean and no Arguments block.
- `SecretPredicatesDocumentation.lua:89–92`: restriction annotation includes combat/encounter/challenge/PvP contexts and individual spell exceptions. This slice exposes an explicit input, not automatic native restriction hooks or exception inference.
- `SpellSharedDocumentation.lua:6–15`: numeric charge fields; `maxCharges` and the newer `isActive` are `NeverSecret`. `isActive` was excluded from the original restriction slice; the 12.0.1 extract follow-up below supplies it.
- `SpellDocumentation.lua:250–254`, `ActionBarFrameDocumentation.lua:138–142`, `SpellBookDocumentation.lua:196–200`: all three queries have restricted output; spell secret arguments are `AllowedWhenTainted`, action/book are `AllowedWhenUntainted`.

### Inferences and native-probe boundary

Wrapping all four unannotated numbers whenever the explicit restriction input is true is a bounded inference; native per-spell exceptions are not modeled. Player-bank-only lookup, nil for invalid/unmodeled identity, exact integer slot validation, and VM rejection details are simulator behavior, not native-verified acceptance/error semantics. Future native probes should compare each field under restricted/unrestricted contexts and secure/tainted action, slot, and bank selectors. No native probe is claimed.

`C_Spell.GetSpellCharges` secret spell identifiers remain a separate unresolved opaque-selector operation. `AllowedWhenTainted` does not authorize generic host decoding, declassification, or caller-taint clearing. This implementation deliberately does not change spell identifier handling or claim the full argument matrix.

## How it works

- [Shared charge model](../wiki/systems/spell-charge-state.md)
- [Duration core](duration-core.md)
- [Existing secret-value boundary](retail-secret-values.md)

## Implementation inventory

- `src/lua_api/state/sim_state.rs` and `state.rs`: pre-existing false-default explicit input; unchanged in this producer slice.
- `src/c_api/c_secrets.rs`: mainline 12.0.5 predicate registration reading only the input.
- `src/c_api/charge_state.rs`: shared rooted five-field producer, bounded output policy, narrowly scoped guarded action/book numeric selector reader; duration producer unchanged.
- `src/c_api/c_spell_book.rs`: new table query reusing the existing player slot/bank-to-spell resolution and shared charge input. Duration identity handling retains its previous public-selector semantics.
- `src/lua_api/globals/action_bar_api.rs`: only the charge-table query gains guarded exact selectors under the mainline 12.0.5 gate; older query behavior and duration handling remain unchanged. No explicit obsolete book-table provider was found to remove; generic namespace fallback is untouched.

## Tests asserting this spec

`tests/cooldown_restriction.rs`: eight cases, filter `cooldown_restriction::`, included in the existing `integration` harness; no Cargo target added. Gate: cumulative `retail-12-0-5` and mainline (`profile-retail` or `client-ptr`). Tests use actual registered queries, public model/environment exports, and existing VM helpers; no replaced providers or weakened assertions.

Actual pre-implementation RED at `84f48be77b910f5daabc2e6318c54357961d10ee`: 2 PASS / 6 FAIL, recorded in `/tmp/patch-12.0.5-batch10-run-1.log` and `/tmp/patch-12.0.5-batch10-runs.json`. Duration and no-data controls pass. Failures identify missing predicate, ordinary restricted fields, absent configured book-table result, and failed secure secret-selector resolution. Parent reports the batch compiled successfully. Earlier charge-model 10/10 GREEN does not establish this secrecy contract.

Batch11 at `d8a93bec37dc09980e41dd37b63d0dd26be88668` records **24 PASS / 0 FAIL**: policy 8, charge controls 10, ignoreGCD controls 6. `/tmp/patch-12.0.5-batch11-runs.json` binds all three logs (`run-0.log` through `run-2.log`) to integration artifact SHA-256 `a4d6874b74ceb453fe834a4e155d7ba58315f1e76c647b659631c3d478ded72f`; each exits 0. Historical startup metadata `/tmp/patch-12.0.5-batch11-startup-run.json` binds the same revision to `lua-errors`, exit 0, stdout `[]`, with saved executable SHA-256 `e3de18633351b84a94e920745cd69b7e7bce99a1c15e6d70420a9d9d9267b970`. Shared builds later replaced that target path; this run does not establish current-binary startup behavior. No rerun, native parity or whole-page completion is claimed.

`/tmp/patch-12.0.5-charge-policy-independent-proof.md` independently confirms the saved 24 PASS and integration artifact hash. `/tmp/patch-12.0.5-batch11-rust-gates.json` records `cargo fmt --check` exit 0 at `5e15752e2d466b028cb87d8cde27b710546a14bc` and default `cargo check` exit 0 from that revision through docs-only `74e6c8845f446967060f4ff49c1cbef301fafe3a`, without warnings. All 3,218 tracked Rust/config hash entries match before/after each gate; no changed paths. Audited charge production/input/fixtures remain unchanged from batch11. These are revision-scoped default gates, not new test execution or all-profile proof. Independent source review notes a 34-line `push_charge_info` readability advisory; no forced-GC stress proof.

## Known gaps (current cycle)

- [x] Independently confirmed bounded batch11 policy 8/8 and charge/ignoreGCD controls 16/16, with artifact binding; startup `[]` remains historical/hash-bound only.
- [ ] Secret spell identifier `AllowedWhenTainted` remains unresolved and untested here; guarded generic unwrap does not solve it.
- [x] Bounded independent audit and revision-scoped default fmt/check gates above; not PTR/older-profile, native or whole-page acceptance.

## 12.0.1 extract follow-up

`isActive` is a public boolean computed from `maxCharges > 1`, `currentCharges < maxCharges`, positive recharge start and positive recharge duration. All three charge queries share this formula. Inactive configured charge input returns a zero-span duration object. Earlier epochs preserve their existing active-only duration behavior.

`patch_12_0_1_cooldown_charge_formula_and_zero_span` tests active, full charges, one maximum charge, zero start and zero duration with restriction enabled, through spell/action/book tables and objects. `patch_12_0_1_cooldown_cast_expiry_transitions` exercises actual cast creation and host-clock expiry of regular cooldowns. `patch_12_0_1_cached_secure_delegate_removed` exercises unchanged cached Game Lua under a tainted caller, rejecting secret numeric setters while permitting real duration objects.

**INFERRED:** automatic charge spending/replenishment is not modeled; tests supply host transitions. Regular cooldown enablement has no hold state. LoC fields remain host-provided flags rather than derived expiration comparisons; cooldown-aura associations do not yet feed cooldown selection. These gaps prevent full hotfix closure.

## Out of scope

Other cooldown/cast-count/aura APIs outside the extract follow-up above, per-spell exceptions, automatic restriction hooks, spending/replenishment, pet/macro mappings, full secret-argument matrix, and native-client parity. No SimState, Cargo, XML, gamepad, or vendor edits in this producer slice; no push or deployment.
