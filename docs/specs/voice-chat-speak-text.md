# Voice-chat speech requests — row409

`C_VoiceChat.SpeakText(voiceID, text, rate, volume, overlap)` records speech requests without audio playback. The 12.0.5 [GlobalAPI source](../../data/patch-api/sources/12.0.5-api-changes.txt) line 409 adds `arg1,3-5 NeverSecret`. Source ID: `global api-C_VoiceChat-SpeakText-409`.

Cached retail `Blizzard_APIDocumentationGenerated/VoiceChatDocumentation.lua` lines 823–836 declare nonnil `voiceID: number`, `text: cstring` (`ConditionalSecret`), `rate: number`, `volume: number`, and `overlap: bool` (default false), `SecretArguments = "AllowedWhenTainted"`, and no returns. Positions 1/3/4/5 carry `NeverSecret`. Contract context, not native execution evidence.

## What it must do

### Stored result

- [x] Append one request per accepted call to the environment's initially empty queue, preserving call order and all five fields exactly. No synthesis, XML parsing, volume normalization, voice catalog or fabricated records.
- [x] Return exactly zero values after an accepted call.
- [x] Omitted overlap uses the declared false default. INFERRED: explicit nil also uses false.
- [x] INFERRED representation policy: accept only finite Lua numbers for voice ID, rate and volume, UTF-8 Lua strings for text, and booleans for supplied nonnil overlap. No numeric/string coercion, integer restriction or numeric range policy. Empty text is accepted unchanged.
- [x] Validation failures leave the entire existing request queue unchanged; later ordinary requests still work. Environments own independent queues.

### Argument policy under `retail-12-0-5`

- [x] Reject original secret values in positions 1, 3, 4 and 5 for secure and tainted callers before any conversion or mutation, even when another argument is malformed. Wrapper payload type does not bypass NeverSecret.
- [x] Preserve caller taint and the original secret wrappers on rejection; rooted secrets retain identity and secrecy across full GC. Public requests remain usable by tainted callers.
- [x] Conservatively reject secret text for both secure and tainted callers before conversion or mutation. This is an explicitly unmodeled policy, NOT implementation of the declared AllowedWhenTainted/ConditionalSecret behavior.

## How it works

- [Lua API](../lua-api.md)
- [C API boundary](../../CLAUDE.md#c-api-boundary)

The provider validates original secret wrappers, constructs a public request, then appends it through a mutable SimState borrow. It never unwraps secrets or modifies caller taint. Pinned rilua `6044544` implements `unwrap_secret` with an untainted-caller guard; using it for tainted secret-text acceptance would misrepresent the declared permission.

## Implementation inventory

- `src/c_api/c_voice_chat_speak.rs` — explicit namespace registration, argument boundary, public `SpeakTextRequest` and queue append.
- Integration required in `src/c_api/mod.rs`, `src/lua_api/globals/register.rs`, `src/lua_api/state/sim_state.rs` and `src/lua_api/state.rs`; exact insertions live in the task handoff. Those files are outside this authoring slice.

## Tests asserting this spec

- `tests/voice_chat_speak_text.rs` — eight authored cases: exact fields/arity, ordered/defaulted requests, secure NeverSecret denial, tainted NeverSecret denial, conservative secret-text rejection, atomic malformed-input rejection and public recovery, rooted-secret GC, environment isolation.

## Development proof — 2026-10-03

Not compiled, formatted or executed: authoring slice permits file writes only. Main session owns integration and targeted RED/GREEN evidence. All requirements remain unchecked.

## Development proof and independent bounded acceptance — 2026-10-03

Inputs and producer landed together in `a0e23199d`. RED with the producers withheld from the working tree: 0 PASS / 8 FAIL. GREEN: 8/8; the combined run was 396 PASS / 1 FAIL, the failure being `c_system_api::test_c_console_get_all_commands_empty` on an untouched console command count. `cargo fmt --check` exit0; startup `lua-errors` `[]`.

Main accepts an independent GPT-6.1-sol review: **ACCEPT WITH QUALIFICATIONS** (report SHA256 `a273fa728f03621dee89db3ef7ed675e9e1304b2ce29bcaaba88028d079800de`, scratchpad-only), own rerun 8/8 exit0. Secret text is rejected outright; cached Blizzard callers were inspected, not executed. Checked requirements are bounded simulator proof on the tested fixtures, not native parity. No `cargo check`, broad suite or older-profile run.

[Page accounting](../../data/patch-api/sources/12.0.5-page-coverage.json): rows 409 under new capability `voice-chat-speak-text`; **107 capabilities/362 IDs; 77 pending /239 bounded /13 partial /33 metadata**.

## Known gaps (current cycle)

- [ ] Declared AllowedWhenTainted and ConditionalSecret text behavior is unmodeled; all secret text is denied, including secure text.
- [ ] Registration/state insertions and compilation/behavioral proof remain main-session work.

## Out of scope

- Audio playback, TTS voices/acquisition, speech completion/events, cancellation, platform SAPI XML tags and bookmarks.
- Native nil/type/range behavior and exact error wording; INFERRED policies above are simulator choices.
- Profiles below `retail-12-0-5`, arguments beyond the five declared positions, other C_VoiceChat methods and other source rows.
