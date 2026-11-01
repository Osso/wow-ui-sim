# Numeric color text

Rust-created color objects accept numeric text in `WrapTextInColorCode`, convert it using Lua number formatting, and wrap it with the object's color markup. String input remains supported; other input types remain errors.

## Best-guess basis (best-guess, native UNKNOWN)

The pinned Retail consumer `Blizzard_HousingMarketCart/Blizzard_HousingMarketCartItemTemplates.lua:26–35` passes numeric prices to this method. `Blizzard_SharedXMLBase/Color.lua:68–69` delegates the method to `C_ColorUtil.WrapTextInColor`, whose Retail declaration is `Blizzard_APIDocumentationGenerated/ColorUtilDocumentation.lua:107–120`. Numeric coercion is inferred from that consumer; native behavior for other values and profiles is not measured.
