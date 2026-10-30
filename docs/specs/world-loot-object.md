# World loot objects

Retail, PTR and Forever expose the eight `C_WorldLootObject` functions declared in their pinned `Blizzard_APIDocumentationGenerated/WorldLootObjectDocumentation.lua:9-124`; the info fields and synchronous GUID event are declared at lines 126-150. Other profiles do not receive these functions. The retired `GetCurrentWorldLootObjectSwapInventoryType` remains absent.

The host supplies GUID-keyed records with one unit token, inventory type, quality and upgrade flags, squared distance and range. Replacing a record fires `WORLD_LOOT_OBJECT_INFO_UPDATED` synchronously with its GUID after mutation. A token belongs to at most one record. Queries return snapshots; unknown tokens or GUIDs return false for predicates, nil for distance and no values for info. Clicks record the supplied token and button without further events; this is not a complete pickup model.

Plunderstorm slot matching maps offensive inventory type 31 to slots 212 and 213 and utility inventory type 32 to slots 216 and 217. Other pairs return false. Retail, PTR and Forever accept `PLAYER_EQUIPED_SPELLS_CHANGED`; supplying a changed equippable player world-loot record emits it without arguments. Equipped-spell changes and pickup outcomes are not yet modeled.

## Best-guess basis

**Best-guess, native UNKNOWN.** Pinned Retail `Blizzard_Plunderstorm/GameModeConstants.lua:2-9` supplies the slot numbers; `Blizzard_APIDocumentationGenerated/ItemConstantsDocumentation.lua:209-210` supplies the type numbers. `Blizzard_ActionBar/WoWLabs/ActionButtonOverrides.lua:2-7,229-235` passes those slots to the matching query. Blizzard's [Plunderstorm guide](https://worldofwarcraft.blizzard.com/en-us/news/24073142/shiver-me-timbers-make-ready-for-the-incoming-plunderstorm) describes two offensive and two utility spell slots. This does not establish behavior for ordinary equipment slots.

Pinned `Blizzard_UIWidgets/Blizzard_UIWidgetTemplateBase.lua:467-474,678-683` registers the event for equippable loot and schedules an update; its native payload and timing are unknown. The no-payload event on supplied changed player loot is a simulator best guess. The empty store, unknown-input results, unique-token replacement and click recording remain unverified; the latter is incomplete because no equipped-spell inventory or click outcome is modeled.
