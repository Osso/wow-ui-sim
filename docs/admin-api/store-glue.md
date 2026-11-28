# Store Glue

## `A_Admin.SetStoreGluePurchaseState(guid, productID, state, errors)`

Supply or replace one simulator host record for `C_StoreGlue.GetVASPurchaseStateInfo`. This setter changes simulator state only. It does not connect to a store service or invent a response.

- `guid`: string in a GUID form recognized by the Store Glue model. An unrecognized GUID is rejected.
- `productID`: signed 32-bit product identifier.
- `state`: signed 32-bit purchase state.
- `errors`: dense array of signed 32-bit error values. The order is preserved by this setter.

Call the setter once for each record. The original CharacterSelect consumer reads distinct supplied records and the API returns `(productID, state, errors)`. Retail behavior is native-evidenced; Forever applicability is inferred from the selected original consumer, not from a Forever API declaration. See [Store Glue VAS state](../specs/store-glue-r88q.md) for the evidence boundary.

```lua
A_Admin.SetStoreGluePurchaseState("Player-1-a1", 2003, 3, {17, 0, -4})
A_Admin.SetStoreGluePurchaseState("Player-2-b2", 189, 4, {0, 29})
```
