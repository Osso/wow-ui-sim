# Profession specializations

`A_Admin` supplies profession specialization data for `C_ProfSpecs` and the Blizzard professions panel. This is simulator scenario input, not a claim about a character's live profession data.

**INFERRED (R88ez, native evidence pending):** An open profession with a supplied specialization record is the default spec skill line. If it has no record, use the first supplied record; if no records exist, return nil. The spec tab is shown when a default skill line exists.

## A_Admin.SetProfessionSpecSkillLines(records)

Replaces the supplied specialization records. Each array entry is a table with:

- `skillLineID` — profession SkillLine ID.
- `configID` — config ID returned for that skill line.
- `specTabIDs` — ordered array of trait-tree IDs.
- `currency` — optional table with `numAvailable` and `currencyName`.
- `specTabInfo` — optional table with `enabled` and `errorReason`.

```lua
A_Admin.SetProfessionSpecSkillLines({
    {
        skillLineID = 164,
        configID = 8101,
        specTabIDs = {9101},
        currency = {numAvailable = 0, currencyName = ""},
    },
    {
        skillLineID = 186,
        configID = 8202,
        specTabIDs = {9202},
    },
})
```

**INFERRED (R88ez, native evidence pending):** An absent skill-line record returns config ID `0`, an empty tab-ID array, false from `SkillLineHasSpecialization`, and currency info `{numAvailable = 0, currencyName = ""}`. A supplied record returns true from `SkillLineHasSpecialization`. `GetSpecTabInfo` uses the supplied `specTabInfo` when present; otherwise a record returns `{enabled = true, errorReason = ""}`. With no default record, it returns `{enabled = false, errorReason = ""}`.

## A_Admin.SetProfessionSpecTabInfo(tabTreeID, info)

Supplies one `ProfTabInfo` record. `info` has `rootNodeID`, `name`, `description`, `rootIconID`, and `highlights`; each highlight is a table with `description`.

**INFERRED (R88ez, native evidence pending):** `GetTabInfo(tabTreeID)` returns the supplied record or nil when the ID is not supplied.

The query shapes are declared in Retail and WoW Forever's pinned `Blizzard_APIDocumentationGenerated/ProfessionSpecUIDocumentation.lua` (`GetConfigIDForSkillLine`: 58, `GetCurrencyInfoForSkillLine`: 73, `GetDefaultSpecSkillLine`: 88, `GetSpecTabIDsForSkillLine`: 197, `GetSpecTabInfo`: 212, `GetTabInfo`: 299, `ShouldShowSpecTab`: 368, `SkillLineHasSpecialization`: 377). Their structures are in `ProfessionSpecConstantsDocumentation.lua` (`ProfTabInfo`: 65–75, `SpecializationCurrencyInfo`: 86–93, `SpecializationTabInfo`: 95–102).
