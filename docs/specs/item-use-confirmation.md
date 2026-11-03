# Item-use confirmation actions (Retail)

`C_Item.ConfirmOnUse()`, `C_Item.ConfirmNoRefundOnUse()` and `C_Item.EndBoundTradeable(type)` are published on Retail (ItemDocumentation.lua:88, :84, :209). The original `CONFIM_BEFORE_USE`, `USE_NO_REFUND_CONFIRM` and `END_BOUND_TRADEABLE` popups call them.

## Confirmed use

`ConfirmOnUse` and `ConfirmNoRefundOnUse` read no arguments and share one helper. If the pending item resolves to a container slot, the confirmed use enters the same accepted-use request model as `C_Container.UseContainerItem`, including its usability input. After that dispatch returns, the pending item is cleared. It is also cleared when the item no longer resolves and when the use is refused. VERIFIED (WM44-C).

Confirmation bypasses the use prompts. The item-122284 token prompt, target reselection and the addon/macro restriction notifications are not modeled. Use of an equipped item is not modeled. Exceptional and non-returning paths are UNKNOWN.

## EndBoundTradeable

`type` is a string. Only `itemconsume` uses the pending item, with the confirmed-use behavior above. The `itemenchant`, `gem` and `spellenchant` branches read pending enchant-target, socket, gem and spell identities. No simulator action produces those identities, so these branches submit nothing and leave the pending item unchanged. VERIFIED branch split (WM44-D). Behavior for any other `type` is UNKNOWN; the simulator does nothing.

The simulator never synthesizes an enchant, socket or server result.
