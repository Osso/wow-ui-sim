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

This is supplied scenario state, not an ownership service or a claim about live character data.

## House Finder scenarios

Retail tests can supply the House Finder neighborhood list and each neighborhood's map, texture, faction-match result, and plot records:

```lua
A_Admin.SetHouseFinderNeighborhoods({
    {
        neighborhoodGUID = "Neighborhood-A",
        neighborhoodName = "Amber Vale",
        neighborhoodType = Enum.NeighborhoodType.Public,
        neighborhoodOwnerType = Enum.NeighborhoodOwnerType.None,
        suggestionReason = Enum.HouseFinderSuggestionReason.None,
        uiMapID = 41,
        textureSuffix = "amber",
        factionMatches = true,
        plots = {
            {
                mapPosition = { x = 0.2, y = 0.3 },
                ownerType = Enum.HousingPlotOwnerType.None,
                plotID = 17,
                plotCost = 1250,
            },
        },
    },
})
A_Admin.SetHouseFinderNeighborhoods({}) -- clear the supplied list
```

Each list entry supplies `neighborhoodGUID`, `neighborhoodName`, `neighborhoodType`, `neighborhoodOwnerType`, and `suggestionReason`. `uiMapID`, `textureSuffix`, `factionMatches`, and `plots` are optional. Each plot supplies `mapPosition` and `ownerType`; `plotID`, `plotCost`, and `ownerName` are optional. Setters replace all scenario records. `HouseFinderRequestNeighborhoods` and `RequestHouseFinderNeighborhoodData` queue their matching consumer events for a later timer tick. No response is sent for an unknown neighborhood. `StartTutorial` follows the consumer-facing flow as **INFERRED (R88g1; native evidence pending)**. The supplied values describe a test scenario, not live neighborhood or faction data.

See the [housing service contract](../specs/c-factory-m5-services.md).
