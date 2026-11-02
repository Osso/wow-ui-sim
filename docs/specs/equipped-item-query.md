# Equipped item query

## What it must do

- [ ] Retail `C_Item.IsEquippedItem(itemInfo)` compares the shared ItemInfo ID with the supplied player equipment records and returns one boolean. Item ID zero does not match.
- [ ] The original secure item action can pass the link returned by `SecureCmdItemParse` and take its already-equipped branch.
- [ ] ItemInfo secret access follows the shared VM restriction.

## Best-guess basis

Retail `Blizzard_APIDocumentationGenerated/ItemDocumentation.lua:1302` declares the ItemInfo argument and boolean return. `Blizzard_FrameXML/SecureTemplates.lua:430` consumes the query. PTR 12.1.7 native evidence in `native-evidence-12.1.7/C_Item.IsEquippedItem.md` shows a context-dependent equality search and emulated boolean publication, but does not identify the compared fields or equipment slots. Comparing supplied equipment item IDs is a simulator best guess, not established native parity. Shared ItemInfo conversion is described in [ItemInfo conversion](item-info-conversion.md).

## Implementation inventory

- `src/c_api/item_spell/c_item.rs`: Retail registration and query.
- `SimState.player.equipped_items`: supplied equipment records.

## Tests asserting this spec

- `tests/item_equipped_consumer.rs` checks the unchanged secure item action with an inventory link and a different supplied equipment state.

## Known gaps

- [ ] Native search eligibility, full slot selection, nested containers, item identity, invalid arguments and unknown-item behavior remain unverified. PTR and other profiles are not implemented by this contract.
