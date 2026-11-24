# Housing scenario inputs

Retail scenarios can supply the player-owned house list used by `C_Housing.GetPlayerOwnedHouses`:

```lua
A_Admin.SetPlayerOwnedHouses({
    { plotID = 101, houseName = "North House", houseGUID = "House-101" },
    { plotID = 202, houseName = "South House", houseGUID = "House-202" },
})
A_Admin.SetPlayerOwnedHouses({}) -- clear the supplied list
```

Each array entry uses the documented `HouseInfo` fields. `plotID` is required; all other fields are optional. The setter replaces the list and does not fire an event. Each `GetPlayerOwnedHouses()` call queues its own `PLAYER_HOUSE_LIST_UPDATED(houseInfos)` event for a later timer tick. Requests return no values. Delivery timing and service behavior are **INFERRED (R88fj; native evidence pending)**; the event payload and `HouseInfo` field shape follow the pinned API documentation.

This is supplied scenario state, not an ownership service or a claim about live character data. See the [housing service contract](../specs/c-factory-m5-services.md).
