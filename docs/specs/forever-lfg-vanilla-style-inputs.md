# Forever Vanilla-style Group Finder inputs

`Blizzard_GroupFinder_VanillaStyle` (`## AllowLoadGameType: classic, camelot`) loads on Forever. Its file-load and `OnLoad` paths need the following surface.

- **Events.** VERIFIED (shipped Lua): Forever 70707 declares `LFG_LIST_ROLE_UPDATE` (`LFGListRolesDocumentation.lua:53`) and `LFG_LIST_SHOW_SEARCH` (`LFGListInfoDocumentation.lua:818`). `LFGListingFrame` and `LFGBrowseFrame` register them in `OnLoad`. Both are registerable on Forever.
- **`C_FriendList.GetWhoRaceFilters()`.** VERIFIED (shipped Lua): `FriendListDocumentation.lua:219` returns a non-nil array of `WhoFilter { name, ID }`. `Mainline/WhoList.lua:388` sorts the result in place during `LFGWhoListFrame` `OnLoad`. Each call returns a fresh array. UNVERIFIED (best guess): the rows are the 13 core playable races of the simulator's character-create race fixture, with its ChrRaces IDs (Human 1 … Draenei 11, Worgen 22, Pandaren 24). The native Forever roster is not established.
- **`C_LFGList.GetActivityInfoTable` level fields.** VERIFIED (shipped Lua): `GroupFinderActivityInfo` declares `minLevel`, `minLevelSuggestion` and `maxLevelSuggestion` non-nil on Retail, PTR, Mists and Forever. `LFGUtil_GetFilteredActivities` (`Blizzard_LFGVanilla_ParentFrame.lua:259`) compares them with the player level. INFERRED (best guess): `minLevel` and `minLevelSuggestion` are the supplied activity minimum level; `maxLevelSuggestion` is 0, which the consumer's `> 0` guard reads as no upper suggestion.

Behavioral coverage: `addon_coverage::all_blizzard_addon_load_errors_are_tracked_per_addon_name` and the load-on-demand shard that loads this addon hold its error count at 0.
