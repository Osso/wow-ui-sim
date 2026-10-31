# Forever global strings

## Contract

- `client-wowforever` publishes the 27,388 unique tags from the committed build `1.60.1.70532` GlobalStrings CSV, the build of the pinned Forever Blizzard UI; other profiles retain their existing generated table.
- Preserve source text, format tokens, quoted and multiline CSV fields. Runtime uses the existing WoW escape resolver.
- Initial publication replaces string values but preserves existing Lua functions. Missing-string restoration leaves all existing values unchanged.
- Duplicate or empty tags fail generation before output replacement; no arbitrary winner or retail string fallback.
- Integer and float constants remain independently registered. This change does not enable a retail API epoch.

## Source and reproduction

Snapshot: `data/db2/wowforever-1.60.1.70532/GlobalStrings.csv`.
Build-specific source URL and SHA-256: adjacent `global-strings-provenance.json`.

```text
python3 tools/gen_forever_global_strings.py
```

The generator writes the existing PHF table representation used by string registration. It uses Python's CSV reader because the legacy line-oriented generator cannot preserve the snapshot's 38 multiline records. It introduces no runtime dependency.

## Build update from 69913 to 70532 — 2026-10-30

The 69913 table lacked strings the pinned 70532 UI reads. Forever `Audio.lua`:270-271 names its `voiceProvider` dropdown `VOICE_CHAT_SERVICE`, so `Blizzard_Setting.lua`:34 rejected the nil name and `InitVoiceSettings` stopped before the device settings. The 70532 export (SHA-256 `79d0987e3479f3cb19dc68475bef33a7876d7f340daf0fc3eb835cf057a68dd3`, 27,388 rows) differs from 69913 as follows:

- 127 added tags, including `VOICE_CHAT_SERVICE` ("Preferred Voice Chat Service"), the Discord voice-provider and age-restriction strings, `NPEV2_*` tutorial prompts and gamepad targeting options;
- 1 removed tag: row 59448 `AADC_SOCIAL_TAB_UNAVAILABLE` is renamed `SOCIAL_TAB_UNAVAILABLE`, the name the 70532 FriendsFrame and SocialUIShared code reads;
- 85 changed values.

The generator is unchanged apart from its default source and header, and regenerating from the 69913 CSV reproduces the previous table body byte for byte. The superseded 69913 GlobalStrings CSV and provenance are removed; that directory's atlas and listfile data stay at 69913.

## Proof scope

`tests/wowforever_global_strings.rs` loads actual `GameplaySettingsGroup.lua`, checks category ordering and exact combat-start help formatting, and checks function preservation and missing-only restoration. Generator fixtures cover quoted multiline text and fail-fast duplicate rejection. Native localization conformance and full TextToSpeech subsystem behavior are not established by these fixtures.
