# Dungeon score rarity colours

## Contract

**Best-guess, native UNKNOWN.** Retail and PTR use profile-specific generated UIDungeonScoreRarity DB2 caches. A missing, invalid or unreadable cache is an error; never substitute colours. The current active row set is the set with the highest final field value. Each family uses the largest minimum not greater than the input; below all minima uses the lowest tier. Ranges are half-open; maximum zero is unbounded. Inputs are required numbers. Outputs are ColorMixin-compatible RGB values.

- Overall rating uses DungeonScoreMin/Max.
- Keystone level uses KeystoneLevelMin/Max.
- Single dungeon combined rating uses fields 009/010.
- Single run rating uses fields 007/008.

## Best-guess basis

[WoWDBDefs UIDungeonScoreRarity.dbd](https://github.com/wowdev/WoWDBDefs/blob/master/definitions/UIDungeonScoreRarity.dbd) layout 6242874E defines the fields, but calls 007–011 unknown. Retail 12.1.5.70575 and PTR 12.1.7.70493 DB2 files each have two five-row sets with final values 96877 and 96879. The first has top score threshold 2200 and the latter 2750. The larger threshold set is selected as a best guess for the newer season. The decoded DB2 rows supply the thresholds. [Forum player observations](https://eu.forums.blizzard.com/en/wow/t/at-what-rating-does-your-rating-turn-orangelegendary/319338) report orange at 2200 in 2021. [Blizzard's season announcement](https://news.blizzard.com/en-us/article/24266321/midnight-season-1-mythic-now-available) lists 1500, 2000, 2500 and 3000 rating achievement milestones; it does not specify colour thresholds or native row selection. Adjacent rows have matching max/min endpoints and the top max is zero; half-open ranges and an open-ended last tier give each boundary one tier. Below-minimum uses the lowest tier because Retail/PTR `ChallengeModeInfoDocumentation.lua:92–106,118–132,223–253` declare non-nil colour outputs.

Retail GroupFinder `Mainline/LFGList.lua:4470` passes leader overall score to the overall function; PTR passes it at :4461. Retail/PTR `Blizzard_ChallengesUI/Mainline/Blizzard_ChallengesUI.lua:490–493,530–533` pass a map's combined affix score to the specific-overall function. The GroupFinder applicant tooltip passes `mapScore` at Retail :2300 / PTR :2287, but its applicant model is unavailable. Its search-result tooltip instead passes `bestRunLevel` to that same function at Retail :4475–4477 / PTR :4466–4468. That quirk may contradict the mapping; native semantics remain unknown. These consumers do not prove native threshold selection. No pinned Retail/PTR consumers of the single-run or keystone color functions were found; their mappings are inferred from generated argument documentation and the DB2 range ratios (roughly one-twentieth and the explicit keystone fields).

## Cache generation

Generate the active profile's persistent cache before running its tests:

```text
wow-cli casc sync-dungeon-score-rarity
```

The default path is the user cache under `wow-ui-sim/dungeon-score-rarity/<profile>/rarity.json`. Release packages and Docker images bundle the generated data beside the executable.

## Known gaps

Pinned Forever, Mists, Era and Anniversary UIDungeonScoreRarity tables contain zero rows. These profiles do not publish the new colour implementations; native empty-table behaviour is unknown. The Mists selected GroupFinder consumer references the specific-overall function, which remains absent on current master. The ChallengesUI score branch and GroupFinder applicant tooltip are not exercised because the supplied season-best and applicant models are missing.
