# Supplied voice settings access

`C_VoiceChat.CanAccessSettings()` exposes an environment-local supplied observation from `VoiceChatState.can_access_settings`. [Original consumer and model](../wiki/systems/voice-settings-access.md) describe the boundary. Retail and Forever start with true as a user-approved default, not native evidence; other profiles remain initially unknown.

## What it must do

- [x] Publish on Retail, PTR, Forever, Mists, Era and Anniversary; keep the member absent on Wrath.
- [x] Return exactly one nonnil boolean equal to the current supplied observation. `None` raises `C_VoiceChat.CanAccessSettings: settings access has not been supplied` rather than selecting a readiness default.
- [x] Preserve false/true transitions, unset recovery and independent environments through the original Audio.lua polling callback. Settings access is not derived from connection, permission, channel or host sound state.

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

- `src/lua_api/sim_substates/mod.rs`: optional host-supplied observation, initially true on Retail and Forever and unknown on other profiles.
- `src/c_api/c_voice_chat_settings.rs`: profile-gated namespace query and named unset error.
- `src/c_api/registration.rs`, `src/c_api/mod.rs`: namespace registration wiring.

## Tests asserting this spec

- `tests/voice_settings_access.rs`: original consumer decisions, result shape, isolation, unknown-state recovery and Wrath absence.
- `tests/voice_settings_access/original_callback.lua`: original selected registrant and polling callback; entry observation without changing vendor callees.

## Verified boundary — 2026-10-26

Current-code revision `688ca21bc` and producer-free sibling `c082a2398` have byte-identical tests. Three exact original-consumer selectors per declaring profile pass GREEN and fail RED at the missing namespace member. Two non-vacuous Wrath selectors preserve raw-member absence; its initially-unset selector has no consumer and returns early. All seven full-feature all-target checks have zero compiler warnings. Paired raw stock maps are identical: Retail 0/0, PTR 124/140, Forever 26/26, Mists 6/12, Wrath/Era/Anniversary 0/0. All 98 final test-discovery, test-run and stock-launch CVar guards remain unchanged.

The true branch proves entry into the original initialization function, not completion or ticker cancellation. The hook deliberately stops before separate device/settings contracts. That historical verification predates the user-approved Retail and Forever default.

## Known gaps (current cycle)

None within the supplied-observation boundary.

## Out of scope

Native readiness criteria/defaults, voice transport, device catalogs, automatic process initialization, events, additional admin APIs and complete voice settings initialization. Tests stop at entry into the original initialization function before separate device/API contracts.
