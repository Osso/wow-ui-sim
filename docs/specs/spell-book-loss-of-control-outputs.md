# Spell and spellbook loss-of-control output restriction

Batch69 covers EXACT rows 313 (`C_Spell.GetSpellLossOfControlCooldownInfo`) and 326 (`C_SpellBook.GetSpellBookItemLossOfControlCooldownInfo`) only. Providers live in `src/c_api/c_spell.rs` and `src/c_api/c_spell_book.rs`; the existing `LossOfControlInfo` records are inputs. [Lua API architecture](../lua-api.md) describes the runtime boundary. Authored October 2, 2026 CDT; tests are inputs awaiting main-owned compiled RED, not passing evidence.

## What it must do

### Records and snapshots

- [ ] Both namespaces return one fresh ordinary five-field table for an explicit record: `startTime`, `duration`, `modRate`, `isActive`, `shouldReplaceNormalCooldown`. INFERRED snapshot, ordinary-table/header validation and rooting contract; GC survival is the observable rooting assertion.
- [ ] Copy the existing typed record verbatim: spell 19750 `(312,237,1.25,true,true)`, spell 642 `(11,27,0.5,true,false)`, and explicit inactive `(0,0,1,false,false)`. These are simulator fixtures, not native captures. Do not recompute from intervals, GCD, charges, time, or flags.
- [ ] Preserve spell absent-map one-nil behavior. INFERRED book policy: known valid identity with absent map also returns one nil, not the action namespace's inactive default.
- [ ] Reads observe replacements/removals without mutating records, action mappings, charges, GCD, or clock. DTO mutation/replacement leaves independent snapshots intact; environments isolate records and policy.

### Output secrecy

- [ ] `cooldowns_restricted` alone wraps the three numeric fields with authentic VM secret-number payloads, unchanged values. INFERRED per-field numeric interpretation of the function-level annotation; combat and stat restriction do not substitute for this flag.
- [ ] Both boolean fields remain public `NeverSecret`, including explicit flags inconsistent with interval-derived activity. The container remains ordinary.
- [ ] Addon public calls retain caller taint. Restricted-field arithmetic fails without exposing payloads; subsequent secure calls recover.
- [ ] Copies and forced GC retain actual userdata identity/allocation sequence, authenticated numeric payloads, and privacy. Trusted-host unwrap inspects payloads without declassifying or clearing taint. Opaque Lua nominal numeric-type parity remains UNPROVED.
- [ ] Flag-off returns a fresh public DTO without making previously rooted secrets public.

### Selector boundary

- [ ] Preserve existing spell `numeric_spell_id`: nonnegative finite numbers cast to u32 (including fractional truncation/saturation), numeric strings and case-insensitive database names resolve; unsupported types and absent map return nil. Do not replace it with strict cooldown parsing or alias parsing.
- [ ] Preserve secure/addon genuine secret spell userdata returning nil. This is a conservative existing-behavior control, NOT secret-input parity credit: cached spell documentation advertises `AllowedWhenTainted`.
- [ ] Book player bank 0 slot 5 resolves actual API-reported spell 19750. Discover spell 642's slot through `FindSpellBookSlotForSpell` and verify its actual item DTO; never fabricate a catalog.
- [ ] Authenticate both original book selectors through actual VM unwrap before public type/domain or identity checks. Secure genuine numeric slot/bank inputs resolve; addon public selectors are accepted. Addon genuine secrets in either position fail before another invalid/unresolved selector, even when the secret payload itself is invalid. INFERRED API-labelled, untainted-caller error contract and validation precedence.
- [ ] INFERRED book misses return one nil for invalid/nonintegral/nonfinite/out-of-range slots, unresolved slots, nonplayer/missing banks, and wrong public types. Typed actual offspec 20473 remains a miss even with a map record, grounded in cached offspec documentation.

### Epoch/profile control

- [ ] INFERRED placement: modern assertions run only under `retail-12-0-5` plus (`profile-retail` or `client-ptr`). The inverse control keeps the previous public spell record and disabled inactive five-field book DTO. Batch68 inverse group remains untouched pending main RED; no production placement is established by these inputs.

## How it works

- [Lua API boundary](../lua-api.md)
- [Existing action LoC contract](action-loss-control-cooldown-info.md)
- [Batch68 normal cooldown output contract](spell-book-cooldown-outputs.md)

## Implementation inventory

- `src/c_api/c_spell.rs` — existing permissive identifier parser and optional LoC map provider; unchanged by this slice.
- `src/c_api/c_spell_book.rs` — existing book provider, player resolver and gated normal `cooldown_query`; unchanged by this slice.
- `src/c_api/c_action_bar_loss_of_control.rs` — existing typed-map rooted DTO precedent; its inactive absence default is not the book contract.
- `tests/spell_book_loss_of_control_outputs.rs` — 27 modern cases and one inverse legacy control; compilation/RED pending.

## Tests asserting this spec

`tests/spell_book_loss_of_control_outputs.rs`:

| Area | Exact test names |
| --- | --- |
| Spell records/absence | `spell_public_first_record_is_verbatim`; `spell_public_second_record_keeps_false_replacement`; `spell_explicit_inactive_record_is_not_absence`; `spell_absent_map_returns_one_nil_under_both_flags` |
| Existing spell inputs | `spell_existing_permissive_public_identifier_behavior_is_preserved`; `spell_secret_userdata_stays_unmodeled_nil_for_secure_and_addon_callers` |
| Actual book records/absence | `book_public_slot_five_selects_spell_record`; `book_discovered_alternate_slot_selects_actual_642_record`; `book_explicit_inactive_record_is_not_absence`; `book_valid_identity_absent_map_returns_inferred_nil`; `book_actual_offspec_20473_is_inferred_miss_even_with_record` |
| Book public misses | `book_invalid_or_unresolved_public_slot_is_inferred_nil`; `book_nonplayer_or_missing_public_bank_is_inferred_nil`; `book_wrong_public_selector_types_are_inferred_nil` |
| Authentication | `book_secure_genuine_numeric_slot_and_bank_resolve_before_type_checks`; `book_secure_authenticated_invalid_numeric_selectors_are_inferred_nil`; `addon_public_selectors_are_allowed_without_changing_taint_or_flags`; `addon_genuine_book_secrets_deny_before_other_invalid_or_unresolved_selector` |
| Field policy/live state | `both_namespaces_copy_explicit_flags_not_interval_derived_activity`; `explicit_cooldown_flag_alone_controls_numeric_privacy`; `live_record_replacement_does_not_recompute_or_expire_payload` |
| Snapshot isolation | `dto_mutation_and_replacement_leave_other_snapshots_unchanged`; `queries_do_not_change_existing_input_maps_or_clock`; `independent_environments_do_not_share_records_or_policy` |
| Rooted secrets | `flag_off_returns_fresh_public_dto_without_declassifying_old_roots`; `addon_arithmetic_denial_preserves_wrapper_identity_and_recovery`; `addon_copies_and_forced_gc_retain_authentic_rooted_numeric_wrappers` |
| Inverse control | `earlier_profiles_keep_public_spell_and_disabled_book_loc_payloads` |

## Known gaps (current cycle)

- [ ] Main must compile and observe RED before production; no build, test, check or gate ran in this authoring slice.
- [ ] Native LoC captures, input parity, per-field numeric secrecy, missing-map semantics, validation/header precedence and epoch placement remain UNPROVED or INFERRED as labelled, not native-verified facts.
- [ ] Future-item fixtures and pet-bank behavior remain UNPROVED; no invented catalog or claim of pet support.
- [ ] Authentic wrapper metadata/payload proof does not establish native Lua nominal numeric-type parity.

### Evidence provenance

Read local retail cache under `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/` on October 2, 2026 CDT:

- `SpellDocumentation.lua:443–453`: `SecretWhenCooldownsRestricted`, `AllowedWhenTainted`, and nil for an unfound spell.
- `SpellBookDocumentation.lua:340–351`: `SecretWhenCooldownsRestricted`, `AllowedWhenUntainted`, two required selectors, and absence for nonexistent/future/offspec items.
- `SpellSharedDocumentation.lua:31–43`: five fields; both booleans explicitly `NeverSecret`.

Local docs describe annotations, not observed native execution. Existing source and adjacent tests ground simulator fixtures/policies only.

## Out of scope

Only rows313/326 test/spec inputs are authorized here. Production changes, other tests/data/wiki/PLAN, and builds/verification belong to main's next step. Future, pet, native and input-parity gaps above are evidence gaps, not exclusions from the underlying API contract.
