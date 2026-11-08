# Supplied Speak For Me activity

`VoiceChatState.is_speak_for_me_active` supplies independent Speak For Me activity. False initialization is simulator policy, not a native default. State-backed query registered after genuine result-boundary RED; bounded Retail query/preservation/startup/default-check proof passes. Bounded cached consumer acceptance independently passes; initial GREEN artifact retention PASS; historical path hygiene FAIL; [bounded follow-up2 PASS](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/voice-speak-for-me-active-2026-10-13/README.md#initial-green-retention-and-current-path-follow-up--2026-10-13). [Profile publication integrity](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/voice-speak-for-me-active-2026-10-13/README.md#new-profile-green-runtime--2026-10-13) is a separate bounded PASS; current preservation selected TEXT gate is bounded PASS; new gate-copy integrity is independently bounded PASS. Existing [voice transcription contract](voice-transcribing-input.md) remains distinct.

## What it must do

- [x] Zero-argument `C_VoiceChat.IsSpeakForMeActive()` returns exactly one non-nil boolean equal to the current supplied input; default false is explicit simulator policy.
- [x] Retained query functions read live false/true/false transitions; retained values remain snapshots and environments are isolated.
- [x] Activity is independent of transcription, permission and channel state; clearing channels or disconnecting does not infer activity.

### Cached Retail edit-box consumer

- [x] The unchanged cached consumer and real ChatFrame3 edit box load through the existing full constructor; the entire runtime-error collection stays empty throughout the test.
- [x] Unmuted false activity disables activation, clears the prompt and hides a previously shown edit box. Unmuted true enables activation and sets `REMOTE_TEXT_TO_SPEECH`; no automatic Show behavior is required.
- [x] Muting preserves queried activity true while disabling activation and displaying `VOICE_TRANSCRIPTION_MUTED` in `RED_FONT_COLOR`; unmuted false restoration clears the prompt and hides again.

## How it works

- [Lua API architecture](../lua-api.md)

## Implementation inventory

- `src/lua_api/sim_substates/mod.rs`: independent supplied input/default.
- `src/c_api/c_voice_chat_activity.rs`: query-only shared registration; Retail/PTR retain `retail-12-0-5`, Mists/WoW Forever expose the documented query, other profiles remain unchanged.
- `src/c_api/c_voice_chat_channels.rs`: existing transcription/channel selectors retain their original gate.
- `tests/voice_speak_for_me_active_input.rs`: two behavioral cases under `retail-12-0-5`.
- `tests/voice_speak_for_me_consumer.rs`: one GUI/capability-gated real cached edit-box consumer case.

## Tests asserting this spec

Generated integration filter `voice_speak_for_me_active_input::` selects default/arity/type and retained-query/transition/environment/independence cases. The [Retail evidence packet](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/voice-speak-for-me-active-2026-10-13/README.md) owns initial/refined RED reached boundaries, later query/transcription results, normal startup diagnostics, compiler/source checks and their exact revisions. [Retention status](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/voice-speak-for-me-active-2026-10-13/README.md#initial-green-retention-and-current-path-follow-up--2026-10-13) is separate from execution. Those historical Retail receipts do not establish current profile publication, native defaults or broader acceptance.

### Cached consumer checkpoint — 2026-10-13

The [cached-consumer checkpoint](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/voice-speak-for-me-active-2026-10-13/README.md#current-cached-consumer-checkpoint--2026-10-13) and linked independent report own exact revision, reached decisions, runtime diagnostics and count. The test exercises the consumer requirements above through the unchanged constructor and cached function, without clearing/filtering runtime errors. No independent consumer RED, automatic true-Show, event-producer, native or rendering acceptance follows. Query, transcription, normal startup, performance and retained-text integrity remain separate evidence scopes.

## Documented-profile publication

Cached `VoiceChatDocumentation.lua` directly declares the zero-argument, one non-nil boolean query for Retail, PTR, Mists and WoW Forever. Manifest membership does not establish contents for unavailable Wrath/Era/Anniversary files; their query contract remains unknown. This slice publishes only the existing independent activity input, not transcription or channel selectors, and preserves their current gates.

- [x] Expose only the modeled activity query for the four directly documented profiles through `src/c_api/`; preserve the existing Retail/PTR `retail-12-0-5` epoch floor and unrelated channel gates. No new publication on unproved Wrath/Era/Anniversary profiles.
- [x] Execute Mists result-boundary RED before changing publication at `abb979c44`: 0 PASS / 2 FAIL, actual wait exit101. Both supplied false and retained supplied true calls reach arity checks then fail boolean assertions with explicit `got nil` diagnostics; later value/isolation/restoration assertions are unreached. Initial missing bwrap target mountpoint failed before execution (wait1, zero tests), separately retained; the earlier `be2fa83fb` private-module E0603 compile failure is not RED. `tests/voice_speak_for_me_profile_input.rs` also applies to WoW Forever, subsequently executed in the separate GREEN epoch.
- [x] Independently verify saved Mists/WoW Forever query-only GREEN; the [owning checkpoint](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/voice-speak-for-me-active-2026-10-13/README.md#new-profile-green-runtime--2026-10-13) owns reached assertions and exact revisions.
- [x] Independently verify revision-scoped Retail/PTR preservation; the [current checkpoint](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/voice-speak-for-me-active-2026-10-13/README.md#current-preservation-checkpoint--2026-10-13) distinguishes Retail64a cached consumer from PTR2bb query/transcription and separate startup/check proof. No PTR cached voice-consumer claim.
- [x] Independently verify profile GREEN publication integrity; the [profile checkpoint](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/voice-speak-for-me-active-2026-10-13/README.md#new-profile-green-runtime--2026-10-13) owns bounded retention and historical failures.
- [x] Complete separate current preservation selected TEXT retention/privacy/link gate; new recovery gate-copy integrity is independently bounded PASS. Earlier Retail evidence remains revision-scoped. [Profile packet](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/voice-speak-for-me-active-2026-10-13/README.md#documented-profile-query-publication--2026-10-13) owns reached RED boundaries, setup failures, source proof, warnings and their separate integrity scopes.

## Known gaps (current cycle)

- [x] Execute genuine result-boundary RED; add explicit state-backed getter instead of the generic one-nil namespace result.
- [x] Bounded Retail query2/transcription-preservation3, fresh startup, default library/bin check, source/format/readability and independent proof.
- [x] Original GREEN packet [bounded follow-up2 PASS](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/voice-speak-for-me-active-2026-10-13/README.md#initial-green-retention-and-current-path-follow-up--2026-10-13); initial artifact-retention PASS and historical path hygiene FAIL preserved. Profile publication integrity is separate from current preservation selected TEXT PASS and bounded new recovery gate-copy PASS.
- [x] Execute and independently verify the real cached edit-box decision case; no independent consumer RED or event-producer/native/render acceptance is claimed.

## Out of scope

Native activity meaning/defaults/producers/transitions, transport, events, setters, permission modeling and broader-profile/native parity are unknown. No inferred channel aggregation or vendor behavior changes.
