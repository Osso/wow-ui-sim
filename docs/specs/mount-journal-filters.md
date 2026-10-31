# Mount journal filters

Retail `C_MountJournal` maintains independent collected, not-collected, unusable, source, and mount-type selections. `IsUsingDefaultFilters` reports whether those selections are at their defaults. `SetDefaultFilters` restores all of them. The displayed mount count, displayed IDs, and displayed info use the same selections. The Blizzard Collections filter dropdown uses the default-state and reset APIs for its reset button.

Search remains separate from the dropdown filter selections. Resetting the filters does not change the search text. Changes that alter the displayed mount list enqueue `MOUNT_JOURNAL_SEARCH_UPDATED`. Filter choices live for the simulator session and are not persisted.

## Best-guess basis — best-guess, native UNKNOWN

Native default values, reset coverage, search participation, event behavior, persistence, and profile differences are unknown. The Retail scenario defaults enable collected, not-collected and unusable mounts, all sources, and all types. This is a best guess, not a measured native fresh-account state. Mount Journal Enhanced resets collected, not-collected, unusable and all sources to enabled: https://github.com/exochron/MountJournalEnhanced/blob/master/MountJournalEnhanced.lua . Blizzard forum support recommends checking all three categories, all types and all sources when a mount is missing: https://us.forums.blizzard.com/en/wow/t/i-got-a-mount-drop-used-it-but-it-doesnt-show-up-in-my-mount-journal/1790609 . Neither establishes fresh-account defaults.

The separate search box motivates the best guess that search is excluded from default-filter comparison and reset. The original consumer handles `MOUNT_JOURNAL_SEARCH_UPDATED` when refreshing its list; emitting it for changed results is a best guess about native behavior. This model does not claim native persistence or parity for other profiles.

The existing mount fixture publishes every source as Drop; source filtering follows that published source. Its fixture records ground and flying filter categories separately from raw type IDs 230 and 248.

## Pinned declarations and consumer

Retail `Blizzard_APIDocumentationGenerated/MountJournalDocumentation.lua:86-98,420-455,518-550,564-593` declares the filter getters, setters, reset and search. `Blizzard_Collections/Mainline/Blizzard_MountCollection.lua:184-193,221-259,952-985` binds the dropdown and search. `Blizzard_Collections/Blizzard_Collections_Mainline.toc:5-6,10,14` selects the load-on-demand consumer. `Blizzard_Menu/MenuTemplates.lua:855-905` defines reset callback and visibility behavior.
