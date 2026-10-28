# Mists supplied scenario finder eligibility

Mists `C_LFGInfo.CanPlayerUseScenarioFinder()` consumes an explicit environment-local boolean and failure-reason string. The pinned 5.5.4.70268 declaration is `Blizzard_APIDocumentationGenerated/LFGInfoDocumentation.lua:51`; the original consumer is `Blizzard_GroupFinder/Classic/PVEFrame.lua:270`. See the [input and consumer model](../wiki/systems/mists-scenario-finder.md).

## What it must do

- [x] Publish the query only on Mists. Return exactly the supplied non-nil boolean and non-nil string, including the supplied string when allowed.
- [x] Keep eligibility independent of player level, expansion and scenario catalog. When no tuple is supplied, use the best-guess scenario default `true, ""`; native behavior is UNKNOWN.
- [x] An absent input returns the scenario default, not a claim about native eligibility.
- [x] The original XML-bound PVEFrame callback enables the visible scenario button when allowed; denial disables it and supplies its tooltip. Allowing again clears the tooltip. The separate original empty-list branch still hides the button.
- [x] Two environments keep independent tuples and presentation state. Clearing the input restores the scenario default; resupplying recovers.
- [x] The LFR original-consumer regression retains every visibility and gap-anchor assertion and completes past270 instead of accepting its old missing-function error.

## How it works

- [Supplied input and original consumer](../wiki/systems/mists-scenario-finder.md)
- [Existing LFR input](mists-lfr-availability.md)

## Implementation inventory

- `src/lua_api/state/sim_state.rs`, `src/lua_api/state.rs` — optional environment-local tuple and unset initialization.
- `src/c_api/scenario_finder_availability.rs` — supplied bool/string query and scenario default.
- `src/c_api/c_lfg_info.rs`, `src/c_api/mod.rs` — Mists-only namespace registration and module wiring.

## Tests asserting this spec

- `tests/mists_scenario_finder.rs` — allowed-first and denied-first original-consumer transitions, exact tuple shape, isolation and unset recovery.
- `tests/mists_lfr_availability.rs` — original LFR observations with complete callback execution past270.

## Known gaps (current cycle)

None within the supplied-input slice. Both original270 entry selectors and the advanced LFR regression have executed RED/GREEN. Native eligibility/default policy remains UNKNOWN.

## Out of scope

Scenario catalog production, party-minimum inference, native service policy, other-profile publication, registered-event delivery after complete OnLoad, vendor edits and unrelated startup errors.
