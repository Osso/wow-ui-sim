# LFG search results

## Contract

- Search snapshots result membership from the current premade listing catalog and schedules the existing deferred results event.
- Result queries and application submission use that membership. Fields for existing results reflect current listing data; changing a result fires `LFG_LIST_SEARCH_RESULT_UPDATED(resultID)`.
- A delisted result remains in the set with `isDelisted=true` and emits the update event. New listings do not enter the set until Search.
- Clearing results empties the set and cancels pending search deliveries without deleting the catalog.
- SetSearchToActivity and SetSearchToQuestID replace the pending search-text filter. Search applies it to listing activity IDs or supplied quest IDs. ClearSearchTextFields clears the pending filter; it does not itself clear results.

## Best-guess basis

**Best-guess, native UNKNOWN.** The pinned GroupFinder consumer calls ClearSearchResults and Search at Retail `Mainline/LFGList.lua:2778,2817`, PTR and Forever `Mainline/LFGList.lua:2765,2804`, and Mists, Era and Anniversary `Classic/LFGList.lua:2090,2125`. Retail `LFGList.lua:215,2395–2410` registers and handles individual result updates; `:2845–2862` deselects a delisted listing. Retail `LFGListInfoDocumentation.lua:965–983` declares `isDelisted` on `LfgSearchResultData`. Era and Anniversary's Vanilla TOCs do not select the Classic LFGList file; publication on those profiles is explicitly requested despite that limitation. No generated declaration for ClearSearchResults was identified. Retail/PTR/Forever `LFGListInfoDocumentation.lua:58,603–622` and Mists/Era/Anniversary `:42,462–480` declare ClearSearchTextFields, SetSearchToActivity and SetSearchToQuestID. Retail `LFGList.lua:2778–2779,3032,3087` uses these controls. Result membership, update delivery, delisting retention, search-filter matching and cancellation timing are inferred; native behavior is unverified.
