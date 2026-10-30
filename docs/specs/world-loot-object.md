# World loot objects

Retail, PTR and Forever expose the eight `C_WorldLootObject` functions declared in their pinned `Blizzard_APIDocumentationGenerated/WorldLootObjectDocumentation.lua:9-124`; the info fields and synchronous GUID event are declared at lines 126-150. Other profiles do not receive these functions. The retired `GetCurrentWorldLootObjectSwapInventoryType` remains absent.

The host supplies GUID-keyed records with one unit token, inventory type, quality and upgrade flags, squared distance and range. Replacing a record fires `WORLD_LOOT_OBJECT_INFO_UPDATED` synchronously with its GUID after mutation. A token belongs to at most one record. Queries return snapshots; unknown tokens or GUIDs return false for predicates, nil for distance and no values for info. Clicks record the supplied token and button without further events; this is not a complete pickup model.

Plunderstorm slot matching maps offensive inventory type 31 to slots 212 and 213 and utility inventory type 32 to slots 216 and 217. Other pairs return false. Retail, PTR and Forever accept `PLAYER_EQUIPED_SPELLS_CHANGED`. Supplying a changed equippable player world-loot record emits it without arguments. `A_Admin.EquipItem` and `UnequipItem` update the existing player equipment model and emit the event when an offensive or utility slot changes; `GetInventoryItemID` and `GetInventoryItemQuality` read that state. Pickup outcomes are not modeled.

## Best-guess basis

**Best-guess, native UNKNOWN.** Pinned Retail `Blizzard_Plunderstorm/GameModeConstants.lua:2-9` supplies the slot numbers; `Blizzard_APIDocumentationGenerated/ItemConstantsDocumentation.lua:209-210` supplies the type numbers. `Blizzard_ActionBar/WoWLabs/ActionButtonOverrides.lua:2-7,229-235` passes those slots to the matching query. Blizzard's [Plunderstorm guide](https://worldofwarcraft.blizzard.com/en-us/news/24073142/shiver-me-timbers-make-ready-for-the-incoming-plunderstorm) describes two offensive and two utility spell slots. This does not establish behavior for ordinary equipment slots.

Pinned `Blizzard_UIWidgets/Blizzard_UIWidgetTemplateBase.lua:467-474,678-683` registers the event for equippable loot and schedules an update; its native payload and timing are unknown. The no-payload event on supplied changed player loot is a simulator best guess. The empty store, unknown-input results, unique-token replacement and click recording remain unverified; click recording does not model pickup outcomes.

Pinned consumer reads in this lane: `Blizzard_UIWidgets/Blizzard_UIWidgetTemplateBase.lua:675-680` calls `IsWorldLootObject` and `GetWorldLootObjectInfo`; `Blizzard_UIWidgetTemplateSpellDisplay.lua:114-118` calls `OnWorldLootObjectClick`; `Blizzard_ActionBar/WoWLabs/ActionButtonOverrides.lua:229-235,254-262,462-470` calls `DoesSlotMatchInventoryType`, `GetInventoryItemQuality` and `GetInventoryItemID`. The API declarations for the world-loot queries are in `WorldLootObjectDocumentation.lua:9-124`.

## Known gaps

Click outcomes, ability rank and upgrade rules, selected replacement slot, and dropped-object creation are outside this bounded model. Native event payload and timing are unknown.
