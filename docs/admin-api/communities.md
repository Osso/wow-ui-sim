# Communities

These inputs control community UI scenarios on Retail and WoW Forever.

### `A_Admin.SetAccountMutedClub(clubId, muted)`

Set whether the account cannot use chat in one club. `clubId` is an opaque club ID; `muted` is a boolean.

### `A_Admin.SetClubTypeAllowed(clubType, allowed)`

Set whether a club type is allowed in community flows. `clubType` is an `Enum.ClubType` value; `allowed` is a boolean.

INFERRED — R88fq, Retail 12.1.5.70688 and Forever 1.60.1.70707: the simulator retains its existing default set of BattleNet and Character because native initial values are UNKNOWN; BattleNet and Character follow that supplied set, Guild is always allowed, and Other and unrecognized types are always rejected, regardless of the set.

```lua
A_Admin.SetClubTypeAllowed(Enum.ClubType.Character, false)
assert(not C_Club.ShouldAllowClubType(Enum.ClubType.Character))
A_Admin.SetClubTypeAllowed(Enum.ClubType.Guild, false)
assert(C_Club.ShouldAllowClubType(Enum.ClubType.Guild))
A_Admin.SetClubTypeAllowed(Enum.ClubType.Other, true)
assert(not C_Club.ShouldAllowClubType(Enum.ClubType.Other))
A_Admin.SetClubTypeAllowed(9999, true)
assert(not C_Club.ShouldAllowClubType(9999))
```

### `A_Admin.SetDiscordServers(servers)` and `A_Admin.SetDiscordUser(userID, userName)`

Retail 12.1+ and WoW Forever. `SetDiscordServers` replaces the server list read by `C_Discord.GetNumDiscordServers`, `GetServerName`, `GetNumDiscordChannels(serverIndex)`, `GetDiscordChannelName(serverIndex, channelIndex)` and `GetServerLinkableChannels(serverIndex)`. Each server needs a string `name` and a `channels` array; each channel needs a string `name` and may set `linkable = false`. A guild link that no longer names a supplied channel is cleared. `SetDiscordUser` supplies `C_Discord.GetDiscordUserID()` and the name `GetDiscordUserName(userID)` returns. The setters fire no events.

INFERRED R88g9 (native evidence pending): unknown indices give `0, false` channel counts and empty names; an unlinked guild reports `false, "", ""`; `GuildLink` with unknown indices keeps the current link; no supplied user gives `""`.

```lua
A_Admin.SetDiscordServers({ { name = "Guild Hall", channels = { { name = "general" } } } })
C_Discord.GuildLink(1, 1)
assert(select(3, C_Discord.GetGuildLinkStatus()) == "Guild Hall")
```
