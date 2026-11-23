# Script execution watchdog

The simulator applies a host-owned, per-invocation elapsed-time watchdog to tainted addon Lua files and frame callbacks. Rilua owns dispatch polling; the simulator selects the timeout and owner scope.

## Content

### VM scope and polling

**VERIFIED (simulator source and R88bn):** the host enters `with_instruction_owner(owner, ...)` for a tainted invocation, then `with_execution_watchdog(timeout, dispatch_interval, ...)` around the Lua call. The watchdog stores a monotonic deadline and a remaining-dispatch count. Rilua selects the watchdog execution loop only while a watchdog is active; it decrements the count on each VM dispatch and reads the clock when the count reaches its interval. The default interval is 10,001 dispatch steps. On deadline overrun, the VM raises `script ran too long`.

**VERIFIED (simulator source):** the owner comes from the host loader context for files or the Rust widget's addon identity for frame callbacks. Unowned secure Blizzard execution does not enter an owner scope and is not timed. The instruction-count budget API, cumulative owner counters, reset API, and `instruction budget exhausted` error are removed.

### Timeout selection

**INFERRED (R88bn):** the native timer unit is milliseconds; simulator policy uses 19,000 ms while loading and during startup callbacks through completion of the initial `PLAYER_ENTERING_WORLD` dispatch, then 200 ms. The native threshold-to-lifecycle mapping is UNKNOWN. `WowLuaEnv::set_script_watchdog_timeout_for_tests` supplies a deterministic override for tests.

### Diagnostics

**VERIFIED (simulator source):** with `WOW_SIM_LOG_HANDLER_TIMINGS` present, tainted file execution emits `[file-budget-success]` or `[file-budget-error]`; failed owner frame callbacks emit `[handler-budget-error]`. The duration filter does not suppress these records. They contain owner/frame/event or owner/file identity plus `timeout_ms` and `elapsed_ms`. An error record can represent a non-watchdog failure. The elapsed timer includes synchronous nested work and is not an exclusive cost. Metadata is escaped, but non-addon chunk paths can remain absolute.

The per-invocation watchdog is separate from the CVar-driven script bucket throttle. **INFERRED (R88bn; native evidence pending):** the restricted bucket is not charged while the existing `startup_script_watchdog_phase` is active; the first post-phase addon invocation starts with the full burst. The bucket contract and remaining assumptions are specified in [script bucket throttle limits](../../specs/script-bucket-throttle-limits.md).

## Sources

- [Startup file watchdog contract](../../specs/startup-file-instruction-diagnostics.md)
- [Rilua capability rows](../../specs/rilua-capability-rows.md)
- [Addon loading pipeline](../../addon-loading-pipeline.md#top-level-file-watchdog-diagnostics)
- VERIFIED (R88bn, Retail 12.1.5.70688 and Forever 1.60.1.70707): VM dispatch polling and the `script ran too long` error.

## See Also

- [[addon-loading]] — top-level addon file execution
- [[event-system]] — frame event and script-handler dispatch
- [[retail-allthethings-name-lookups]] — original long-running AllTheThings callback
