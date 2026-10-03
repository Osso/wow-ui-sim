# Encounter-event sound lookup without errors

Source `prose-2026-03-12-043` in [12.0.5 register](../../data/patch-api/sources/12.0.5-register.json):

> The C_EncounterEvents.GetEventSound API no longer errors when called.

Tests-only slice of the existing mutable event/trigger sound override provider; no provider change or audio playback.

Cached retail `Blizzard_APIDocumentationGenerated/EncounterEventsDocumentation.lua`, lines 55–70:

```lua
Name = "GetEventSound",
Type = "Function",
MayReturnNothing = true,
SecretArguments = "AllowedWhenUntainted",
Documentation = { "Returns information on a custom sound file to be played when an encounter event trigger occurs." },
Arguments =
{
    { Name = "encounterEventID", Type = "number", Nilable = false },
    { Name = "trigger", Type = "EncounterEventSoundTrigger", Nilable = false },
},
Returns =
{
    { Name = "sound", Type = "EncounterEventSoundInfo", Nilable = false },
},
```

Lines 119–129:

```lua
Name = "SetEventSound",
Type = "Function",
SecretArguments = "NotAllowed",
Documentation = { "Sets a custom sound file to be played when an encounter event trigger occurs." },
Arguments =
{
    { Name = "encounterEventID", Type = "number", Nilable = false },
    { Name = "trigger", Type = "EncounterEventSoundTrigger", Nilable = false },
    { Name = "sound", Type = "EncounterEventSoundInfo", Nilable = true },
},
```

`EncounterEventsSharedDocumentation.lua` declares `EncounterEventSoundTrigger` as an enumeration with `NumValues = 3`, `MinValue = 0`, `MaxValue = 2`: `OnTextWarningShown = 0`, `OnTimelineEventFinished = 1`, `OnTimelineEventHighlight = 2`. Its `EncounterEventSoundInfo` structure fields are:

```lua
{ Name = "file", Type = "FileAsset", Nilable = false },
{ Name = "channel", Type = "UISoundSubType", Nilable = false, Default = "g_defaultSI3UISoundSubTypeForLua" },
{ Name = "volume", Type = "number", Nilable = false, Default = 1 },
```

## What it must do

- [ ] Return exactly one sound table for a configured event/trigger without error, preserving explicitly set `file`, `channel` and `volume` fields; independent trigger overrides remain distinct.
- [ ] Return without error for a known event lacking an override, an unconfigured trigger, and a valid unknown event ID. INFERRED: these misses produce zero values, not a single nil; documented `MayReturnNothing` permits this but does not establish miss policy.
- [ ] Reflect clearing one override through `SetEventSound(eventID, trigger, nil)` while retaining another trigger. INFERRED: nil clears the override; this matches the existing provider, not independently verified native behavior.
- [ ] Return a detached table: mutating its fields must not affect the next lookup. INFERRED native copy policy; tests assert the existing model's isolation guarantee.
- [ ] Accept public arguments from addon-tainted callers without errors or caller-taint changes; an outer untainted caller remains untainted.

## How it works

- [Lua API](../lua-api.md).

## Implementation inventory

- `src/lua_api/globals/missing_surface/encounter_events.rs` — existing Rust `get_event_sound` / `set_event_sound`, left unchanged.
- `src/lua_api/workarounds/temporary/encounter_state.rs` — pre-existing guarded Lua compatibility definition, left unchanged; not added or relied on as a new fallback.

## Tests asserting this spec

- `tests/encounter_event_sound_no_error.rs` — four tests seed via the public setter, use existing catalog events 1/2, unknown positive ID 900001, documented triggers 0/1/2, explicit sound fields, exact return arity, detached mutation and addon-tainted calls.

## Known gaps (current cycle)

- [ ] Main session must compile, format and execute tests. All four tests are expected to pass against the existing provider; no execution proof produced here.
- [ ] Existing event IDs 1/2/3 are a tiny fixed catalog, not a live encounter catalog. Configured sound values themselves are mutable state-backed overrides.

## Out of scope

Secret arguments, invalid arguments, defaults for omitted sound fields, audio playback, native event catalog discovery, exact native error wording, and source-row accounting. INFERRED policies above describe bounded simulator behavior, not native parity. Numeric file IDs and `Master`/`SFX` channel strings are simulator fixtures; actual asset existence/playability and native acceptance are not verified.
