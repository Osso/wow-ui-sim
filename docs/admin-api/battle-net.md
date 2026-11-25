# Battle.net Account State

These setters supply local account data for Retail and WoW Forever scenarios. Values do not connect to Battle.net or change a real account.

## `A_Admin.SetPlayerBattleNetInfo(battleTag[, accountID[, broadcastText[, isAFK[, isDND[, isRealIDEnabled[, isAppearOffline[, connected[, featuresEnabled]]]]]]]])`

Supplies the local account record and connection state used by legacy Battle.net globals. Calling this setter supplies an account record; omit the call to model no account. Values do not connect to Battle.net or change a real account.

- `battleTag`: string or nil. Nil models a null record string; an empty string remains an empty string.
- `accountID`: numeric Battle.net account ID; defaults to `0`.
- `broadcastText`: account broadcast text; defaults to an empty string.
- `isAFK`, `isDND`, `isRealIDEnabled`, `isAppearOffline`: booleans; default to `false`.
- `connected`: supplied connectivity gate; defaults to `false`.
- `featuresEnabled`: supplied connection-mode gate; defaults to `true` (normal mode).

**INFERRED — R88fi, Retail 12.1.5.70688 and Forever 1.60.1.70707:** `BNFeaturesEnabled()` reads `featuresEnabled`; it does not require an active connection. `BNConnected()` and `BNFeaturesEnabledAndConnected()` return the same `featuresEnabled and connected` predicate. `BNGetInfo()` returns no values when features are disabled or no account record was supplied. Otherwise it returns, in order, nil deprecated presence ID, BattleTag, account ID, broadcast text, AFK, DND, Real ID enabled, and appear offline. The BattleTag is nil only when the supplied record string is nil; an empty string remains a string.

```lua
A_Admin.SetPlayerBattleNetInfo("Malfurion#1234", 42, nil, nil, nil, nil, nil, true, true)
local _, battleTag = BNGetInfo()
-- battleTag == "Malfurion#1234"
```

## `A_Admin.SetBattleNetAccountFeatures([friendTags[, titleFriendCustomNames[, titleFriends[, friendsListEnabled[, friendsListSupported]]]]])`

Supplies the five account feature results used by the Retail/Forever social UI. Omitted values default to `true`. **INFERRED (`R88fi (native evidence pending)`):** each API returns its matching supplied boolean.

The arguments map to `C_BattleNet.AreFriendTagsEnabled`, `AreTitleFriendCustomNamesEnabled`, `AreTitleFriendsEnabled`, `IsBattleNetFriendsListEnabled`, and `IsBattleNetFriendsListSupported` in that order.

## `A_Admin.SetBattleNetFriendInvite(index, inviteID, accountName, battleTag, friendLevel, creationTimestamp)`

Supplies one pending Battle.net friend invite. `index` must replace an existing invite or append the next invite; other indices raise an error. The values map to the record fields used by `C_BattleNet.GetFriendInviteInfo(index)`. `BNGetNumFriendInvites()` returns the number of supplied invites.

## `A_Admin.SetBattleNetFriendState(index, isFriend, isOnline, isFavorite)`

Supplies the friend, online, and favorite flags for an existing `BnetFriend` entry. The setter updates the `is_online` flag on each attached game account; it does not create game accounts. The index must identify an existing entry.

**INFERRED — R88g2 (native evidence pending):** `BNGetNumFriends()` returns four values in this order: total friends, online friends, favorite friends, and favorite online friends. Counts include supplied `BnetFriend` entries marked `is_friend`. A friend is online when at least one attached game account is online. The default fixture supplies two friends, one online and one favorite, so the counts are `(2, 1, 1, 0)`.

**INFERRED — R88g2 (native evidence pending):** `BNSetSelectedFriend(index)` stores the selected 1-based Battle.net friend index used by the original FriendsFrame.
