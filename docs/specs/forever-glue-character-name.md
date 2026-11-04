# Forever glue character names

Forever character-select consumers query the existing supplied character roster.
The query lives in `src/lua_api/globals/missing_surface/character_select.rs`.
[Character identity](character-identity.md) describes separate in-game name inputs.

## What it must do

- [x] Forever `GetBasicCharacterInfo` publishes `characterName` from the selected supplied account, not the legacy `name` field. PTR retains its existing path; other profiles are unchanged.
- [x] The original Shared CharacterSelectUtil lookup and Camelot tooltip must accept two distinct supplied names, including when specID is absent.
- [x] `fullName` passes through one named **EMULATED** seam: the supplied account name is already display text. No surname is invented or copied from a different character. Native composition awaits WM100 and will replace this seam.

## How it works

- [Lua API](../lua-api.md)

## Implementation inventory

- `src/lua_api/globals/missing_surface/character_select.rs` — supplied name projection and the named full-name seam.
- `tests/glue_character_select.rs` — original selected glue lookup, tooltip and roster tests.

## Tests asserting this spec

`forever_character_tooltip_projects_two_supplied_character_names` and the existing missing-spec and roster cases.

## Known gaps (current cycle)

- [ ] Native fullName composition (WM100 pending).

## Out of scope

Roster ownership, persistence, class/race/catalog modeling and other profile field changes.

## Source evidence

No matching pinned generated API declaration was found for the glue query.
Forever `Blizzard_GlueXML/Shared/CharacterSelect/CharacterSelectUtil.lua:4-17`
requires `characterName`; the selected
`Camelot/CharacterSelect/CharacterSelectUtil.lua:7-19` consumes `fullName` as text.
These consumer observations establish field use, not native name composition.
