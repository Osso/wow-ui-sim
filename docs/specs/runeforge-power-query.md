# Runeforge power query

`C_LegendaryCrafting.GetRuneforgePowersByClassSpecAndCovenant` accepts four nullable numeric arguments and returns a fresh empty table. It is published for Retail, PTR and WoW Forever. Each call returns its own table.

## Best-guess basis (best-guess, native UNKNOWN)

The pinned Retail, PTR and WoW Forever `Blizzard_APIDocumentationGenerated/LegendaryCraftingDocumentation.lua:181–197` declarations specify nullable class, specialization, covenant and filter arguments and a non-nil table of numeric IDs. The simulator has no runeforge catalogue, so the empty result describes its current data, not a claim about a populated client. Retail's `Blizzard_EncounterJournal/Mainline/Blizzard_LootJournal.lua:185–194` uses this query.

## Known gaps

- Native filtering semantics are unknown.
- The missing runeforge catalogue needs a DB2 export of `RuneforgeLegendaryAbility` before populated results can be modeled.
