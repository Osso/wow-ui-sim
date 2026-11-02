# Item spell query

## What it must do

- Retail `C_Item.GetItemSpell(itemInfo)` selects the first nonzero trigger-zero item spell in legacy slot order and returns its SpellName text and spell ID. Duplicate or invalid association slot indices reject that association set.
- When the association set has no selected spell, the supplied equipped item's enchantment can provide its first type-seven spell. A missing selected SpellName or no selection returns zero values.
- `ChatFrameUtil.InsertLink` can use the result to choose the `/use` macro branch for an item with a spell and the `/equip` branch for an item without one.

## Best-guess basis

Retail `Blizzard_APIDocumentationGenerated/ItemDocumentation.lua:948` declares the input and outputs. WM34-A native evidence for retail 12.1.5/70688 identifies selection and lookup; zero-value missing-name publication is emulated, not live-observed, and successful return arity is inferred. ItemEffect, ItemXItemEffect, SpellItemEnchantment and SpellName data are generated from the 12.1.5.70391 CSV exports.

## Known gaps

- Modified-item effect collections, bag-held object enchantments, all thirteen enchantment slots, object resolution order and embedded effect keys are not fully modeled. The current object path covers the equipped record's single supplied enchantment.
- Eligibility changes, live loading and full protected-wrapper return behavior remain unverified.
