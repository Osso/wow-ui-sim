# PvP subzone point of interest

## What it must do

- [ ] Retail `C_PvP.IsSubZonePVPPOI()` returns one boolean by looking up the supplied current area in the supplied AreaTable flag records and testing bit value 32768.
- [ ] Missing current area or missing area record returns false. Other flag bits do not affect this query.

## Best-guess basis

Retail `Blizzard_APIDocumentationGenerated/PvpInfoDocumentation.lua:1086` declares the boolean query. WM23-misc build 70688 identifies the current-area lookup and flag value; false/true publication is conditionally emulated, not observed live. The host supplies current-area identity and records; empty inputs are simulator scenario defaults, not claimed native area data.

## Implementation inventory

- `src/c_api/c_pvp/area_poi.rs`: explicit current-area and AreaTable flag inputs and query.
- `src/c_api/c_pvp.rs`: Retail registration.

## Tests asserting this spec

- `tests/pvp_subzone_poi.rs`: API-level distinct flagged, unflagged and absent inputs; no selected Retail FrameXML consumer was found. RED/GREEN are pending.

## Known gaps

- [ ] AreaTable loading, live area transitions, real-client publication and other profiles are unverified; no area flag catalog is inferred.
