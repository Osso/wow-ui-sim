# Shared game clock

## Contract

- The game clock starts at 12:00. `A_Admin.SetGameTime(hour, minute)` supplies integer values in the ranges 0–23 and 0–59; invalid inputs raise an error. `A_Admin.SetGameTime(nil)` clears the clock.
- `GetGameTime()` returns the supplied hour and minute, or `-1, -1` when unknown. The unknown result is a best guess, not verified native behavior.
- `GetLocalGameTime()` returns the same pair, and is registered only for Retail, PTR, and WowForever. Its unknown result is `-1, -1`.

## Evidence and test scope

- The pinned `Blizzard_APIDocumentationGenerated/SystemTimeDocumentation.lua` declares `GetLocalGameTime` at line 20 in each of the Retail, PTR, and WowForever profile caches. Mists, Era, and Anniversary omit it; Wrath has no such documentation file in its pinned cache.
- Native evidence S20 sections 1 and 5 establishes two numeric return values and `-1, -1` when no usable source is available. The generated documentation names the results hour and minute.
- No Retail Blizzard Lua caller was found. The integration test is a preservation/API-contract test, not a RED through an original consumer.

## Known gaps

- There is one supplied clock; realm and local game time are not distinguished, and the clock does not advance automatically.
- Classic-profile absence and WowForever availability are covered by profile-aware test logic but were not executed in this lane.
