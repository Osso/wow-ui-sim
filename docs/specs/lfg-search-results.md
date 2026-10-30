# LFG search results

## Contract

- Search populates a player result set from the current premade listing catalog and schedules the existing deferred results event.
- Result queries and application submission use that set. Clearing results empties it and cancels pending search deliveries without deleting the catalog.
- A later Search repopulates the set; catalog changes do not create results until Search.

## Best-guess basis

**Best-guess, native UNKNOWN.** The pinned GroupFinder consumer calls ClearSearchResults and Search at Retail `Mainline/LFGList.lua:2778,2817`, PTR and Forever `Mainline/LFGList.lua:2765,2804`, and Mists, Era and Anniversary `Classic/LFGList.lua:2090,2125`. Era and Anniversary's Vanilla TOCs do not select the Classic LFGList file; publication on those profiles is explicitly requested despite that limitation. No generated declaration for ClearSearchResults was identified. The separation of catalog and results, and cancellation timing, are inferred; native behavior is unverified.
