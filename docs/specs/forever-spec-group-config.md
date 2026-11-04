# Forever specialization-group configuration

Forever 70707 `C_SpecializationInfo.GetCombatConfigIDForSpecGroup(groupIndex)`
queries supplied loaded player configurations, not the active configuration ID.
WM99 establishes the local native selection and optional-result paths; complete
protected entry-to-result continuity and live execution remain unverified.

## What it must do

- [x] Require an index. Omitted/nil input raises the native diagnostic body `Usage: local configID = C_SpecializationInfo.GetCombatConfigIDForSpecGroup(groupIndex)`. Context-dependent error decoration is not fixed.
- [x] Search supplied configurations in provider order, requiring combat kind and the selected group flag. Index 1 uses flag 1; every other accepted index, including 99, uses flag 8. No group-count validation is added.
- [x] Return exactly one matching numeric ID, including zero, or one nil if no match exists. The default supplied collection is empty. Do not alias the separately stored active configuration ID or the legacy talent grid.
- [ ] Publish only on Forever; preserve other profile surfaces.

## How it works

- [Forever talent model](../wiki/systems/forever-talents.md)
- [Lua API](../lua-api.md)

## Implementation inventory

- `src/c_api/c_spec.rs` — profile publication, argument validation and ordered selection.
- `src/c_api/forever_talents.rs` — supplied loaded configuration records in existing Forever character state.

## Tests asserting this spec

- `tests/forever_spec_group_config.rs` — primary/secondary flags, order, discriminator, index 99, zero ID, absent matches, arity and required-input diagnostics.
- `tests/test_keybindings_spellbook_talents.rs` — original Camelot class-talent tab and spellbook tooltip flow.

## Known gaps (current cycle)

- [ ] Native live population, switching, persistence and update timing are unknown.

## Out of scope

Trait catalog reconstruction, implicit seeded configurations and active-config selection changes.

## Source evidence

Forever 70707 `Blizzard_APIDocumentationGenerated/SpecializationInfoDocumentation.lua:90-103`
declares the required luaIndex and nullable number. The selected
`Blizzard_PlayerSpells/Camelot/ClassTalents/Blizzard_ClassTalentsFrame.lua:183`
consumes it while selecting a tab. WM99's recovered native query independently
establishes first-match selection, flags 1/8, zero IDs and nil absence.
