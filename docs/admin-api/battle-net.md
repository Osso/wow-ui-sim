# Battle.net Account State

These setters supply local account data for Retail and WoW Forever scenarios. Values do not connect to Battle.net or change a real account.

## `A_Admin.SetPlayerBattleNetInfo(battleTag[, accountID[, broadcastText[, isAFK[, isDND[, isRealIDEnabled[, isAppearOffline]]]]]])`

Supplies the local identity returned by legacy `BNGetInfo()`.

- `battleTag`: required BattleTag string.
- `accountID`: numeric Battle.net account ID; defaults to `0`.
- `broadcastText`: account broadcast text; defaults to an empty string.
- `isAFK`, `isDND`, `isRealIDEnabled`, `isAppearOffline`: booleans; default to `false`.

**INFERRED (`R88fi (native evidence pending)`):** `BNGetInfo()` returns, in order, nil deprecated presence ID, BattleTag, account ID, broadcast text, AFK, DND, Real ID enabled, and appear offline. On Retail/Forever, a non-empty supplied BattleTag enables the legacy `BNFeaturesEnabled()` consumer path.

```lua
A_Admin.SetPlayerBattleNetInfo("Malfurion#1234", 42)
local _, battleTag = BNGetInfo()
-- battleTag == "Malfurion#1234"
```

## `A_Admin.SetBattleNetAccountFeatures([friendTags[, titleFriendCustomNames[, titleFriends[, friendsListEnabled[, friendsListSupported]]]]])`

Supplies the five account feature results used by the Retail/Forever social UI. Omitted values default to `true`. **INFERRED (`R88fi (native evidence pending)`):** each API returns its matching supplied boolean.

The arguments map to `C_BattleNet.AreFriendTagsEnabled`, `AreTitleFriendCustomNamesEnabled`, `AreTitleFriendsEnabled`, `IsBattleNetFriendsListEnabled`, and `IsBattleNetFriendsListSupported` in that order.
