# Spell and spellbook cooldown outputs — exact rows 305/322

Batch 68 tests/spec authored October 2, 2026; bounded producer implementation follows the main-reported compiled RED at `e5d11e497`. `C_Spell.GetSpellCooldown` and `C_SpellBook.GetSpellBookItemCooldown` must implement the bounded output contract below. Source: `data/patch-api/sources/12.0.5-api-changes.txt:305,322`, each `SecretWhenSpellCooldownRestricted -> SecretWhenCooldownsRestricted`. Exact register IDs are `global api-C_Spell-GetSpellCooldown-305` and `global api-C_SpellBook-GetSpellBookItemCooldown-322`; both register entries are consolidated-final deltas and both coverage entries remain `audit-pending`, with no capabilities. The earlier batch-map row-529 attribution was incorrect; these are the two primary rows. No coverage JSON is changed here. [Lua API architecture](../wiki/systems/lua-api.md) supplies subsystem context.

Cached retail documentation corroborates function-level `SecretWhenCooldownsRestricted = true`: `SpellDocumentation.lua:268–283` and `SpellBookDocumentation.lua:215–231`. The latter says `SecretArguments = "AllowedWhenUntainted"` and documents missing/future/offspec nil results; the spell function says `AllowedWhenTainted`. `SpellSharedDocumentation.lua:19–31` lists five required fields, with `NeverSecret = true` on both booleans. These are current-cache observations, not evidence of when individual fields were introduced. Five-field book placement at the bounded 12.0.5 epoch is an explicit inference needed to complete this missing provider, not a dated native introduction claim. Function metadata does not itself prove per-field secrecy.

## What it must do

Every requirement remains unchecked: compiled pre-producer RED is recorded below, but producer GREEN, native probes and independent acceptance remain pending.

### Meaningful model before output-policy credit

- [ ] Preserve `C_Spell.GetSpellCooldown`'s existing strict numeric u32 input selection, errors and five-field interval result. Do not add public aliases or change its conservative secret-number input rejection for this output-only row.
- [ ] For the bounded Retail/PTR 12.0.5 domain, book queries must resolve the real player-bank-0 entry through the existing book resolver used by charges/durations, not interpret the slot as a spell ID or return constant disabled data. Slot 5 must be verified by an actual `GetSpellBookItemInfo(5, 0)` query to map to 19750. Alternate 642 must use an API-discovered actual slot, not invented catalog wiring.
- [ ] Both queries must project the existing spell-keyed `SpellCooldownState`, GCD and elapsed `start_time` through the current latest-ending-active-interval policy. Spell-only, GCD-only, overlap, no-cooldown and expired results must agree on concrete intervals. No new state, charge defaults, alternate provider or duration-object changes.
- [ ] Return exactly one ordinary accessible rooted table containing required `startTime`, `duration`, `modRate = 1`, public `isEnabled = true`, and public `isActive = (duration > 0)`. The existing spell getter already provides this meaningful shape. The bounded book provider must supply all five fields before output restriction earns credit; a disabled four-field constant is not an acceptable semantic model.
- [ ] Do not introduce optional `activeCategory`, `timeUntilEndOfStartRecovery` or `isOnGCD` models. Preserve the earlier/Forever book provider's disabled four-field behavior outside the shared Retail/PTR 12.0.5 gate, without changing existing profile tests.

### Inferred selector and miss policy

- [ ] Book slot must be finite, integral and within 1..=i32::MAX; bank must be numeric exactly 0. Public wrong types, nil selectors, invalid/missing slots and unsupported banks return exactly one nil under either restriction flag. This is a chosen bounded resolver policy, not full native coercion/error parity.
- [ ] An actual catalog offspec entry (20473, discovered through the real slot query and verified with `isOffSpec`) must return one nil. The shared resolver currently resolves catalog entries without excluding offspec, so reusing identity alone cannot satisfy this assertion. Future-entry nil is required but has no modeled fixture in this batch; no new catalog/state may be fabricated for it.
- [ ] Authenticate both book selectors through the actual VM AllowedWhenUntainted boundary before identity/miss resolution. Public selectors remain permitted from addon-tainted callers. Secure callers can pass actual typed secret NUM slot 5 and bank 0. Tainted secret selectors must fail even when the other selector is invalid, nil, unknown or another bank; preserve caller taint and secure recovery. Tests identify the API and argument position, assert the existing VM's untainted-caller error boundary, and do not prescribe native exception wording.

### Inferred output-field restriction

- [ ] Under the explicit live `cooldowns_restricted` flag, independently wrap `startTime`, `duration` and `modRate` as actual VM-owned secret numeric userdata with unchanged authenticated host `Val::Num` payloads. Apply this policy to active and zero/inactive intervals, including numeric 1 for modRate. Zero-field secrecy and ordinary-table placement are explicit inferences, not implied native truth from the function annotation.
- [ ] Both booleans remain public, ordinary boolean values in active and inactive results. Misses remain one nil, never fabricated restricted tables. Only unrestricted numeric outputs assert Lua `type == 'number'`; opaque restricted userdata does not gain nominal primitive-type credit.
- [ ] Restriction is scoped by the existing Retail/PTR 12.0.5 helper gate, not substituted by combat or unit-stat policy. `C_Secrets.ShouldCooldownsBeSecret` is a state sanity control, not the expected-value oracle. Fresh queries use the current flag; turning it off cannot declassify previous rooted wrappers.

### Authentication, snapshots and lifetime

- [ ] Secure and tainted public-input calls preserve their caller frames. Addon code may inspect table accessibility, field secrecy and public booleans without reading opaque numeric payloads; do not clear stack taint or replace security-query functions.
- [ ] Addon arithmetic on each authentic numeric wrapper must fail, without tested interval payload disclosure. Trusted host inspection must independently authenticate each field and verify its numeric payload, userdata identity, allocation sequence and root membership. No generic declassification or fake secret flags.
- [ ] Tainted copying through an ordinary table retains original wrapper identity and privacy. Rooted originals and copied wrappers survive fresh queries, allocation churn and forced GC; fresh results retain correct payloads and privacy.
- [ ] DTO field mutation/replacement does not modify other snapshots or source inputs. Queries remain read-only over spell/GCD intervals, clock anchor, charge inputs, action mapping and restriction flag; no consumption or fabricated charges.
- [ ] Later queries reflect interval replacement, GCD removal and explicit expiry; independent environments retain separate inputs and restriction policies. Returned snapshots remain independent of later model changes.

## How it works

- [Lua API architecture](../wiki/systems/lua-api.md)
- [Shared cooldown restriction contract](cooldown-restriction.md)
- [Existing cooldown duration selection](action-cooldown-duration.md)
- [Action output-policy precedent](action-cooldown-output-restriction.md) — simulator precedent only, not proof of native spell/book field policy.

## Implementation inventory

- `tests/spell_book_cooldown_outputs.rs`: 28 authored tests; 27 under Retail/PTR 12.0.5, one inverse-gated earlier/Forever preservation control. Uses existing grouped integration discovery; no Cargo/build/registration edits or new target.
- `src/c_api/c_spell.rs::get_spell_cooldown`: unchanged strict `u32::from_stack` parser delegates to shared `push_spell_cooldown_info`; `read_spell_cooldown_snapshot` isolates the original immutable borrow, elapsed-clock read, interval selection and restriction tuple. Publisher still roots the five-field DTO before keys/wrappers, then writes three actual typed host-secret NUMs under the existing helper gate and two public BOOLs, returning one result. Extraction changes no policy, fields, gating, parser, interval, clock, access, taint or wiring. No alias/input-policy credit. Previously main-recorded GREEN (162 passed, one ignored) and startup `0 []` predate this extraction; equivalence remains pending main's relevant refresh. No gates run for this bounded self-review.
- `src/c_api/c_spell_book.rs::cooldown_query`: sole existing `GetSpellBookItemCooldown` binding unconditionally references `cooldown_query::get`. Two alternative private inline module declarations carry the Retail/PTR 12.0.5 gate and its inverse; modern selector authentication, cooldown-only offspec resolution and getter have no individual gates. Both original selectors still authenticate through VM `unwrap_secret` before resolution; API/argument-context errors omit payloads, and the same rooted interval publisher remains in use. The inverse module preserves the old slot-only disabled four-field getter body unchanged. Module extraction equivalence remains pending main's source-scoped proof refresh; no gates or duplicate Forever compilation run for this bounded fix.
- `src/c_api/c_spell_book.rs::resolve_player_spellbook_entry`: unchanged player-bank-0 validation/mapping shared with charge/duration lookup and the modern module. `cooldown_query::resolve_cooldown_spellbook_entry` accepts authenticated Values and rejects the actual entry's typed `SkillLineData.off_spec_id`; no query-callback indirection, stack mutation or catalog change.
- `src/c_api/charge_state.rs`: existing typed-selector VM authentication precedent and explicit profile-gated cooldown restriction predicate; unchanged.
- `src/c_api/cooldown_duration.rs` and `src/lua_api/globals/action_bar_api.rs::spell_cooldown_times`: existing duration helper and latest-ending cooldown/GCD interval policy; unchanged. No ignoreGCD argument is added to the two ordinary DTO queries.

## Tests asserting this spec

`tests/spell_book_cooldown_outputs.rs` directly queries both real namespaces. Main reports compiled pre-producer execution at `e5d11e497`: 27 selected, 5 PASS / 22 FAIL. Tests and fixtures are unchanged by this producer.

| Capability | Authored coverage | Proof level |
|---|---|---|
| Meaningful public intervals | Ten separate spell/book tests for spell-only, GCD-only, overlap, zero and expiry; alternate actual slot/642 test | Five unrestricted spell controls PASS; book real-model RED; producer GREEN pending |
| Restricted intervals and shape | Both namespaces across five interval states; five required fields and public booleans; flag independence | Pre-producer numeric metadata false / book four-field disabled DTO RED; producer GREEN pending |
| Selectors and misses | Invalid/missing/offspec slots, other/nil banks, wrong types, authentic secure secret NUMs, tainted-secret denial before identity, unchanged spell u32/rejection boundary | Pre-producer book bank/parse/auth/offspec RED; producer GREEN pending |
| Privacy and lifetime | Caller preservation, arithmetic denial/recovery, actual wrapper metadata, tainted copy/forced GC, flag-off old-root privacy | Downstream opacity/root/copy/GC assertions not independently established by first failures |
| Snapshots, inputs and profiles | Mutation/replacement, read-only model, live intervals/two environments, inverse-gated legacy provider | Producer GREEN pending; earlier/Forever test excluded from current 27 |

Fixture clock is anchored at 335 seconds. Actual spell 19750 has `(312,237)`, ending 549; GCD `(330,300)` ends 630 and therefore wins overlap. Alternate 642 uses `(310,270)` with its actual dynamically discovered book slot. Expiry changes only the clock anchor to 900; no sleeps or ticks. Meaningful payload assertions precede secrecy assertions so the constant book provider cannot gain guard credit. Active margins bound the intended development run, not unlimited process lifetime. No tests require secret BOOL equality in tainted Lua or replace namespace/security functions.

### Proof ledger and ownership

This producer owns only `src/c_api/c_spell.rs`, `src/c_api/c_spell_book.rs` and this spec. No state/Cargo/tests/other API/data/wiki/PLAN/vendor changes; protected dirty source is not read, searched, hashed, formatted, staged or reverted. Only owned Rust formatting with `skip_children` and the coherent producer/spec commit are authorized here. No delegation, build, test, check, lint, readability gate, operations, push or native probe is run by this implementer.

| Command / supplied operation | Revision / scope | Result / cost | Validity |
|---|---|---|---|
| Main-reported default lib + integration compilation; exact argv not supplied | Pre-producer `e5d11e497`, combined main checkout | Exit 0, 511.6772441709 seconds, zero diagnostics | Compiled RED artifact only; changed producers require new GREEN compilation |
| Main-reported current selected execution; exact argv not supplied | Same pre-producer code; integration SHA256 `109f64e6021592d84569eeca5295aaa864b136e7fb96a13f0d8cca92e3e64ba9` | 27 selected, 5 PASS / 22 FAIL, exit 101, 5.977232970 seconds | Genuine pre-producer RED, not current producer proof |

Observed main-reported failures include book start 0 instead of 312/330/310, disabled four-field shape, bank/invalid/offspec/selector authentication, and unrestricted numeric metadata under restriction. These are meaningful model/policy boundaries; the 22 failures do not independently establish downstream opacity or GC failures. No fixture repair was needed. Main owns producer GREEN, check/startup/security/readability, independent acceptance and accounting. Requirements remain unchecked and no rows promote.

Authored test/spec date is October 2, 2026. Local Git/build-host dates are separate provenance, not an inferred source epoch or native field-introduction date.

Rows 305/322 receive no credit here. Broader coverage remains the supplied baseline of 192 pending / 155 bounded / 14 partial / 1 metadata, 362 IDs / 73 capabilities; independent batch 67 rows 301/309 are outside this artifact, and accepted batch 66 rows 237/241 are not recredited. Baseline totals are coordination context, not recomputed acceptance evidence.

## Known gaps (current cycle)

- [ ] Main must compile the committed producers and run current GREEN plus check/startup/security/readability and independent acceptance. Pre-producer compilation/RED is recorded above; book-model failures are not output-only guard failures.
- [ ] Validate real book intervals/five-field DTO, both selector authentication boundaries, cooldown-only offspec nil and numeric restriction, including downstream root/opacity/copy/GC assertions. Implementation alone earns no credit.
- [ ] Current-cache C_Spell `SpellIdentifier`/AllowedWhenTainted input policy is not implemented or credited here. Existing numeric-only parser and conservative actual-secret rejection remain explicit gaps, not new input-row requirements.
- [ ] Future entries lack a concrete existing model fixture. Pet bank/catalog support, native missing/offspec/header behaviors and full selector coercion/error parity are unproved; no opaque data-source fallback is permitted.
- [ ] Native restricted field types, zero/modRate secrecy, field-version placement, predicate conditions, access rules, exception semantics, acquisition and UI consumer behavior remain unproved. Current cached metadata is not native execution evidence.

## Out of scope

Public aliases and spell input-policy expansion; new cooldown/catalog/pet/charge state; held-cooldown modeling; duration objects or ignoreGCD changes; optional fields; fake callbacks/root templates; other runtime/registration edits; vendor changes; native/global-privacy/full-profile parity; coverage promotion, wiki/data/PLAN edits, deployment or push. No native field-introduction date is assigned from the cache.
