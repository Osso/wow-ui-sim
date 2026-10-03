# B96 independent verification

Revision: a0e23199dc60d4056add5f5a15b68f23dcac38cc. HEAD matches; git diff a0e23199d HEAD -- src tests and working src/tests diff both empty (exit 0). No repository edits, cargo, agents, or model CLIs.

Static review and permitted prebuilt tests in progress.

## Command proof voice_chat_speak_text::

Prebuilt integration-8ea324359263a4d2, --test-threads=1: 8 passed; 0 failed; 0 ignored; 0 measured; 10153 filtered out; 0.74s; exit 0; stderr empty. Full output observed in tool response. Logging variable error occurred after successful test; test not rerun.

## Command proof click_binding_spell_identifier::

```

running 9 tests
test click_binding_spell_identifier::authentic_secret_identifiers_reject_after_gc_in_secure_and_tainted_calls ... ok
test click_binding_spell_identifier::empty_eligibility_preserves_default_interaction_profile ... ok
test click_binding_spell_identifier::existing_spell_profile_does_not_declare_eligibility ... ok
test click_binding_spell_identifier::explicit_membership_resolves_names_and_miss_controls ... ok
test click_binding_spell_identifier::full_link_alias_is_explicit_not_parsed ... ok
test click_binding_spell_identifier::membership_and_named_alias_mutations_are_live_and_environment_local ... ok
test click_binding_spell_identifier::numeric_alias_precedence_and_removal_are_live ... ok
test click_binding_spell_identifier::public_tainted_calls_preserve_taint_and_boolean_contract ... ok
test click_binding_spell_identifier::strict_public_identifiers_validate_before_alias_resolution ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 10152 filtered out; finished in 0.83s


```
Exit: 0

## Command proof encounter_event_sound_no_error::

```

running 4 tests
test encounter_event_sound_no_error::addon_sound_lookup_does_not_error_or_change_stack_taint ... ok
test encounter_event_sound_no_error::configured_event_sound_returns_stored_fields_and_one_result ... ok
test encounter_event_sound_no_error::missing_sound_overrides_return_no_values_without_error ... ok
test encounter_event_sound_no_error::returned_sound_table_mutation_does_not_change_stored_override ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 10157 filtered out; finished in 0.36s


```
Exit: 0

## Final verdicts

### 1. C_VoiceChat.SpeakText — ACCEPT WITH QUALIFICATIONS

8/8 tests passed, exit 0. Genuine state-backed request recording, not a wrapped no-op: src/c_api/c_voice_chat_speak.rs:28–34 validates then appends; SimState field at src/lua_api/state/sim_state.rs:470, empty default at src/lua_api/state.rs:443. Registration at src/lua_api/globals/register.rs:87 is effective in the tested environment.

Earned spec checkboxes: docs/specs/voice-chat-speak-text.md:11–15 and :19–21 (all eight bounded requirements). Tests assert exact fields, zero results, ordered/defaulted calls, atomic rejection/recovery, independent queues, real secure/tainted callers and authentic rooted secret identity after GC. NeverSecret checks classify original wrappers at all four positions before any public conversion; mixed malformed arguments cannot bypass them. This earns bounded row409 NeverSecret credit.

Qualification: src/c_api/c_voice_chat_speak.rs:46–52 rejects all secret text despite cached VoiceChatDocumentation.lua:825/831 declaring AllowedWhenTainted/ConditionalSecret. Tests explicitly prove conservative denial, not declared secret-text support. That gap remains unearned; no audio/native-parity credit. The module header at :2 describes unwrap_secret denial although the implementation never calls it. Spec :46 integration gap is stale; formatting/build execution not independently proven here.

Changed tests/c_system_api.rs:109 now supplies boolean false, matching cached declaration :834; old numeric 100 would fail the new strict provider. New malformed-input test explicitly rejects 100. That existing smoke test was inspected, not executed under the three authorized filters. Cached NarrationManager.lua:29 omits overlap (covered); TextToSpeechFrame.lua:178–184 supplies overlap (public boolean covered). Real cached caller execution and secret narration remain unverified.

### 2. Click-binding spell eligibility — ACCEPT WITH QUALIFICATIONS

9/9 tests passed, exit 0. src/c_api/c_click_bindings_spell.rs:18–34 queries live HashSet membership after shared identifier validation; src/lua_api/state/sim_state.rs:203 and src/lua_api/state.rs:191 provide independent empty-default state. Alias/member mutations, misses, exact public boolean arity, strict endpoints/invalid inputs, profile independence, actual tainted calls and authentic secrets after GC are behavioral assertions.

Earned spec checkboxes: docs/specs/click-binding-spell-identifier.md:11–14 and :18–21 (all eight bounded requirements). Row255 receives bounded public SpellIdentifier credit, not native eligibility or secret-permission parity.

Registration winner: init_lua_state applies temporary bootstrap at src/lua_api/env_init/mod.rs:58 (workarounds/mod.rs:264 installs the guarded true-returning Lua default), registers the existing profile at :59, then overwrites the getter with Rust under retail-12-0-5 at :60–61. Existing profile registrar only supplies profile methods. The false/mutation tests prove the old unconditional-true definition does not win. Older profiles retain the Lua definition. No need to remove it for the stated compatibility arrangement.

Defect in documentation: docs/specs/click-binding-spell-identifier.md:36 and :46 require removal of the old provider, contradicting its deliberately retained older-profile role. :47 says no runtime proof exists, now stale. Field/module/registration portions of :46 are earned, removal is not. Full UI load/show/hide and interaction-click control checkbox :48 is not earned by these runs.

Compatibility risk: cached Blizzard_ClickBindingUI.lua:429 now refuses every cursor spell until host eligibility is seeded; Blizzard_SpellBookItem.lua:469 disables click-binding affordances for unseeded spells. This follows explicit empty-default policy, but is a real behavior change from unconditional true, not unchanged native catalog behavior. Tests preserve default profile interactions, not loaded UI behavior. src/c_api/c_spell.rs:671–676 rejects secret identifiers even for secure callers; AllowedWhenTainted permissions remain explicitly unmodeled.

### 3. Encounter event sound — ACCEPT WITH QUALIFICATIONS

4/4 tests passed, exit 0. Credit bounded existing modeled behavior, not a new producer fix. src/lua_api/globals/missing_surface/encounter_events.rs:163–193 stores event/trigger sound records in environment Lua state; :265–275 reads the configured record and :307–315 copies fields. Getter :147–160 returns the stored copy or zero values. Mutable sound overrides are not a placeholder despite fixed event catalog IDs.

Earned spec checkboxes: docs/specs/encounter-event-sound-no-error.md:53–57 (all five bounded public-input requirements): distinct configured triggers/fields, missing/unknown calls without error, selected clearing, detached output, tainted public calls preserving caller taint. Guarded compatibility getter at workarounds/temporary/encounter_state.rs:207 cannot overwrite the already registered Rust getter. Zero-value miss assertions distinguish Rust behavior from that Lua getter's single nil.

Qualification: cached EncounterEventsDocumentation.lua:58 declares AllowedWhenUntainted, but getter :148–152 and parsers :326–343 never authenticate with unwrap_secret before validation. Secret inputs fall through representation matching; no secure-secret acceptance/tainted-secret denial proof exists. This is a pre-existing full-contract gap, excluded by the bounded spec (:78), not newly introduced here. No full secret-argument credit. No native catalog/default/audio claims.

The four tests passing before producer changes is appropriate for an already modeled API: they test state-dependent behavior rather than merely absence of errors. Parent-to-commit producer diff is empty. Do not claim RED or a newly fixed producer.

## Proof limits and merge risk

HEAD exactly a0e23199dc60d4056add5f5a15b68f23dcac38cc; requested source/test diff and working diff empty. Authorized prebuilt filters observed 21/21 passing, all exit 0, stderr empty. No cargo or extra filters run; binary build provenance/profile flags not independently established, although feature-gated test discovery proves retail-12-0-5 is enabled. No pre-commit binary executed: click tests distinguish historical constant true by static counterexample; new speech field/tests cannot compile unchanged against the parent. Runtime RED counts are therefore unverified, not invented.

[EXIST] PASS: both producer files (94/34 lines), three test files (309/289/110 lines), specs, registration/state/source/declaration files read.
[SUBSTANTIVE] PASS for bounded state-backed behavior in all three slices; stated secret-contract/native-catalog gaps remain.
[WIRED] PASS: speech globals registration, click post-bootstrap registration, encounter missing_surface.rs:235 registrar; passing calls exercise effective runtime definitions.
[ANTI-PATTERN] PASS: zero TODO/FIXME/HACK/XXX in the two new producer files and three tests; no empty producer bodies or commented-out implementation observed.

Merging today: bounded behavioral credit supported; full Blizzard caller regression safety is not. Empty click eligibility intentionally changes visible spell-binding availability; secret speech and encounter secret-input contracts remain incomplete. Documentation retains stale integration/execution/removal claims. No repo files modified; report only.
