# Communities

These inputs control community UI scenarios on Retail and WoW Forever.

### `A_Admin.SetAccountMutedClub(clubId, muted)`

Set whether the account cannot use chat in one club. `clubId` is an opaque club ID; `muted` is a boolean.

### `A_Admin.SetClubTypeAllowed(clubType, allowed)`

Set whether a club type is allowed in community flows. `clubType` is an `Enum.ClubType` value; `allowed` is a boolean.

BattleNet and Character follow the supplied allowed-type set. Guild is always allowed and Other is always rejected, regardless of the set. This policy is INFERRED — R88fq, Retail 12.1.5.70688 and Forever 1.60.1.70707.

```lua
A_Admin.SetClubTypeAllowed(Enum.ClubType.Character, false)
assert(not C_Club.ShouldAllowClubType(Enum.ClubType.Character))
A_Admin.SetClubTypeAllowed(Enum.ClubType.Guild, false)
assert(C_Club.ShouldAllowClubType(Enum.ClubType.Guild))
A_Admin.SetClubTypeAllowed(Enum.ClubType.Other, true)
assert(not C_Club.ShouldAllowClubType(Enum.ClubType.Other))
```
