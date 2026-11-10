# Cooldown restriction input and charge-table secrecy

Integration checkpoint (2026-10-16): SS-P1 Retail seven-case RED → GREEN, existing charge/cooldown preservation and separate startup observation recorded in [bounded proof](../baselines/approved-batch-2026-10-16.md). No native-parity claim.

An explicit `SimState.cooldowns_restricted` input controls the zero-argument `C_Secrets.ShouldCooldownsBeSecret()` predicate and numeric secrecy on three charge tables: `C_Spell.GetSpellCharges`, `C_ActionBar.GetActionCharges`, and `C_SpellBook.GetSpellBookItemCharges`. Policy and new book-table registration require both `retail-12-0-5` and mainline (`profile-retail` or `client-ptr`). Older queries/profile defaults remain unchanged. Existing five-field charge input stays unchanged. See [charge model](../wiki/systems/spell-charge-state.md).

## What it must do

Implemented below; historical batch11/default-profile gates and [saved bounded Retail/PTR acceptance](#bounded-retailptr-selector-acceptance--2026-10-16) have separate source attribution. Broader compatibility remains unproven.

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

### Host-only numeric spell selector (bounded Retail/PTR GREEN)

- [ ] Under the existing `retail-12-0-5` mainline gate (`profile-retail` or `client-ptr`), only `C_Spell.GetSpellCharges` consumes an authentic secret numeric selector through rilua's `inspect_host_secret_number<T>(&LuaState, Val, FnOnce(f64) -> LuaResult<T>) -> LuaResult<T>`. The VM authenticates the live numeric wrapper independently of caller taint; the trusted Rust closure uses the existing `read_spell_identifier_value` alias-first resolver. Numeric aliases retain precedence over numeric identity; no new conversion or identifier-domain validation is introduced.
- [x] Keep the inspected number and resolved identifier internal to host lookup: no Lua callback, selector/payload return, logging, error interpolation, generic unwrap relaxation, or caller-taint clearing. The existing charge producer alone publishes the configured table under unchanged output secrecy; the public table and `maxCharges` remain public. Input wrapper and stored charge state remain unchanged.
- [ ] Public selectors and other secret payload kinds retain their existing resolver behavior; this is not secret-string selector support. Action/book `read_charge_selector_number` remains guarded by `unwrap_secret`/`AllowedWhenUntainted`. Charge durations, strict shared `read_public_spell_identifier_at`, and older/non-mainline profiles remain unchanged.

This is an explicitly approved simulator operation, not native identifier-domain acceptance or the full `AllowedWhenTainted` argument matrix. Consumer integration pins published rilua `ddf18d7bf13aa50f873a5792e0873c6c8a0db5a3`. Its five new tests reached actual scaffold RED: two rejection controls passed and three host-consumption cases failed. The implemented API's first GREEN build was safety-aborted for sustained host memory pressure; no test ran in that attempt. That abort remains historical. Saved source `70ebaee0e` subsequently compiled normal Retail/PTR libraries and reached [bounded consumer GREEN](#bounded-retailptr-selector-acceptance--2026-10-16). Dependency own five-test GREEN remains unproven. First/third checklist items retain open broader selector/alias/kind/profile matrices; source inspection is not exhaustive runtime or native trust-boundary proof.

### SS-P1 numeric-secret cooldown/LoC selectors (bounded Retail proof)

Approved bounded slice with observed Retail RED before producer and seven-case GREEN afterward. [Bounded PTR preservation](#ss-p1-ptr-preservation--2026-10-18) now supplies separate original-PTR consumer execution. No deployment/native-parity claim.

- [ ] Under the existing mainline `retail-12-0-5` gate, accept authentic numeric-secret selectors in `C_Spell.GetSpellCooldown`, `GetSpellChargeDuration`, `GetSpellCooldownDuration`, `GetSpellLossOfControlCooldownInfo`, and `GetSpellLossOfControlCooldownDuration`, for secure and tainted callers. Consume the selector only inside the host lookup; leave the wrapper secret, caller taint intact, and existing action/book guards unchanged.
- [ ] Preserve current producer policies, independent of selector secrecy. With `cooldowns_restricted` false/true/false, cooldown and LoC tables remain public; their timing/rate numbers follow that restriction input and retain configured payloads. Existing flags remain public. Charge/cooldown/active-LoC duration objects and their timing remain nonsecret with unchanged rate semantics. The already-integrated `GetSpellCharges` precedent is not expanded into new output exposure.
- [ ] Preserve the current alias-first resolver for charge/cooldown durations: numeric alias 900001 selects spell 19750 rather than the absent 900001 row. Do not infer alias support for the numeric-only cooldown/LoC table and LoC-duration readers, or add secret-string support, new identifier conversions, or domain/error rules.
- [ ] Preserve exact empty-result policies without a GCD for the pre-R17 SS-P1 behavior: cooldown returns one inactive enabled table (0/0/1 timing/rate, restriction still applies); cooldown duration returns one public zero-span object; charge duration and LoC info return one nil; absent LoC duration returns zero values. For R17 Retail 12.1.5/Forever duration queries, the native-analysis outcomes are INFERRED: valid unknown identifiers and absent charge systems return zero values, while supported inactive cooldown/recovery systems return a zero duration object (see [spell cooldown duration](spell-cooldown-duration.md#what-it-must-do) and `/home/osso/.local/state/wow-ui-sim/handoff/pr46-49-decomp-answers.md:2398-2477`). Secret selectors do not fabricate records.
- [ ] Execute the original cached Retail `Blizzard_ActionBar/Shared/ActionButton.lua` defaults/update/apply block unchanged against actual APIs and Cooldown widgets. Its `ActionButton_UpdateCooldown` calls cooldown before charges before LoC. Assert all three configured widget timings/rates, then LoC replacement clears normal/charge timing while retaining LoC timing. Cover unrestricted secure/tainted callers and restricted secure callers; preserve the separate restricted-tainted widget setter boundary rather than authorizing it here.

Observed bounded behavioral coverage: `tests/cooldown_restriction.rs::ssp1` (seven tests; five endpoint cases, empty-result controls, ordered original consumer). Tests inject authentic host-secret numeric wrappers into a minimal fixture; they do not establish that the live consumer normally receives secret spell IDs, native identifier acceptance, native output policies, or rendered pixels. Runtime consumer fixture requires the original Retail cache and fails explicitly if unavailable; no reconstructed fallback is supplied. Actual RED/GREEN scope and retained execution provenance are linked in the bounded proof checkpoint above.

## SS-P1 PTR preservation — 2026-10-18

Executed source `b0b2bb5193ce7085c0045e04a5fe5d4f0a3c9201`, based on `10606898e`: **PTR 8/8 GREEN**, **Retail 7/7 GREEN**, both exit0, zero failed/ignored/measured. Filter `ssp1` selects the seven named `cooldown_restriction::ssp1` cases plus PTR-only `ss_p1_ptr_preservation::original_ptr_action_button_preserves_ssp1_secret_spell_selection`; Retail selects only the same seven existing cases. Exact per-name counts and commands: `/home/osso/.local/state/wow-ui-sim/handoff/ss-p1-ptr-run/{proof-ledger.json,ptr-eight.log.names.json,retail-seven.log.names.json}`. Initial stale-harness seven-only output is superseded, not additional coverage. No behavioral RED or producer/test repair was needed.

The new case executes the unchanged cached PTR ActionButton defaults/update/apply block (SHA-256 `1c4f8e4eca223d1b277b0ad4c044b40e2116afdaf708b4ee7dceb3db78b6390e`), real APIs and real Cooldown widgets. Public/authentic numeric-secret spell19750 selects normal `(12,80,1)`, charge `(12,40,2)`, LoC `(14,24,2)`; replacement clears normal/charge while retaining LoC. Unrestricted secure/tainted and restricted secure contexts preserve selector opacity and caller taint. Restricted-tainted widget setters remain excluded.

Separate normal PTR `sound,gui,casc,client-ptr` startup exits0 with stdout byte-exact against `docs/baselines/ptr-lua-errors.json` (`[]` plus newline). `WOW_SIM_CASC`/`WOW_SIM_NO_SOUND` unset; child-only XDG storage starts with a byte-exact current persisted CVar copy; original before/after stat/hash unchanged (`startup-guards.json`). Whole-workspace fmt check and PTR package `cargo check --locked -j1` exit0 without diagnostics. Cargo/rustc1.99.0; async user units capped at9G.

Post-proof rebase onto `032a3e7d8` yields test commits `3a10bce23` + `677247c03`, byte-identical test sources. Parent explicitly waived reruns: branch changes no timed-signal or text-column files; added base changes cover timed-signal source/test/spec and PTR text-column tests/spec. Evidence remains attributed to the executed revision, not fresh post-rebase execution. No native parity, live secret-ID provenance, rendered/full-suite/all-profile acceptance, secret-string support, action/book/shared-selector widening or Retail behavior change.

## How it works

- [Shared charge model](../wiki/systems/spell-charge-state.md)
- [Duration core](duration-core.md)
- [Existing secret-value boundary](retail-secret-values.md)

## Implementation inventory

- `src/lua_api/state/sim_state.rs` and `state.rs`: pre-existing false-default explicit input; unchanged in this producer slice.
- `src/c_api/c_secrets.rs`: mainline 12.0.5 predicate registration reading only the input.
- `src/c_api/c_spell.rs`: mainline-only host inspection of authentic secret numeric charge selectors, feeding the existing alias-first resolver and unchanged charge-table producer; other selector/duration paths remain separate.
- `src/c_api/charge_state.rs`: shared rooted five-field producer, bounded output policy, narrowly scoped guarded action/book numeric selector reader; duration producer unchanged.
- `src/c_api/c_spell_book.rs`: new table query reusing the existing player slot/bank-to-spell resolution and shared charge input. Duration identity handling retains its previous public-selector semantics.
- `src/lua_api/globals/action_bar_api.rs`: only the charge-table query gains guarded exact selectors under the mainline 12.0.5 gate; older query behavior and duration handling remain unchanged. No explicit obsolete book-table provider was found to remove; generic namespace fallback is untouched.

## Tests asserting this spec

`tests/cooldown_restriction.rs`: nine cases, filter `cooldown_restriction::`, included in the existing `integration` harness; no Cargo target added. Gate: cumulative `retail-12-0-5` and mainline (`profile-retail` or `client-ptr`). Tests use actual registered queries, public model/environment exports, and existing VM helpers; no replaced providers or weakened assertions.

Actual pre-implementation RED at `84f48be77b910f5daabc2e6318c54357961d10ee`: 2 PASS / 6 FAIL, recorded in `/tmp/patch-12.0.5-batch10-run-1.log` and `/tmp/patch-12.0.5-batch10-runs.json`. Duration and no-data controls pass. Failures identify missing predicate, ordinary restricted fields, absent configured book-table result, and failed secure secret-selector resolution. Parent reports the batch compiled successfully. Earlier charge-model 10/10 GREEN does not establish this secrecy contract.

Batch11 at `d8a93bec37dc09980e41dd37b63d0dd26be88668` records **24 PASS / 0 FAIL**: policy 8, charge controls 10, ignoreGCD controls 6. `/tmp/patch-12.0.5-batch11-runs.json` binds all three logs (`run-0.log` through `run-2.log`) to integration artifact SHA-256 `a4d6874b74ceb453fe834a4e155d7ba58315f1e76c647b659631c3d478ded72f`; each exits 0. Historical startup metadata `/tmp/patch-12.0.5-batch11-startup-run.json` binds the same revision to `lua-errors`, exit 0, stdout `[]`, with saved executable SHA-256 `e3de18633351b84a94e920745cd69b7e7bce99a1c15e6d70420a9d9d9267b970`. Shared builds later replaced that target path; this run does not establish current-binary startup behavior. No rerun, native parity or whole-page completion is claimed.

`/tmp/patch-12.0.5-charge-policy-independent-proof.md` independently confirms the saved 24 PASS and integration artifact hash. `/tmp/patch-12.0.5-batch11-rust-gates.json` records `cargo fmt --check` exit 0 at `5e15752e2d466b028cb87d8cde27b710546a14bc` and default `cargo check` exit 0 from that revision through docs-only `74e6c8845f446967060f4ff49c1cbef301fafe3a`, without warnings. All 3,218 tracked Rust/config hash entries match before/after each gate; no changed paths. Audited charge production/input/fixtures remain unchanged from batch11. These are revision-scoped default gates, not new test execution or all-profile proof. Independent source review notes a 34-line `push_charge_info` readability advisory; no forced-GC stress proof.

## Known gaps (current cycle)

- [x] Independently confirmed bounded batch11 policy 8/8 and charge/ignoreGCD controls 16/16, with artifact binding; startup `[]` remains historical/hash-bound only.
- [x] Bounded numeric secret spell selector resolved by [saved Retail/PTR acceptance](#bounded-retailptr-selector-acceptance--2026-10-16); full `AllowedWhenTainted` matrix remains unproven. Historical RED: Authored `tainted_secret_spell_selector_resolves_configured_opaque_charge_table` uses the existing simulator row19750 (current1/max3/start12/duration40/rate2), explicit alias `fixture heal` and an actual host-secret numeric wrapper. A tainted call must return one configured table with public max and four opaque restricted fields, preserving caller taint/input secrecy; a later unrestricted public-alias query and stored-state control remain unchanged. This is an explicit simulator mapping, not native identifier-domain validation or parity. Actual test source `c732ea38f` against sealed normal Retail library `7273875b2`: compile0/no Rust diagnostics; **0 PASS / 1 FAIL / 8 filtered, exit101** at the configured-table assertion after public-alias, caller-taint, input-opacity and return-arity controls pass. Source explains the nil path; later output protections/recovery/state assertions remain UNREACHED. Receipt: `/home/osso/.local/state/wow-ui-sim/verification/wrath-source-current/charge-secret-selector-retail-red-20261016T130556308321Z`; independent RED/source/readability acceptance: `/home/osso/.local/state/wow-ui-sim/verification/charge-secret-selector-next/red-report.md`. This saved RED predates the approved host-only numeric operation described above. Bounded pinned-VM inspection found no public numeric opaque-input operation; guarded unwrap or shared-resolver relaxation is not a solution. The numeric inspector implementation is published and pinned as described above; consumer GREEN is now separately accepted below. The historical run retains its UNREACHED boundary; only the fresh saved runs upgrade reached coverage. No taint clearing or generic security bypass is authorized.
- [x] Bounded independent audit and revision-scoped default fmt/check gates above; not PTR/older-profile, native or whole-page acceptance.

## Bounded Retail/PTR selector acceptance — 2026-10-16

Saved source `70ebaee0e70997b691edeaf894156ca376c2ad1f` contains charge integration `3e4a887e9` and pins rilua `ddf18d7bf13aa50f873a5792e0873c6c8a0db5a3`. Independently accepted copied real modules against emitted normal libraries: **Retail 25/25 charge cases; PTR 25/25** (restriction9 + charge-duration10 + ignoreGCD6 per profile), all exit0, zero failed/ignored/measured. Retail's same bounded process also passes three [quest turn-in cases](quest-turn-in-model.md#bounded-retailforever-acceptance--2026-10-16), making 28/28; no extra selector rerun or additive duplicate counts.

`tainted_secret_spell_selector_resolves_configured_opaque_charge_table` passes once per mainline profile. It reaches the configured table, public max/table, all four opaque restricted fields with denied unwrap/arithmetic, preserved caller taint and input opacity, subsequent unrestricted public-alias recovery and unchanged stored charge state. Source `src/c_api/c_spell.rs:549–568,794–803` keeps numeric inspection inside the host lookup closure, reuses the alias-first resolver and unchanged charge producer; this is not a Lua declassification endpoint. Numeric-alias precedence, secret strings/other payload kinds and older-profile preservation are not separately established by these executions.

Evidence: `/home/osso/.local/state/wow-ui-sim/handoff/green-batch-worktree/{REPORT.md,final-reconciliation.json,batch-ledger.json,batch-ledger-retry.json}`; exact selection recipe `/home/osso/.local/state/wow-ui-sim/handoff/green-batch-recipe/README.md`; independent per-name/build-feature/snapshot/seal/CVar/startup reconciliation `/home/osso/.local/state/wow-ui-sim/verification/green-batch-20261016/report.md`. Successful component epochs under `/home/osso/.local/state/wow-ui-sim/verification/green-batch/`: `green-retail-20261016T205154411414Z`, `green-ptr-20261016T210302219940Z`. Original ledger retains aborted Retail/PTR builds; retry ledger supplies successful epochs. All nine batch seals match saved hashes/stats; normal libraries have `profile.test=false`, exactly one expected client marker and sound/gui/casc; component cfgs equal emitted features.

Separate cached-Blizzard-UI normal Retail/PTR startups both exit0 with stdout `[]`, zero Lua errors (`--no-addons --no-saved-vars`, CASC/sound disabled). All six batch original-CVar before/after hash/stat guards agree with the unchanged persisted original; initial private byte-exact copies are receipt-proven. Startup private copies persist changes, not host changes. [Quest receipt](quest-turn-in-model.md#bounded-retailforever-acceptance--2026-10-16) owns Forever's separate failing startup. Bounded batch total is 57/57 profile-case executions, 29 logical tests; not overall startup GREEN, full integration-target/native/GUI acceptance, dependency own tests, new fmt/check gates or current-HEAD proof. Historical batch11 gates above retain their original revisions.

## 12.0.1 extract follow-up

`isActive` is a public boolean computed from `maxCharges > 1`, `currentCharges < maxCharges`, positive recharge start and positive recharge duration. All three charge queries share this formula. Inactive configured charge input returns a zero-span duration object. Earlier epochs preserve their existing active-only duration behavior.

`patch_12_0_1_cooldown_charge_formula_and_zero_span` tests active, full charges, one maximum charge, zero start and zero duration with restriction enabled, through spell/action/book tables and objects. `patch_12_0_1_cooldown_cast_expiry_transitions` exercises actual cast creation and host-clock expiry of regular cooldowns. `patch_12_0_1_cached_secure_delegate_removed` exercises unchanged cached Game Lua under a tainted caller, rejecting secret numeric setters before mutation while permitting public duration objects. Numeric setters authenticate secret arguments using the VM guard on `retail-12-0-5` and later; secure accepted secrets retain private frame timing. The separate `patch_12_0_1_tainted_secret_duration_object` RED is explicitly ignored: the existing duration-object setter rejects secret-bearing objects before rendering, and the duration core lacks an opaque consumer path that preserves caller taint.

**INFERRED:** automatic charge spending/replenishment is not modeled; tests supply host transitions. Regular cooldown enablement has no hold state. LoC fields remain host-provided flags rather than derived expiration comparisons; cooldown-aura associations do not yet feed cooldown selection. These gaps prevent full hotfix closure.

## Out of scope

Other cooldown/cast-count/aura APIs outside the extract follow-up above, per-spell exceptions, automatic restriction hooks, spending/replenishment, pet/macro mappings, full secret-argument matrix, and native-client parity. No SimState, Cargo, XML, gamepad, or vendor edits in this producer slice; no push or deployment.
