# Mythic+ map request reentrancy

Synchronous `C_MythicPlus.RequestMapInfo` response dispatch can re-enter addon event handlers while the request caller is still running. The simulator now records eligible requests and queues response events for a later timer boundary; **INFERRED R88ev (native evidence pending)** defines the cooldown and response mapping.

## Content

### Root cause

- **VERIFIED (simulator source):** the old `RequestMapInfo` path dispatched `CHALLENGE_MODE_MAPS_UPDATE` inside the request call.
- **VERIFIED (same-root Retail run at master `e6407a379e2f22703426b56ed285862751788b3c`):** the explicit addon-root run discovered 78 addons, loaded 49, and recorded 10 unique errors with 159,423 total occurrences, including repeated `RequestMapInfo` recursion, stack overflow, and instruction-budget errors.
- **VERIFIED (same-root Retail run at branch `ce192415b406914145c394ac5ef40ecdf8a9c41c`):** the same command discovered 78 addons, loaded 49, and recorded 7 unique errors with 7 total occurrences; it recorded no stack overflow, instruction-budget, or watchdog error.

### Current inferred request behavior

- **INFERRED R88ev (native evidence pending):** `RequestMapInfo` requires an existing player, admits the first request when no timestamp exists, saves the admitted request time before checking connection, and suppresses later requests until the simulator clock advances by strictly more than 60 seconds.
- **INFERRED R88ev (native evidence pending):** admitted connected map-info requests queue `CHALLENGE_MODE_MAPS_UPDATE` asynchronously; `RequestCurrentAffixes` queues `MYTHIC_PLUS_CURRENT_AFFIX_UPDATE` without a cooldown, and `RequestRewards` retains the simulator's deferred `WEEKLY_REWARDS_UPDATE` compatibility event.
- **VERIFIED (Retail and Forever targeted tests):** two map-info calls in one handler coalesce through the cooldown, response delivery occurs after the handler returns, explicitly supplied map records are visible to the consumer, exactly 60 seconds is suppressed, and a request after 60.01 seconds is admitted.

### Comparison limits

- **VERIFIED (same-root Retail outputs):** AllTheThings loaded on both revisions (master 3.4 seconds; branch 5.4 seconds); the master output has four AllTheThings instruction-budget errors, while the branch output has none.
- **VERIFIED (same-root branch output):** one `Blizzard_ChallengesUI` error stack ends at the simulator's deferred `Mythic+ map info response event` and reports a nil map-row access.
- **INFERRED:** the nil map-row error is consistent with the run's empty challenge-map input; the evidence does not prove whether native Retail would emit the response event with an empty map catalogue.
- **VERIFIED (same-root branch output):** the other five errors are the two approved TokenUI records, two `Clicked` errors, one secret-texture error, and one Baganator missing-API error.
- **VERIFIED (same binary and addon root; SavedVariables condition changed):** the no-SavedVariables run reported two Clicked errors; loading the real ClickedDB in an isolated session reported zero. **INFERRED:** the first-install Clicked callback ordering explains the difference; see [Clicked first-install saved variables](clicked-first-install-savedvariables.md). This is separate from the Mythic+ request change.

## Sources

- [Mythic+ request response contract](../../specs/mythic-plus-map-info-request.md)
- `src/c_api/c_mythic_plus.rs` — request admission and deferred response events
- `src/lua_api/state_types/mythic_plus_scenario.rs` — map-info request timestamp
- `src/lua_api/globals/group_queries_relationships.rs` — player connection predicate
- `tests/c_mythic_plus_probes.rs` — response, cooldown, and consumer tests

## See Also

- [[script-execution-watchdog]] — replacement for instruction-count quotas
- [[event-system]] — queued and dispatched frame events
