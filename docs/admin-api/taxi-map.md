# Taxi map

`A_Admin.SetTaxiMapNodes(nodes)` supplies the flight-master nodes that `C_TaxiMap` and the legacy taxi slot globals read. Each node table requires `mapID`, `nodeID`, `name`, `atlasName`, `textureKit`, `faction` (0–2), `x`, `y`, `state` (`Enum.FlightPathState`, 0–2), `slotIndex`, `isUndiscovered`, `useSpecialIcon` and `isMapLayerTransition`. `specialIconCostString` is optional.

Two optional fields feed the Retail legacy globals:

- `cost`: a nonnegative integer copper cost read by `TaxiNodeCost`. A node without `cost` reports 0.
- `routes`: an array of hops `{ sourceSlot = n, destinationSlot = m }` from the current node to this node. `GetNumRoutes`, `TaxiGetNodeSlot`, `TaxiIsDirectFlight` and `TaxiGetSrcX`/`Y`, `TaxiGetDestX`/`Y` read it. A node without `routes` has no hops.

```lua
A_Admin.SetTaxiMapNodes({
    { mapID = 14, nodeID = 901, name = "Alpha", atlasName = "TaxiNode", textureKit = "", faction = 0,
      x = 0.2, y = 0.3, state = Enum.FlightPathState.Current, slotIndex = 1, isUndiscovered = false,
      useSpecialIcon = false, isMapLayerTransition = false },
    { mapID = 14, nodeID = 902, name = "Beta", atlasName = "TaxiNode", textureKit = "", faction = 0,
      x = 0.4, y = 0.5, state = Enum.FlightPathState.Reachable, slotIndex = 2, isUndiscovered = false,
      useSpecialIcon = false, isMapLayerTransition = false, cost = 15,
      routes = { { sourceSlot = 1, destinationSlot = 2 } } },
})
```

Retail also has `A_Admin.SetTaxiMapID(uiMapID)` and `A_Admin.OpenTaxiMap(uiMapSystem)`, which fire `PLAYER_INTERACTION_MANAGER_FRAME_SHOW` and `TAXIMAP_OPENED`.

## INFERRED R88g9 (native evidence pending)

- `NumTaxiNodes` returns the highest supplied `slotIndex`.
- `TaxiNodeGetType` returns `"CURRENT"`, `"REACHABLE"` or `"UNREACHABLE"` for states 0, 1 and 2, and `"NONE"` for a slot with no node. The simulator does not model `"DISTANT"`.
- `TaxiNodeName` and `TaxiNodePosition` return no values for an empty slot.
- `TaxiIsDirectFlight` is true when the node has exactly one hop.
- Hop coordinates are the supplied `x` and `y` of the hop's source or destination node.
