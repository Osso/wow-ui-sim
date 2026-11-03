# Item use, confirmation and trade drop (Retail)

These `C_Item` members are published on Retail (ItemDocumentation.lua:80, :84, :88, :199, :209, :1703): `ConfirmBindOnUse`, `ConfirmNoRefundOnUse`, `ConfirmOnUse`, `DropItemOnUnit`, `EndBoundTradeable(type)` and `UseItemByName(itemInfo, target)`. The original `USE_BIND`, `CONFIM_BEFORE_USE`, `USE_NO_REFUND_CONFIRM` and `END_BOUND_TRADEABLE` popups, `SecureCmdUseItem` and the secure `target` action call them. WM68 does not establish native registration of `ConfirmBindOnUse` and `DropItemOnUnit`; they are published from the documentation declaration.

## UseItemByName

The item argument converts like `EquipItemByName`: numbers, item links and exact names give an item ID, and an unresolved name gives zero. The optional target is read but not modeled. A missing player, a zero ID or missing item metadata skips the action. The first match is taken from the player root slots, then from the backpack and carried bags, with each bag before its contents. VERIFIED (WM68).

The ordinary path tests four prompts in this order: `USE_BIND_CONFIRM`, `CONFIRM_BEFORE_USE`, `USE_NO_REFUND_CONFIRM`, then `END_BOUND_TRADEABLE`. Their conditions are UNKNOWN, so each item's predicate outcomes are supplied simulator inputs. The first selected prompt saves the item as the pending item and stops. The first three events have no arguments; `END_BOUND_TRADEABLE` has one argument, `itemconsume`. VERIFIED (WM68). Without a prompt, a container item enters the accepted-use request model of `C_Container.UseContainerItem`, including its usability input. Use of an equipped item, the special-item path and the early `END_BOUND_TRADEABLE` path are not modeled.

## Confirmed use

`ConfirmBindOnUse`, `ConfirmOnUse` and `ConfirmNoRefundOnUse` read no arguments. If the pending item resolves to a container slot, the confirmed use enters the same request model and bypasses the four prompts. After dispatch returns, the pending item is cleared, also on a lookup miss or a refusal. VERIFIED (WM44-C, WM68). The item-122284 token prompt, target reselection and restriction notifications are not modeled.

## EndBoundTradeable

`type` is a string. Only `itemconsume` uses the pending item, with the confirmed-use behavior above. The `itemenchant`, `gem` and `spellenchant` branches read pending enchant-target, socket, gem and spell identities that no simulator action produces, so they submit nothing and leave the pending item unchanged. VERIFIED branch split (WM44-D). Behavior for any other `type` is UNKNOWN; the simulator does nothing.

## DropItemOnUnit

The argument is a unit token. Only an owned-item cursor (mode 1) supplies the item, and the unit must resolve. When it is the current trade target, the item goes into the first free ordinary trade slot, 1–6 (INFERRED order). Then `TRADE_PLAYER_ITEM_CHANGED` fires with one argument, the one-based slot, and the shared cursor clear runs. When all six slots are full, the cursor is retained and no event fires. VERIFIED (WM68). Staging a new trade target, the special slot 7, duplicate detection and the give branch are not modeled.

## Unpublished members

These stay known gaps in the 10.2.6 sweep:
- `ActionBindsItem`, `ReplaceEnchant`, `ReplaceTradeEnchant` and `ReplaceTradeskillEnchant` are registered, but they act on pending enchant action, spell and target state that the simulator does not model. Their clearing and results are UNKNOWN (WM68).
- The registration of `BindEnchant` is UNKNOWN, and its confirmed enchant action is unmodeled (WM68).
- The registration of `EndRefund` is UNKNOWN. Its candidate branches select enchant or socket confirmation; they are not a demonstrated refund request or refund-record deletion (WM68).

The simulator never synthesizes an enchant, socket, trade, refund or server result.
