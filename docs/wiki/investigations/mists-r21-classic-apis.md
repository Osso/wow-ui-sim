# Mists R21 Classic API behavior

This page records behavior observed in the Mists 5.5.4.70631 native binary for the Classic reputation list, engraving-mode flag, and backpack-currency list. Era and Anniversary behavior is UNKNOWN and is not changed by this lane.

## Content

### Evidence scope and selected consumers

- **VERIFIED:** Native behavior in this page is from Mists 5.5.4.70631. Era 1.15.x and Anniversary native behavior is UNKNOWN.
- **VERIFIED:** The selected Mists `Blizzard_UIPanels_Game_Classic.toc` loads `Classic/ReputationFrame.lua` and `Classic/ContainerFrame_Shared.lua`; the selected `Blizzard_TokenUI_Mists.toc` loads the Mists TokenUI consumer. The integration tests assert the actual loaded Lua sources.
- **VERIFIED (pre-fix simulator test):** `ReputationFrame_Update` originally read a hidden child at visible index 2 after collapsing the first header because the simulator had not rebuilt the backing array.

### Classic reputation list

- **VERIFIED:** `GetNumFactions()` returns the visible-prefix length. `GetFactionInfo(index)` reads the same full backing array, so hidden rows remain reachable at positions above the visible count.
- **INFERRED for ordinary populated lists:** after `CollapseFactionHeader(1)`, index 2 is the next visible header. Native sorting is verified; the visible-count path can exclude some empty, ineligible headers in states not covered by this model.
- **VERIFIED:** The native row comparator puts visible rows before hidden rows. Visible groups follow the separately sorted header order. Hidden groups compare equal, and the sort is unstable; hidden-tail order is undefined and tests must not assert it.
- **VERIFIED:** Headers sort with one special header first, then a numeric field descending, then by name; record-backed headers precede synthetic headers when the prior keys tie. The numeric field's identity as Expansion is **INFERRED**.
- **VERIFIED:** Within a group, the header comes first and rows sort by faction name. Rows with missing records compare equal.
- **VERIFIED:** `ExpandFactionHeader` and `CollapseFactionHeader` use the same one-based backing-array positions as `GetFactionInfo`. An invalid or non-header index takes the expand-all or collapse-all path.
- **VERIFIED:** `GetFactionInfo` returns 16 values for a normal row, 14 for a header, and 13 for the missing-row fallback `nil, nil, 1, 0, 0, 0, nil, nil, nil, nil, nil, nil, nil`.
- **VERIFIED:** Selected faction identity is stored by faction ID and resolved against the visible rows; a selected faction hidden by collapse has no visible index.

The simulator rewrites the shared `__wow_reputation_state.factions` table into one visible-prefix array. Each child row keeps an internal pointer to its owning header so later sorts preserve group membership. `headerSortValue` and `isSpecialHeader` are simulator fixture fields, not claimed native field names. Treating nonzero scenario `factionID` as record-backed is an inferred simulator mapping.

### `C_Engraving` mode

- **VERIFIED:** Mists `GetEngravingModeEnabled()` takes no arguments, returns one boolean, and starts false.
- **VERIFIED:** `SetEngravingModeEnabled(enabled)` stores the boolean and synchronously fires `ENGRAVING_MODE_CHANGED(enabled)` on every call, including unchanged values.
- **VERIFIED:** WowForever 1.60.1.70707 contains no Engraving or RuneCast strings; Era behavior remains UNKNOWN.
- **VERIFIED:** The original bag consumer updates rune icons from the event. The focused mode and original-consumer tests passed before the final reputation-only implementation change.

### Backpack currency

- **VERIFIED:** Mists `GetBackpackCurrencyInfo(index)` returns `name, count, icon, currencyID`, or four nils for an empty or out-of-range one-based watched-list slot.
- **VERIFIED:** `SetCurrencyBackpack` takes a one-based currency-tab list index and a numeric zero/nonzero flag. The native getter and setter have no three-slot cap.
- **VERIFIED:** The selected Mists TokenUI displays at most three watched slots; the API can return a fourth.
- **VERIFIED:** The focused currency API test and corrected original TokenUI test passed before the final reputation-only implementation change.

### Verification status

The post-change reputation tests, profile matrix, and final handoff are pending. The currency and engraving results above are targeted-test evidence from the current lane, not final acceptance of the complete R21 goal.

## Sources

- `/home/osso/.local/state/wow-ui-sim/handoff/pr46-49-decomp-answers.md` — R21 native findings for Mists 5.5.4.70631 and the R21-1b reputation ordering follow-up.
- [Reputation list contract](../../specs/mists-legacy-reputation-list.md) and [fixture](../../specs/mists-reputation-fixture.md).
- [Engraving mode contract](../../specs/mists-engraving-inputs.md) and [backpack-currency contract](../../specs/mists-backpack-currency.md).
- [Reputation tests](../../../tests/mists_r21_reputation.rs), [currency tests](../../../tests/mists_r21_currency.rs), and [engraving tests](../../../tests/mists_engraving.rs).
- `src/c_api/mists_reputation_list.rs` and `src/lua_api/workarounds/temporary/reputation_state.rs` — simulator row-list projection.

## See Also

- [[mists-engraving-inputs]] — original engraving consumers and supplied-input boundaries.
- [[consumer-selection-and-red-proof]] — runtime-selected consumer and RED proof requirements.
