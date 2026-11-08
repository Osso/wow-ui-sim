# Supplied voice settings access

`C_VoiceChat.CanAccessSettings()` exposes an environment-local supplied observation from `VoiceChatState.can_access_settings`. [Original consumer and model](../wiki/systems/voice-settings-access.md) describe the boundary.

Every declaring profile starts with false. Native evidence for PTR 12.1.7, Retail 12.1.5 and Forever 1.60.1 (VERIFIED, `/home/osso/.local/state/wow-ui-sim/handoff/native-evidence-12.1.7/C_VoiceChat.CanAccessSettings.md`): the function is the AND of the voice-service readiness state, it is not a CVar or account flag, and its static initial state returns false at process start. It becomes true only after the voice service reaches ready, observed by polling. The simulator has no voice service, so false is the permanent observation. Mists, Era and Anniversary default to the same false because their pinned Audio.lua polls the query until the voice proxy settings are ready; that default is INFERRED from the pinned polling loop (no binary evidence).

## What it must do

- [x] Publish on Retail, PTR, Forever, Mists, Era and Anniversary; keep the member absent on Wrath.
- [x] Return exactly one nonnil boolean equal to the current supplied observation. `None` raises `C_VoiceChat.CanAccessSettings: settings access has not been supplied` rather than selecting a readiness default.
- [x] Start false on every declaring profile. PTR/Retail/Forever are VERIFIED by the native evidence file above; Mists/Era/Anniversary are UNVERIFIED (no binary) but share the same pre-ready behaviour.
- [x] Preserve false/true transitions, unset recovery and independent environments through the original Audio.lua polling callback. Settings access is not derived from connection, permission, channel or host sound state.
- [x] Keep the five-second polling ticker free of Lua errors on profiles whose voice proxy never initializes. The ticker must observe false and continue polling; it must not hit the unset error.

### Pinned declarations

Paths are relative to the active profile's `AddOns/Blizzard_APIDocumentationGenerated/`. Each function has no declared arguments and one nonnil boolean result `canAccess`.

| Profiles | VoiceChatDocumentation.lua declaration |
|---|---|
| Retail, PTR, Forever | namespace :5; function :41-42; result :44-47 |
| Mists, Era, Anniversary | namespace :5; function :38-39; result :41-44 |
| Wrath | File and declaration absent; do not publish |

## How it works

- [Voice settings access](../wiki/systems/voice-settings-access.md)

## Implementation inventory

- `src/lua_api/sim_substates/mod.rs`: optional host-supplied observation, initially false on every profile that publishes the query (PTR/Retail/Forever VERIFIED, Mists/Era/Anniversary UNVERIFIED); Wrath does not publish the query.
- `src/c_api/c_voice_chat_settings.rs`: profile-gated namespace query and named unset error.
- `src/c_api/registration.rs`, `src/c_api/mod.rs`: namespace registration wiring.

## Tests asserting this spec

- `tests/voice_settings_access.rs`: original consumer decisions, result shape, isolation, unknown-state recovery and Wrath absence.
- `tests/voice_settings_access/original_callback.lua`: original selected registrant and polling callback; entry observation without changing vendor callees.

## Verified boundary — 2026-10-26

Current-code revision `688ca21bc` and producer-free sibling `c082a2398` have byte-identical tests. Three exact original-consumer selectors per declaring profile pass GREEN and fail RED at the missing namespace member. Two non-vacuous Wrath selectors preserve raw-member absence; its initially-unset selector has no consumer and returns early. All seven full-feature all-target checks have zero compiler warnings. Paired raw stock maps are identical: Retail 0/0, PTR 124/140, Forever 26/26, Mists 6/12, Wrath/Era/Anniversary 0/0. All 98 final test-discovery, test-run and stock-launch CVar guards remain unchanged.

The true branch proves entry into the original initialization function, not completion or ticker cancellation. The hook deliberately stops before separate device/settings contracts. This historical verification predates the native evidence; it is unaffected by the initial-value change because tests that need readiness supply true explicitly.

## Known gaps (current cycle)

None within the supplied-observation boundary.

## Out of scope

Native readiness criteria/defaults, voice transport, device catalogs, automatic process initialization, events, additional admin APIs and complete voice settings initialization. Tests stop at entry into the original initialization function before separate device/API contracts.
