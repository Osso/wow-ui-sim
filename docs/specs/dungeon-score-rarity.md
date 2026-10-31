# Dungeon score rarity colours

## Contract and native basis

Retail and PTR read the generated, per-profile pinned-CDN `UIDungeonScoreRarity` cache. Missing, invalid or unreadable cache data is an error; the missing-cache error names `wow-cli casc sync-dungeon-score-rarity`. No other source supplies rows.

Retail client build 70688 behavioural evidence establishes a linear scan in table order. A row matches when its PlayerCondition gate passes and its family bounds contain the input in `[min, max)`. A maximum of zero is unbounded. The first match supplies the colour. Field 11 is the condition ID; zero means no condition. The simulator evaluates the pinned CDN conditions as follows:

- 96877: time event 248 passed.
- 96879: time event 121 passed and 248 has not passed.

Type 289 tests membership in the passed-time-event snapshot. Unknown nonzero conditions produce an explicit error rather than an invented result. The families use `DungeonScoreMin/Max`, `KeystoneLevelMin/Max`, fields 007/008 (single run), and fields 009/010 (single dungeon overall), respectively.

Retail `Blizzard_GroupFinder_Mainline.toc` selects `Mainline/LFGList.lua`; its search-result tooltip calls the overall API at line 4470. The consumer test loads the addon closure and checks the function source and rendered score.

## Best-guess basis

**Best-guess, native UNKNOWN:** `SimState` starts with passed events `{121, 248}`. The client reads a supplied snapshot; its producer and current live contents are unknown. This default enables condition 96877 and is not a claim about live server state. A no-match scan returns nil; the 70688 no-colour Lua return path was not fully resolved. RGB unpacking and the ColorMixin result retain the existing simulator behaviour; native payload packing was not established by the behavioural evidence.

## Cache generation

Generate each active profile's persistent cache with:

```text
wow-cli casc sync-dungeon-score-rarity
```

The default path is the user cache under `wow-ui-sim/dungeon-score-rarity/<profile>/rarity.json`. Release packages and Docker images bundle generated data beside the executable.

## Known gaps

The simulator decodes only the two pinned dungeon rarity PlayerConditions above, not arbitrary PlayerCondition trees. It does not model the native snapshot producer or change notifications. Native argument conversion and the exact no-colour Lua result require further evidence. The 70688 client evidence and the pinned Retail 70575 CDN condition trees are distinct builds; PTR condition trees have not been independently verified. Forever, Mists, Era and Anniversary do not publish these implementations; their pinned rarity tables contain no rows. The ChallengesUI score branch and GroupFinder applicant tooltip remain untested. The search-result tooltip passes `bestRunLevel` to the specific-overall function; native intent for that call remains unknown.
