# Latest transmog appearance

`C_TransmogCollection.GetLatestAppearance` returns no values before an appearance is collected in the simulator session. `A_Admin.AddTransmogAppearance` records the new appearance's visual ID and category; subsequent queries return that pair. A later collection replaces the pair. The query is published for Retail, PTR and WoW Forever.

## Best-guess basis (best-guess, native UNKNOWN)

The pinned Retail `Blizzard_APIDocumentationGenerated/TransmogItemsDocumentation.lua:553–562`, and PTR and WoW Forever versions at lines 554–563, declare a query that may return nothing or a visual ID and category. The Retail `Blizzard_Collections/Mainline/Blizzard_Wardrobe.lua:546–556` consumes the pair. Recording the latest simulator collection and starting with no value are best guesses; native behavior is unverified.

## Known gaps

Native update and clear timing, including persistence across sessions or reloads, is unknown. The legacy `A_Admin.AddTransmog` path supplies no visual/category pair and does not set the latest appearance.
