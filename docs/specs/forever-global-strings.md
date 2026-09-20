# Forever global strings

## Contract

- `client-wowforever` publishes the 27,262 unique tags from the committed build `1.60.1.69913` GlobalStrings CSV; other profiles retain their existing generated table.
- Preserve source text, format tokens, quoted and multiline CSV fields. Runtime uses the existing WoW escape resolver.
- Initial publication replaces string values but preserves existing Lua functions. Missing-string restoration leaves all existing values unchanged.
- Duplicate or empty tags fail generation before output replacement; no arbitrary winner or retail string fallback.
- Integer and float constants remain independently registered. This change does not enable a retail API epoch.

## Source and reproduction

Snapshot: `data/db2/wowforever-1.60.1.69913/GlobalStrings.csv`.
Build-specific source URL and SHA-256: adjacent `global-strings-provenance.json`.

```text
python3 tools/gen_forever_global_strings.py
```

The generator writes the existing PHF table representation used by string registration. It uses Python's CSV reader because the legacy line-oriented generator cannot preserve the snapshot's 38 multiline records. It introduces no runtime dependency.

## Proof scope

`tests/wowforever_global_strings.rs` loads actual `GameplaySettingsGroup.lua`, checks category ordering and exact combat-start help formatting, and checks function preservation and missing-only restoration. Generator fixtures cover quoted multiline text and fail-fast duplicate rejection. Native localization conformance and full TextToSpeech subsystem behavior are not established by these fixtures.
