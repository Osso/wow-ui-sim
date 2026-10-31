# Mount journal filters

Retail filter state is session-local. Fresh collected, source, and type masks are zero. `SetDefaultFilters` sets collected mask `3`, the valid source bits among indices 0–11, and type mask `0x1f`. It leaves search unchanged. `IsUsingDefaultFilters` compares those three masks exactly and ignores search.

Getters use zero-based bit indices. Source checks require both a set bit and valid source data. Type validity accepts 0–31 except 3; setting type 3 or an out-of-range type raises an error. Bulk type reset nevertheless sets bit 3. Collected setters accept 0–3; invalid source setters do nothing. Bulk operations replace the source and type masks rather than preserving higher bits.

The original Collections consumer registers `SetDefaultFilters` as a reset callback, but does not call it from `MountJournal_OnLoad`. Thus loading the consumer does not itself initialize the filters.

## Native basis (70688 behavioural spec)

The clean-room evidence is `native-evidence-70688/C_MountJournal_FilterState.md` and the function-specific files `GetCollectedFilterSetting.md`, `IsSourceChecked.md`, `IsTypeChecked.md`, `IsValidSourceFilter.md`, `IsValidTypeFilter.md`, `IsUsingDefaultFilters.md`, `SetDefaultFilters.md`, `SetSearch.md`, `SetSourceFilter.md`, `SetTypeFilter.md`, `SetCollectedFilterSetting.md`, `SetAllSourceFilters.md`, and `SetAllTypeFilters.md`. These describe build 70688, whereas the pinned Retail UI is 70575.

## Known gaps

- Event delivery timing relative to setter return is unknown. The simulator retains queued `MOUNT_JOURNAL_SEARCH_UPDATED` behavior when displayed IDs change; native listener notifications can occur even without a displayed-list change.
- Native valid-source data is runtime-dependent and not supplied by the evidence. The simulator fixture currently publishes only Drop, so its valid-source mask contains only that source; it does not model the native data refresh.
- Exact type-error text for invalid collected indices and search argument errors is unknown.
- Verified (M8): `LE_MOUNT_JOURNAL_FILTER_COLLECTED/NOT_COLLECTED/UNUSABLE` are 1/2/3 and `NUM_LE_MOUNT_JOURNAL_FILTERS` is 3. The collected getter tests bit == index for 0–3 and returns false above 3. Blizzard_MountCollection passes the constants directly, so after `SetDefaultFilters` (bits 0 and 1) the original UI reports Collected checked and Not Collected/Unusable unchecked.
- Which native bit the displayed-list filter reads for collected, uncollected and unusable mounts is unknown. The simulator filters with the UI constants (1, 2, 3).
- The 70575-to-70688 behavior differences require native confirmation.

## Pinned declarations and consumer

Retail `Blizzard_APIDocumentationGenerated/MountJournalDocumentation.lua:86-98,420-455,518-550,564-593` declares the filter getters, setters, reset and search. `Blizzard_Collections/Mainline/Blizzard_MountCollection.lua:137-193,221-259,952-985` binds the dropdown and search. `Blizzard_Collections/Blizzard_Collections_Mainline.toc:5-6,10,14` selects the load-on-demand consumer. `Blizzard_Menu/MenuTemplates.lua:855-905` defines reset callback and visibility behavior.
