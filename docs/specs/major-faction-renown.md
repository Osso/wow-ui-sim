# Major faction renown level and cap

`C_MajorFactions.GetCurrentRenownLevel(factionID)` and `C_MajorFactions.HasMaximumRenown(factionID)` read the supplied faction record in `state.major_factions` (`src/c_api/c_major_factions.rs`).

## Behavior
- [x] `GetCurrentRenownLevel` returns the record's `renown_level`.
- [x] `HasMaximumRenown` returns true when `renown_level >= max_level`.
- [x] An unknown or non-number faction ID returns 0 and false. There is no -1 path.
- [x] Values are mutable supplied state. A change after startup reaches the next query and the Blizzard consumer `RenownRewardUtil.AddMajorFactionLandingPageSummaryToTooltip` (next-level rewards below the cap, no progress block at the cap).

Tests: `tests/major_faction_renown_level.rs` (prefork full UI, Retail and PTR).

## Native evidence
Decomp 12.1.7 §37–§38, overlaps 0, VERIFIED: every failure path returns 0, and the cap check compares the current level against the threshold in the same units, with the threshold defaulting to 0 when its record is missing.

## Known gaps
- The native store keeps raw values and divides by 100 behind a per-record flag. The simulator stores levels directly, so the unit conversion is not modelled.
- The native record that supplies the cap threshold is not identified. The simulator uses the faction's `max_level` (published as `maxLevel` by `GetMajorFactionData`), not a separate threshold record.
- The native eligibility check is not modelled.
- The source of the native collection's values is UNKNOWN (§42).
