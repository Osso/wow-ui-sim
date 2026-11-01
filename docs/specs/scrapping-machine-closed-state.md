# Closed scrapping machine

Retail, PTR and WoW Forever start with no scrapping machine open and no pending items. `C_ScrappingMachineUI.HasScrappableItems` returns false; `GetCurrentPendingScrapItemLocationByIndex` accepts a numeric index and returns no value. `CloseScrappingMachine`, `RemoveAllScrapItems`, and `ValidateScrappingList` leave the closed, empty state unchanged.

## Best-guess basis (best-guess, native UNKNOWN)

Retail, PTR and WoW Forever `Blizzard_APIDocumentationGenerated/ScrappingMachineUIDocumentation.lua:11,25,60,69,91` declare these methods and the pending lookup's numeric argument and optional result. `Blizzard_ScrappingMachineUI.lua:1,35–37,77–78,89–91,118–121,137` uses them. The PTR and WoW Forever `Blizzard_ScrappingMachineUI.toc` files have no game-type or screen exclusions and select the XML that loads this Lua consumer. The closed and empty scenario is the simulator's supplied state, not a claim about native behavior for an open machine.

## Known gaps

Opening a machine and the frame's OnShow need machine-identity data and are not covered. `GetScrappingMachineName` and `GetScrapSpellID` remain unavailable. Item insertion, scrapping, and native state transitions are not modeled.
