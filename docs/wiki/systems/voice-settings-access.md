# Voice settings access

The [supplied-observation contract](../../specs/voice-settings-access.md) exposes settings access without claiming native voice-proxy readiness. The Rust host supplies `Some(false)` or `Some(true)` on `SimState.voice_chat.can_access_settings`; `None` means unknown and raises a named error. Each environment owns its observation. No admin function or readiness side effect is added.

## Original consumer

The selected `Blizzard_SettingsDefinitions_Shared` TOC is enabled, allows both screens, and includes Audio.xml without a profile filter. Audio.xml:3 loads Audio.lua. SettingsRegistrar waits for VARIABLES_LOADED and PLAYER_ENTERING_WORLD on the game screen before calling the Audio registrant. The off-glue registration starts a five-second polling ticker.

In Retail Audio.lua:565, false continues polling; true enters InitVoiceSettings and cancels only after initialization succeeds. PTR/Forever query at :657; Mists/Era/Anniversary at :549. The original note states that initialization requires the voice proxy process to be initialized. Existing connection/permission/channel state does not establish that dependency. Wrath has neither this addon nor its declaration.

## Bounded proof

The fixture loads the original dependency closure and calls the selected Audio registrant. It locates the resulting original polling callback without replacing any vendor callee. A debug call hook observes entry into the original InitVoiceSettings and stops before separate device/settings contracts. This proves the query's branch decision, not complete initialization, timer cancellation, transport or native readiness. The unset path must fail explicitly and recover after supply.

Initial own Retail RED reached original Audio.lua:565 at scaffold `c082a2398` with the absent namespace member. Final proof at current-code `688ca21bc` uses that producer-free sibling with byte-identical tests: three exact original-consumer GREEN/RED selectors on all six declaring profiles, plus two non-vacuous Wrath raw-publication absence cases. The Wrath initially-unset selector returns without a consumer and adds no capability proof.

Seven full-feature all-target checks have zero compiler warnings. All 98 protected final launch CVar guards are unchanged. Paired raw stock maps are byte-identical at Retail 0/0, PTR 124/140, Forever 26/26, Mists 6/12 and Wrath/Era/Anniversary 0/0; no added keys require masking-predecessor attribution. Formatting and changed-line readability pass. These results do not make an unsupplied forecast run clean or establish complete device initialization, timer cancellation or native readiness criteria.

## Sources

- [Contract and declaration citations](../../specs/voice-settings-access.md)
- [Behavioral tests](../../../tests/voice_settings_access.rs)
- [C API query](../../../src/c_api/c_voice_chat_settings.rs)

## See Also

- [Lua API architecture](../../lua-api.md)
- [Speak For Me activity](../../specs/voice-speak-for-me-active-input.md) — a separate observation, not a readiness derivation.
