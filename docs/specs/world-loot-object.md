# World loot objects

Retail, PTR and Forever expose the eight `C_WorldLootObject` functions declared in their pinned `Blizzard_APIDocumentationGenerated/WorldLootObjectDocumentation.lua:9-124`; the info fields and synchronous GUID event are declared at lines 126-150. Other profiles do not receive these functions. The retired `GetCurrentWorldLootObjectSwapInventoryType` remains absent.

The host supplies GUID-keyed records with one unit token, inventory type, quality and upgrade flags, squared distance and range. Replacing a record fires `WORLD_LOOT_OBJECT_INFO_UPDATED` synchronously with its GUID after mutation. A token belongs to at most one record. Queries return snapshots; unknown tokens or GUIDs return false for predicates, nil for distance and no values for info. Clicks record the supplied token and button without further events. Slot/type matching reads host-supplied observations; an unsupplied pair returns false.

The empty store, unknown-input results, unique-token replacement, click recording and unsupplied slot/type result are user-approved defaults, not native evidence. This model does not infer game-world objects, proximity, equipment compatibility or click outcomes.
