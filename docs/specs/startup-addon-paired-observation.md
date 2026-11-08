# Startup addon paired observation

Test-only observation in [`tests/startup_perf.rs`](../../tests/startup_perf.rs) pairs each successful canonical addon load result with that same call's outer wall time. This is an observation contract, not a startup optimization. Independent2794 accepts bounded current-run proof at source `490bc1080`; [packet](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/test-perf/evidence/startup-canonical-paired-2026-10-12/README.md) owns exact results and provenance. Public retention/link/privacy independently passed at `da4bbc442` within retained-text scope.

## What it must do

- [x] Preserve canonical `new_game_env`, discovery, ordered `load_addon` calls, post-load workarounds, startup events, readiness query (`UIParent ~= nil and PlayerFrame ~= nil and IsLoggedIn()`), original assertions, and strict elapsed `< 20s` threshold. Do not substitute a separate startup path.
- [x] Start an outer wall clock immediately before each `load_addon` call and stop it immediately on return, before result conversion or row recording. Pair the successful result with that duration; preserve existing load-error behavior.
- [x] Retain typed rows until the full startup clock stops after startup events. Recording and allocation overhead remain inside canonical elapsed time; subtract nothing. JSON encoding and stderr emission occur only after that clock stops.
- [x] Emit one JSON row per successful addon load and one full-startup summary to stderr with prefix `[startup-observation]`, before either original readiness or budget assertion, so assertion failures leave observation evidence. Include full-startup total and existing addon-load, post-load-workaround, and startup-event phase durations as integer nanoseconds, plus the original readiness boolean.
- [x] Each addon row includes discovered identity, returned addon name, discovery-order ordinal, same-call outer wall nanoseconds, Lua/XML file counts, and eight base timing fields as integer nanoseconds: `io`, `xml_parse`, `xml_process`, `lua_compile`, `lua_call`, `lua_exec`, `savedvars`, `total`. Include all six cache counters (`cache_hits`, `cache_misses`, `cache_lookup_misses`, `cache_replay_failures`, `cache_store_successes`, `cache_store_failures`) and counts of `warnings`, `nil_symbol_observations`, and `missing_requirements`; emit no diagnostic text.

## Implementation inventory

- `tests/startup_perf.rs` — canonical startup timing test and test-only observation/serialization coverage.
- `src/loader/mod.rs` — existing `LoadResult` and `LoadTiming` inputs; unchanged by this contract.

## Tests asserting this spec

- `tests/startup_perf.rs::full_game_startup_stays_under_budget` — one canonical body PASS; parent/child summaries are not separate executions. Ordered paired rows and original readiness/budget assertions have bounded current-run proof.
- `tests/startup_perf/observation.rs::tests::addon_observation_roundtrip_preserves_values_and_only_diagnostic_counts` — concrete distinct durations, counters and identities; asserts decoded JSON values, integer nanosecond units and diagnostic counts without text. One focused serializer execution PASS; separate from the canonical body.

## Additional bounded proof — 2026-10-18

The unchanged canonical selector `startup_perf::full_game_startup_stays_under_budget` passed at source `f8afe3b3f9e23082aa7c44acef9feafb05ff4f59`, based on `29fac7076965dd53973168c47d972e4090f27b98`, using the same Retail integration build as the timer/unit proof batch (`--no-default-features --features sound,gui,casc,client-retail`). Raw stderr contains **211** discovery-ordered addon rows and **one** summary: readiness true, `startup_elapsed_ns=14848171400 < budget_ns=20000000000`. Parsed rows retain both identities, file counts, eight timings, six cache counters and three diagnostic counts with nonnegative integer values. Discovered identities and returned addon titles are distinct fields, not an equality requirement.

Private-store-relative receipt `handoff/a1-timer-unit-tests/build-03-B7.json` records exact argv, source, emitted features, executable SHA-256, ordered names and parsed summary; adjacent stdout/stderr retain the original assertions and records. Parent/child PASS summaries are **one logical canonical execution**, not two. Clock boundaries remain source-audited. This passing run does not exercise original assertion failure or load-error behavior, and makes no native, historical timing-equivalence or all-profile claim.

## Known gaps (current cycle)

- [x] Implement observation and focused concrete serialization coverage; saved compilation and formatting PASS. Production check reuse covers unchanged production/library scope only, not new tests.
- [x] Obtain bounded current-run evidence for canonical ordering and same-call pairing; source audit establishes clock/emission boundaries.
- [ ] Execute emission-before-failing-assertion behavior: source order is established, but this passing run did not exercise assertion failure.
- [x] Public packet retention/link/privacy independently verified at `da4bbc442`; native/causal/historical acceptance remains separate.

Initial compile E0583 at `59f3ff136` remains historical failure, separate from current PASS. Diagnostic collection counts do not establish compatibility-clean startup. Captured allowlist values are null, including `WOW_SIM_LOG_HANDLER_TIMINGS` and `XDG_CACHE_HOME`; actual bytecode enable/disable controls, `ASSET_RESOLVER_CACHE_DIR`, and `WOW_SIM_VERBOSE` child values are unknown. Captured `WOW_SIM_BYTECODE_CACHE_DIR` is not an actual reader. No full cache-policy or cache-contents attestation.

## Out of scope

- Exclusive buckets, pure-body timings, causal attribution, or summing overlapping buckets as elapsed time: loader timing buckets overlap.
- Performance repair, runtime instrumentation, native-client parity, and historical benchmark equivalence: paired observation establishes none of these.
