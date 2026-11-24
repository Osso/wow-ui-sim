# Battle.net Account State

These setters supply local account data for Retail and WoW Forever scenarios. Values do not connect to Battle.net or change a real account.

## `A_Admin.SetPlayerBattleNetInfo(battleTag[, accountID[, broadcastText[, isAFK[, isDND[, isRealIDEnabled[, isAppearOffline]]]]]])`

Supplies the local identity returned by legacy `BNGetInfo()`.

- `battleTag`: required BattleTag string.
- `accountID`: numeric Battle.net account ID; defaults to `0`.
- `broadcastText`: account broadcast text; defaults to an empty string.
- `isAFK`, `isDND`, `isRealIDEnabled`, `isAppearOffline`: booleans; default to `false`.

The simulator returns a nil deprecated presence ID, then these values in the order read by Blizzard UI. The return shape and default values are **INFERRED** (`R88fi (native evidence pending)`).

```lua
A_Admin.SetPlayerBattleNetInfo("Malfurion#1234", 42)
local _, battleTag = BNGetInfo()
-- battleTag == "Malfurion#1234"
```

## `A_Admin.SetBattleNetAccountFeatures([friendTags[, titleFriendCustomNames[, titleFriends[, friendsListEnabled[, friendsListSupported]]]]])`

Supplies the five account feature results used by the Retail/Forever social UI. Omitted values default to `true`. These defaults are **INFERRED** (`R88fi (native evidence pending)`).

The arguments map to `C_BattleNet.AreFriendTagsEnabled`, `AreTitleFriendCustomNamesEnabled`, `AreTitleFriendsEnabled`, `IsBattleNetFriendsListEnabled`, and `IsBattleNetFriendsListSupported` in that order.
