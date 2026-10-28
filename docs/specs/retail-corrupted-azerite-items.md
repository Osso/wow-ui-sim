# Retail corrupted and Azerite items

Retail publishes `C_Item.IsCorruptedItem(ItemInfo)` and `C_AzeriteItem.IsAzeriteItem(ItemLocation)`, as declared in `Blizzard_APIDocumentationGenerated/ItemDocumentation.lua:1195` and `AzeriteItemDocumentation.lua:77` in the pinned Retail UI cache. Both accept secret arguments only when the caller is untainted.

Corruption is supplied by item ID. Azerite identity is supplied by the existing active Azerite item location. An item not marked by supplied scenario state returns false; this is a simulator scenario default, not a claim about native item data.

The original Retail `SetItemButtonOverlay` and `AzeritePaperDollItemOverlayMixin:SetAzeriteItem` consumers must reflect distinct supplied values. Other profiles are outside this contract.
