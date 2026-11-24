# Communities

These inputs control community UI scenarios on Retail and WoW Forever.

### `A_Admin.SetAccountMutedClub(clubId, muted)`

Set whether the account cannot use chat in one club. `clubId` is an opaque club ID; `muted` is a boolean.

### `A_Admin.SetClubTypeAllowed(clubType, allowed)`

Set whether a club type is allowed in community flows. `clubType` is an `Enum.ClubType` value; `allowed` is a boolean.

The default allowed types are BattleNet and Character. Retail also allows Guild. Other is not allowed. This default policy is INFERRED R88fq (native evidence pending).

```lua
A_Admin.SetClubTypeAllowed(Enum.ClubType.Character, false)
assert(not C_Club.ShouldAllowClubType(Enum.ClubType.Character))
```
