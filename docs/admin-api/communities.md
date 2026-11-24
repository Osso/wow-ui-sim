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
