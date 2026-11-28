# Premade Groups

> Not yet implemented. These functions are planned for a future admin API addition.

| Function | Description |
|----------|-------------|
| `A_Admin.AddPremadeListing(...)` | Add a listing to the premade group finder |
| `A_Admin.ClearPremadeListings()` | Remove all premade group listings |
| `A_Admin.UpdatePremadeListing(...)` | Update an existing premade group listing |

## Forever Vanilla-style Group Finder inputs

- `A_Admin.SetPremadeListingMembers(searchResultID, members)` replaces the `LfgSearchResultPlayerInfo` rows of one listing, in member-index order. Each row needs `className`, `classFilename`, `assignedRole`, `isLeader` and `lfgRoles = { tank, healer, dps }`; `name`, `level`, `areaName`, `specName` and `isLeaver` are optional. An unknown listing or a wrongly typed field raises an error.
- `A_Admin.SetSavedLFGListRoles({ tank, healer, dps })` sets the solo roles that `C_LFGListRoles.GetSavedRoles` returns (roles saved by an earlier session). Forever only.
