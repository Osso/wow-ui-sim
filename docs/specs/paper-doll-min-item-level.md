# Paper doll minimum item level

`C_PaperDollInfo.GetMinItemLevel()` returns one nil when no level-scaling floor is modeled. It is available on Retail, PTR, Forever, Mists, Era, and Anniversary.

## Best-guess basis — native UNKNOWN

The pinned Retail `Blizzard_APIDocumentationGenerated/PaperDollInfoDocumentation.lua:192-198` declares a nullable number; PTR and Forever declare the same at lines 201–207, and Mists, Era and Anniversary at lines 75–81. [Warcraft Wiki](https://warcraft.wiki.gg/wiki/API_C_PaperDollInfo.GetMinItemLevel) also lists a nullable number. Retail `Blizzard_UIPanels_Game/Mainline/PaperDollFrame.lua:1299-1302` applies `max(minItemLevel or 0, avgItemLevelEquipped)`. The nil no-floor scenario is a best guess, not a native observation.

## Known gaps

Timewalking and other level-scaling display floors are not modeled. Native behavior and the circumstances that set a floor are unknown.
