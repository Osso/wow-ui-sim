# Merchant items

`A_Admin.SetMerchantItems(items)` replaces the merchant offer item IDs used by the Retail and WoW Forever merchant model. Pass one positive item ID or an array of item IDs. Array entries that are not positive numbers are ignored. An empty array clears the list.

This input supplies offer identities only. It does not supply prices, costs, refundability, merchant-open state, or item eligibility. The selected Blizzard `MerchantFrame` consumer applies its filter to these supplied offers.

```lua
A_Admin.SetMerchantItems({ 6948, 1728 })
```
