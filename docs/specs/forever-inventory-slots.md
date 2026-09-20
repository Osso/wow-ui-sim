# Forever inventory slot namespace

## Contract

- Forever exposes `C_PaperDollInfo.GetInventorySlotInfo` and `GetInventorySlotInfoForInvSlot` through the existing slot lookup implementation, without enabling a retail API epoch.
- Existing slot IDs, icon file IDs, false `checkRelic`, three-value known-slot returns and nil unknown-slot behavior remain unchanged.
- Other profiles retain their existing publication policy.

## Evidence and tests

Forever build `1.60.1.69913` publishes both methods in `Blizzard_APIDocumentationGenerated/PaperDollInfoDocumentation.lua`. Its `Blizzard_UIPanels_Game/Camelot/PaperDollFrame.lua` calls the first before `SetID` in `PaperDollItemSlotButton_OnLoad`.

`tests/wowforever_inventory_slots.rs` exercises the real vendor OnLoad using Head, MainHand and Ammo buttons, plus both namespace methods and unknown-slot behavior. This reuses simulator slot data; it does not establish native conformance for untested slots or argument coercion.
