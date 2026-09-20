# Forever next raid target marker

## Contract

Forever exposes `GetNextAvailableRaidTargetMarkerIndex(startIndex, reverseSearch=false, wrapSearch=false, treatDeadNonFriendlyAsAvailable=false)` over existing GUID-keyed unit marker assignments. Other profiles retain their existing surface.

Search includes `startIndex`, traverses markers 1–8 in the requested direction, and crosses the boundary only when wrapping is enabled. Return the first free marker, or zero when exhausted; do not mutate assignments. When requested, a known dead nonfriendly unit's marker is reusable. Unknown assigned GUIDs remain occupied. Current target, focus and enemy-pool snapshots supply existing unit health/reaction; friendly assignments remain occupied.

## Evidence and limits

Pinned Forever `Blizzard_APIDocumentationGenerated/RaidMarkersDocumentation.lua` declares the arguments and defaults. `Blizzard_GamepadActionBars/TargetActionBars/Shared.lua` and `Blizzard_GamepadTargeting/TargetLogic.lua` pass the next candidate index and explicitly interpret zero as no marker available. This models those consumers; native conformance, secret-value enforcement, and undocumented out-of-range behavior are not claimed. The simulator rejects starting indices outside 1–8.

## Verification

Grouped integration tests in `tests/wowforever_raid_marker_constants.rs` cover occupied/free transitions, inclusive forward/reverse search, wrapping, exhaustion, dead hostile/neutral versus friendly/live units, and the actual Shared button consumer using real assignment/search APIs.
