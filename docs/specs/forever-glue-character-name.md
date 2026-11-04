# Forever glue character names

Forever character-select consumers query the existing supplied character roster.
The query lives in `src/lua_api/globals/missing_surface/character_select.rs`.
[Character identity](character-identity.md) describes separate in-game inputs.
WM100 recovers Forever 70707's local name composition and result projection;
complete protected-wrapper continuity and live-client execution remain unknown.

## What it must do

- [x] Forever `GetBasicCharacterInfo` publishes `characterName` from the selected supplied account, not legacy `name`. PTR retains its existing path; Retail keeps `name`.
- [x] Publish supplied `surname` separately. Empty surname makes `fullName` equal the first name. Otherwise format the first name, supplied separator (space or hyphen), and surname through `FULL_PLAYER_NAME_WITH_SEPARATOR` from the pinned GlobalStrings value. The live native separator-mode flag is unknown; the host supplies its choice.
- [ ] Only if that localized key is absent, use one named EMULATED missing-format seam. WM100 proves the key and argument order, not the localized literal. No title or realm is appended.
- [x] Missing/nil GUID raises the native usage body `Usage: local basicInfo = GetBasicCharacterInfo(guid)`. Unknown GUID returns zero Lua values, not one nil or a table. Zero-result evidence is WM100 post-lookup emulation, not live execution.
- [x] Original Shared CharacterSelectUtil lookup and Camelot tooltip accept two distinct supplied names when specID is absent. Supplied surname/separator changes must also reach the original tooltip.

## How it works

- [Lua API](../lua-api.md)

## Implementation inventory

- `src/lua_api/globals/missing_surface/character_select.rs` — supplied projection, localized composition and missing-format seam.
- `src/lua_api/state_types/social.rs` — per-account surname and separator inputs.
- `tests/glue_character_select.rs` — original glue lookup, tooltip, roster and required-input cases.

## Tests asserting this spec

`forever_character_tooltip_projects_two_supplied_character_names`,
`forever_character_tooltip_composes_supplied_surname_and_separator`,
`forever_basic_character_info_requires_guid_and_unknown_returns_no_values`,
and existing missing-spec and roster cases.

## Known gaps (current cycle)

- [ ] Complete native 30-field Forever projection. WM100 also establishes superDistrictID and 26 shared fields; this slice changes only consumed name fields and required-input/absence behavior. Existing modeled fields are not claimed as a complete native record.
- [ ] Live separator-flag selection, localized format availability and protected-wrapper continuity remain unknown.

## Out of scope

Roster ownership, persistence, unrelated class/race/catalog modeling and other profile behavior changes.

## Source evidence

No matching pinned generated API declaration was found for the glue query.
Forever `Blizzard_GlueXML/Shared/CharacterSelect/CharacterSelectUtil.lua:4-17`
requires `characterName`; selected `Camelot/CharacterSelect/CharacterSelectUtil.lua:7-19`
consumes `fullName` as text. WM100 separately recovers name/surname composition,
30 Forever fields versus 27 Retail fields, the required GUID diagnostic and
zero-return post-lookup failure. Pinned Forever GlobalStrings provides the format
literal; its presence does not establish every native locale's value.
