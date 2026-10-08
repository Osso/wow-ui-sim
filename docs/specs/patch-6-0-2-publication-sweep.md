# Patch 6.0.2 publication audit

## Contract

Pin Warcraft Wiki pageid 556401 and its transcluded diff independently. Retain every explicit API occurrence, all prose statements and enum values. API sweeps prove publication/absence only, applying later registers oldest first. They do not prove historical signature, return, numeric, security or domain parity.

`C_Scenario.GetBonusSteps()` returns a detached ordered array of bonus step IDs from the active scenario. Completed bonus steps remain enumerated. `C_Scenario.GetBonusStepRewardQuestID(stepID)` returns the optional reward of the matching bonus step; unknown/nonbonus steps and inactive scenarios return nil. Invalid nonnumeric IDs error. Both queries use existing `ScenarioState.steps`; no new fixture defaults or vendor changes.

Retire only `C_Scenario.GetBonusCriteriaInfo`, `C_Scenario.GetBonusStepInfo` and `C_Vignettes.GetVignetteInstanceID`: complete qualified/bare cached-consumer and source/test grep scans must be empty, and pinned master/queued registers must not re-add them. Both raw and ordinary lookup remain nil. Preserve every current consumer and caller.

## Proof boundaries

Record exact remaining gaps and reasons, including unmodeled garrison missions/recruitment, applicant workflows, historical stat/talent APIs, removed-but-consumed globals and historical enum contracts. Pending 6.1.0/6.2.0/6.2.2 integration comments precede later registers.

Targeted gates: all publication sweeps, bare/cached own behavior and existing scenario regressions, Python fixtures, reproduction of all saved sources, Mists tests check, format, addons-enabled startup comparison, and clean/later-audit validator portability. No full integration suite or CASC texture acceptance on this host.

See [audit](../wiki/investigations/patch-6-0-2-api-audit.md) and [evidence](../../data/patch-api/evidence/6.0.2-session-2026-10-08/).
