# Forever item-button helper ownership

**VERIFIED (Forever 1.60.1.70707):** the selected original
`Blizzard_ItemButton/Mainline/ItemButtonTemplate.lua` publishes
`SetItemButtonCount` (line 9), `SetItemButtonTexture` (99),
`SetItemButtonTextureVertexColor` (116), and
`SetItemButtonNormalTextureVertexColor` (151). The addon TOC selects the
Mainline family file; the Forever test checks where all four functions were
actually loaded from. No simulator-side Lua or Rust startup caller of these
helpers was found before loading the provider.

The original `ItemButtonMixin:Reset`
(`Blizzard_ItemButton/Shared/ItemButtonTemplate.lua:113`) resets a populated
count to zero and hides its font string. With no empty-background texture
configured, it also clears and hides the icon. The count/texture/color
contract is checked through the selected original helpers on an actual
`ItemButton`.

**Unchanged:** Retail already used the original provider. Other profiles'
bootstrap defaults remain pending separate evidence. The independent
`CreateFrame("ItemButton")` wrapper is not a verified duplicate and remains
unchanged.
