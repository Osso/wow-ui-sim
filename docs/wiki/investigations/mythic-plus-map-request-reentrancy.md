# Mythic+ map request reentrancy

Synchronous `C_MythicPlus.RequestMapInfo` response dispatch can re-enter addon event handlers while the request caller is still running. The simulator now records eligible requests and queues response events for a later timer boundary; **INFERRED R88ev (native evidence pending)** defines the cooldown and response mapping.

## Content

### Root cause

- **VERIFIED (simulator source):** the old `RequestMapInfo` path dispatched `CHALLENGE_MODE_MAPS_UPDATE` inside the request call.
- **VERIFIED (same-root Retail run at `15417213e7799bffae5ebd5cb6e396656300cbcc`):** the explicit addon-root run discovered 78 addons, loaded 49, and recorded 8 unique errors with 159,655 total occurrences, including repeated `RequestMapInfo` recursion, stack overflow, and instruction-budget errors.
- **VERIFIED (same-root Retail run at `71a13c2e0c4938b0ad774045247fd7970a591cf8`):** the same command discovered 78 addons, loaded 49, and recorded 5 unique errors with 5 total occurrences; it recorded no stack overflow or instruction-budget/watchdog error.

### Current inferred request behavior

- **INFERRED R88ev (native evidence pending):** `RequestMapInfo` requires an existing player, admits the first request when no timestamp exists, saves the admitted request time before checking connection, and suppresses later requests until the simulator clock advances by strictly more than 60 seconds.
- **INFERRED R88ev (native evidence pending):** admitted connected map-info requests queue `CHALLENGE_MODE_MAPS_UPDATE` asynchronously; `RequestCurrentAffixes` queues `MYTHIC_PLUS_CURRENT_AFFIX_UPDATE` without a cooldown, and `RequestRewards` retains the simulator's deferred `WEEKLY_REWARDS_UPDATE` compatibility event.
- **VERIFIED (Retail and Forever targeted tests):** two map-info calls in one handler coalesce through the cooldown, response delivery occurs after the handler returns, explicitly supplied map records are visible to the consumer, exactly 60 seconds is suppressed, and a request after 60.01 seconds is admitted.

### Comparison limits

- **VERIFIED (same-root branch output):** `AllTheThings` appears in the addon loading summary, and no AllTheThings error or instruction-budget/watchdog error appears in the branch JSON output.
- **VERIFIED (same-root branch output):** one `Blizzard_ChallengesUI` error stack ends at the simulator's deferred `Mythic+ map info response event` and reports a nil map-row access.
- **INFERRED:** the nil map-row error is consistent with the run's empty challenge-map input; the evidence does not prove whether native Retail would emit the response event with an empty map catalogue.
- **UNVERIFIED:** two branch `Clicked` errors report a nil `ClickCastHeader`; the comparison does not establish a causal link to the watchdog or Mythic+ changes.

## Sources

- [Mythic+ request response contract](../../specs/mythic-plus-map-info-request.md)
- `src/c_api/c_mythic_plus.rs` — request admission and deferred response events
- `src/lua_api/state_types/mythic_plus_scenario.rs` — map-info request timestamp
- `src/lua_api/globals/group_queries_relationships.rs` — player connection predicate
- `tests/c_mythic_plus_probes.rs` — response, cooldown, and consumer tests

## See Also

- [[script-execution-watchdog]] — replacement for instruction-count quotas
- [[event-system]] — queued and dispatched frame events
