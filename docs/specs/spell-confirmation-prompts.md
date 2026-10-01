# Spell confirmation prompts

Bounded Mainline retail 12.0.5+ pending-prompt input, query and actions (`retail-12-0-5` plus `profile-retail` or `client-ptr`). `A_Admin.QueueSpellConfirmationPrompt(record)` is an explicit simulator input, not a native WoW API. Fixtures use actual registered APIs, never generic event injection or test-time replacements. [Audit context](../wiki/investigations/patch-12-0-5-api-audit.md).

## What it must do

- [x] Begin with no pending prompts; `GetSpellConfirmationPromptsInfo()` returns exactly one empty table. Empty state is simulator policy, not a native-default claim.
- [x] Accept a complete caller-supplied record through `A_Admin.QueueSpellConfirmationPrompt(record)`. All ten fields below are explicit and nonnil; do not fabricate spell, item, currency, difficulty or timing values.
- [x] Retain the record before synchronously dispatching exactly ten `SPELL_CONFIRMATION_PROMPT` arguments. A real frame listener querying pending state during dispatch must see the new record before the admin call returns.
- [x] Retain two different spell IDs independently, with all supplied values preserved. No native query ordering is asserted.
- [x] Infer pending lifecycle only: accept/decline remove only the matching spell ID; absent IDs do nothing; repeated queue input for one spell ID replaces its entire record without adding a duplicate.
- [x] Return independent record/sequence snapshots and copy caller input; mutations cannot change retained data, earlier snapshots, subsequent queries or another environment's state.

### Source-grounded event/query mapping

Cached `Blizzard_APIDocumentationGenerated/UnitDocumentation.lua:3935–3951` declares `SynchronousEvent = true` and ten nonnil event fields. Cached `Blizzard_Game/Mainline/EventImplementation.lua:389–402` binds these arguments directly to the query-style names; argument 2 becomes `confirmType` with **no conversion**. The handler compares that value to `Enum.ConfirmationPromptUIType` and routes BonusRoll's final three arguments directly to `BonusRollFrame_StartBonusRoll`.

| Event position | Declared event name | Caller input / query field | Type |
| --- | --- | --- | --- |
| 1 | `spellID` | `spellID` | number |
| 2 | `effectValue` | `confirmType` | number |
| 3 | `message` | `text` | cstring |
| 4 | `duration` | `duration` | number |
| 5 | `currencyTypesID` | `currencyID` | number |
| 6 | `currencyCost` | `currencyCost` | number |
| 7 | `currentDifficulty` | `difficultyID` | number |
| 8 | `displayItemID` | `displayItemID` | number |
| 9 | `itemContext` | `itemContext` | number |
| 10 | `treasureContextLevel` | `treasureContextLevel` | number |

Cached `Blizzard_Game/Mainline/EventImplementation.lua:804–818` consumes the pending-query sequence with exactly these record fields on entering world. This establishes consumer expectations, not a generated query return declaration. Cached `SpellConstantsDocumentation.lua:13–17` supplies enum values used by the fixtures: BonusRoll `1`, StaticTextAlert `3`, SimpleWarningAlert `4`.

Retained patch clause: `data/patch-api/sources/12.0.5-api-changes.txt:559–562` appends event fields 8–10. Fixtures use distinct concrete `220081`, `23`, `71` in those positions, plus a second independent record and a replacement with different values. These are caller-supplied fixture values, not real gameplay records.

### Evidence limits

Cached sources ground event names, positions, nilability, synchronous designation, direct field mapping and query consumer shape. **Inferred simulator policy:** empty initial state, one record per spell ID, replacement, matching-ID action removal, absent-ID no-op, copying and per-environment ownership. Native action results, validation/rejection rules, query ordering, duplicate handling, input triggers, duration interpretation and state publication ordering have not been probed. State-before-listener is an explicit simulator contract, not native-verified behavior. Actions take `spellID` as requested for this bounded contract; no native signature or return-value claim follows.

## How it works

- [Lua API and environment state](../lua-api.md).
- [Synchronous frame event dispatch](../event-system.md).
- [Patch audit evidence boundaries](../wiki/investigations/patch-12-0-5-api-audit.md).

## Implementation inventory

- `tests/spell_confirmation_prompts.rs`: six independently audited passing behavioral fixtures, gated to `profile-retail` + `retail-12-0-5`; discovered by the existing generated integration harness, no new Cargo target.
- `src/lua_api/globals/real/spell_confirmation_prompts.rs`: host-owned complete records, admin parsing/publication, snapshot query and matching-ID actions. Numeric `spellID` keys preserve finite values without integer truncation; signed zero shares one identity.
- `src/lua_api/state/sim_state.rs` and `state.rs`: per-environment pending map, empty by default, under the same Mainline epoch/profile gate.
- `src/lua_api/globals/{real/mod.rs,register.rs,admin.rs}`: gated public globals and admin input wiring.
- `src/lua_api/workarounds/temporary/inert_global_defaults.rs`: exact inert query moved to a separately cfg-selected bootstrap for earlier/nonmainline profiles; modeled profiles never execute that fallback. Runtime source search found no pre-existing accept/decline stub registrations to supersede; those profiles retain their prior action surface.
- `src/event/valid_events_c.rs`: existing event-name registration, unchanged.

Parsing requires every numeric field to be a finite public number and text to be a public UTF-8 string; these are host validation policies, not native rejection claims. Secured input tables obey VM access checks. Opaque secret values are rejected without decoding or disclosure. Parsing and host copying finish before pending state changes. Query tables and event text are stack-rooted across allocations/callbacks; no simulator borrow crosses dispatch.

## Tests asserting this spec

`tests/spell_confirmation_prompts.rs` contains six cases:

| Case | Observable contract |
| --- | --- |
| `empty_default_and_absent_actions_keep_no_pending_prompts` | Callable actions checked before absent-ID calls; exact one empty-table query result |
| `queue_publishes_exact_ten_arguments_and_query_state_synchronously` | Actual admin transition; exact ten nonnil values; listener queries complete post-state synchronously |
| `two_pending_spell_ids_retain_independent_complete_records` | Two complete records without ordering assumptions |
| `accept_and_decline_remove_only_matching_pending_spell` | Both actions preserve the other record; absent-ID calls preserve pending state |
| `repeated_spell_id_replaces_one_record_without_duplicate` | Full replacement with distinct values, unrelated record retained |
| `input_and_query_mutations_do_not_cross_snapshots_or_environments` | Caller/query mutation isolation, retained snapshots and separate environments |

Proof ledger (2026-10-01): tests/spec `6f7427fef41267f39abc24a032e56e63bf41ad8f` compiled in the parent's saved 127.27s build (`/tmp/patch-12.0.5-batch15-red-build.json`, `.log`, and result artifact). Actual selected RED: 0/6, exit 101, 1.44s; `/tmp/patch-12.0.5-batch15-red-run.json` and `.log` identify that revision and binary hash. Failures stop at missing modeled surface prerequisites, not six independent reached behavior failures. Production `b07fc61f6`, docs/compiled snapshot `329eabfbb`: independent [saved proof](/tmp/patch-12.0.5-spell-prompts-independent-proof.md) confirms six prompt cases plus eight party and four Recent Allies controls PASS. GREEN compile exits 0; default fmt/check exit 0 at `81421d057`, with twelve scoped hashes unchanged and later quest fixtures excluded from the GREEN binary. Security, validation and rooting are source-inspected only; no invalid/secret/secured-input, forced-GC, reentrant-action or all-profile execution proof.

Separate parent startup `/tmp/patch-12.0.5-batch15-green-startup-run.json`, `-startup.json` and `-startup.log`: exit 0, `[]`, zero Lua errors, 46.39s; normal wow-sim SHA256 `bd2be508b9061dd7ca1267c404942e5aa52bb352ca405c52c7e2f296977accd3`. Artifact was emitted by `/tmp/patch-12.0.5-batch16-red-build.json`. That combined command exits **101**: unrelated bin wow-sim test compilation fails at `enable_state.rs:225` because `AddonMetadata.addon_dir` is missing. Normal binary and integration executable were emitted successfully; startup does not erase that failure. Startup is parent evidence, not an independent-verifier claim.

## Known gaps (current cycle)

- [x] Parent records actual targeted RED and its revision before production implementation.
- [x] Obtain actual GREEN and required independent gates for the implemented bounded input/state/query/actions and replaced empty-query provider.
- [ ] Native semantics remain unknown as listed above; do not upgrade inferred lifecycle choices to native parity.

## Out of scope

Gameplay rewards, currency deduction, real spell/item triggers, automatic expiration/timing and `SPELL_CONFIRMATION_TIMEOUT`. They require additional gameplay/lifecycle contracts, not invented effects for a pending-record fixture. Native missing/invalid-field rejection rules and action return values are unspecified; no rejection fixtures claim native semantics. No vendor edits, new all-profile behavior, whole-page audit completion or native parity claim.
