# Forever trait-group display information

Forever 70707 `C_Traits.GetGroupDisplayInfoByTreeID` projects supplied display
relations and SkillLine records in provider order. WM101 establishes the local
collector and six-field publisher. No matching Forever CSV is available;
relation-column names and unknown-tree emptiness are inferred.

## What it must do

- [ ] Require numeric treeID in signed 32-bit bounds, then truncate fractions toward zero. Zero has no special branch. Rejected types use `Usage: local displayInfos = C_Traits.GetGroupDisplayInfoByTreeID(treeID)`.
- [ ] Return one fresh indexed array with groupID, requested treeID, skillLineID, orderIndex, displayName and icon per row. Preserve supplied order; do not sort by orderIndex.
- [ ] Join supplied SkillLine display text/icon. A missing join retains the row with empty displayName and numeric-zero icon.
- [ ] One named INFERRED seam maps an unknown tree without supplied relations to an empty array. Do not borrow Retail data or invent populated rows.
- [ ] Publish only on Forever. The original Camelot class-talent tab must consume the result without a missing-method error.

## How it works

- [Forever talent model](../wiki/systems/forever-talents.md)

## Implementation inventory

- `src/c_api/trait_group_display.rs` — supplied relations, SkillLine joins and query.
- `src/c_api/forever_talents.rs` — display model in existing Forever player talent state.
- `src/c_api/registration.rs` — namespace registration.

## Tests asserting this spec

- `tests/forever_trait_group_display.rs` — distinct supplied values, provider order, missing join, snapshots, fractional selector, bounds and required-input errors.
- `tests/test_keybindings_spellbook_talents.rs` — original class-talent tab switch and spellbook tooltip.

## Known gaps (current cycle)

- [ ] Native relation acquisition, concrete provider, populated data and protected continuity remain unknown.

## Out of scope

Database export generation, live refresh, events, sorting and other profile publication.

## Source evidence

Forever `Blizzard_APIDocumentationGenerated/SharedTraitsDocumentation.lua:297-310`
and `:1080-1092` declare the query and record. Selected
`Blizzard_PlayerSpells/Camelot/ClassTalents/Blizzard_ClassTalentsFrame.lua:95`
consumes the ordered array. WM101 independently recovers signed numeric selection,
provider order, missing-join projection and the always-created result table.
