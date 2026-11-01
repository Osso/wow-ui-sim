# Report player eligibility

Retail `Blizzard_APIDocumentationGenerated/ReportSystemDocumentation.lua:11-22` declares `C_ReportSystem.CanReportPlayer(playerLocation)` returning a Boolean. `Blizzard_UnitPopupShared/UnitPopupSharedButtonMixins.lua:1250-1253` calls it after validating the player location.

## Best-guess basis (best-guess, native UNKNOWN)

Report eligibility is an explicit scenario input, `SimState.report_player_eligible`, initially false. The API returns that value for a table argument and rejects non-tables. It does not infer eligibility from a player, account, or report type. A host can supply true or false for the scenario.

## Test

`report_system::report_button_respects_supplied_eligibility` calls the original unit-popup consumer with a valid player location and checks both supplied values and invalid input.

## Known gaps

Native defaults and argument validation details are unknown. Per-player, account, report-type, evidence and restriction policy are not modeled; the supplied Boolean applies to the whole scenario.
