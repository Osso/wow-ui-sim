# Transmog outfit test input

Forever scenarios can set outfit capacity before creating records through the real `C_TransmogOutfitInfo.AddNewOutfit` API:

```lua
A_Admin.SetUnlockedOutfitSlots(2, 2)
C_TransmogOutfitInfo.AddNewOutfit("Copper", 123)
C_TransmogOutfitInfo.AddNewOutfit("Silver", 456)
```

`SetUnlockedOutfitSlots(source, count)` replaces one capacity value. `source` must be `0`, `1`, or `2`; `count` must be a non-negative integer that fits `u32`. This changes only the simulator's unlocked capacity input. It does not create outfits or emit events. It is simulator-only and does not model native unlock policy.

See the [Forever spellbook contract](../specs/forever-camelot-spellbook.md) for the selected Player Spells consumer proof.

## Forever publication

Forever publishes the shared outfit model for every `C_TransmogOutfitInfo` member that its pinned `TransmogOutfitInfoDocumentation.lua` declares, plus `C_Transmog.GetSlotVisualInfo` and `C_Item.CanItemTransmogAppearance`. Forever names the option pair `GetOptionsForSlot` (rows carry `type`) and `SetViewedOptionForSlot` (fires `VIEWED_TRANSMOG_OUTFIT_SLOT_OPTION_CHANGED`); its slot options run to 15 (`ArmorPlate`).

Exceptions: `ChangeToOutfit`, `ClearOutfit`, `GetActiveOutfitID`, `GetCurrentlyViewedOutfitID`, `GetAllSlotLocationInfo`, `GetLinkedSlotInfo`, `GetTransmogOutfitSlotFromInventorySlot` and the two sheathe members stay on the temporary Lua fallback (`transmog_outfit_slot_defaults`), whose lib tests pin that behaviour on Forever. INFERRED R88g9 (native evidence pending): Forever reuses the Retail option catalog for `GetOptionsForSlot` and offers no armor-type options.
