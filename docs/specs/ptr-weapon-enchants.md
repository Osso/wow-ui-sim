# PTR weapon enchant snapshots

PTR publishes `C_Item.GetWeaponEnchantInfo` from the existing per-slot supplied weapon enchant state. The pinned PTR `Blizzard_APIDocumentationGenerated/ItemDocumentation.lua:1141-1154` declares the query and `ItemConstantsDocumentation.lua:819-828` declares its slots. `Blizzard_BuffFrame/BuffFrame.lua:763-766` consumes it during `WEAPON_ENCHANT_CHANGED` updates. Retail has no matching pinned declaration and does not publish this query. Forever retains its existing publication.

Empty slots are simulator scenario defaults, not claims about native state. Supplied charges and remaining time must appear in the BuffFrame enchant entry after the event.
