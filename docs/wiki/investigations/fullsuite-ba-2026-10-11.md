# Frozen ba full-suite result — 2026-10-11

[Saved packet](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/query-final-suite-result-2026-10-11/README.md) independently authenticates the frozen `ba413e8e177f04dc6fc7491834606ce36c750954` execution: **15,038 PASS / 3 FAIL / 19 skipped / 15,041 executed**. Unit `full-suite-1791749250.service` completed at20:48:40UTC; runner0 is not test success. Broad acceptance FAIL, full handoff OPEN.

## Coverage

| Scope | Result | Proof level |
|---|---|---|
| Integration | 10,713 PASS /2 FAIL;19 skips | Unique named terminal rows,10715 executed; phase exit100 |
| Prefork | 2321 main +4 auxiliary PASS | Four result blocks, each named case counted once; phase exit0 |
| Lib | 2000 PASS /1 FAIL | Unique named terminal rows,2001 executed; phase exit100 |
| Original Garrison load | One prefork PASS | Already inside totals; historical1dcc failure preserved |
| New achievement and Garrison/bootstrap | Eight +four integration PASS | Already inside totals; not native/catalog/profile-wide proof |

## Observed failures and root boundaries

1. `startup_perf::full_game_startup_stays_under_budget`:96.24s>20s. Addon-loading interval89.75s dominates the measured fixture; owner/file/primitive cost is unmeasured. Timer includes environment creation, discovery/loading, post-load workarounds and startup events. Outer nextest625.669s includes wrapper/gate lifecycle, not startup duration.
2. `talent_panel_onupdate_perf::talent_panel_open_onupdate_tick_latency_stays_under_budget`: baseline1.18ms, talent25.65ms, max45.69ms; rounded delta24.47ms>20ms. Average35/max75 ceilings pass. Tick timer includes timer processing, layout and OnUpdate dispatch, not callback bodies alone. Exact child35.37s differs from outer522.698s.
3. `asset_resolver_config::tests::gui_resolution_cache_preparation`: cold child fails before cache assertions reading encoding key `53dcc8beeb4c9e75b69c5f2a8a67c7e4` from `/syncthing/World of Warcraft` through archive lookup/fallback. This is not proof that the effective historical index lacked an entry, nor a cache-behavior verdict. Selected product/build/index mismatch or incomplete inputs remain unproved; no source fix, cache deletion, fallback or native-input acquisition follows from this failure.

Both performance selectors and the asset selector passed in saved1dcc. Compared test/harness and asset-wrapper/locked dependency sources are unchanged; this is an observed outcome change, not controlled causal attribution. Admission/resource ceilings do not explain per-test runtime. No per-test resource or vendor/cache input seal exists. Exclusive workload locking is conditional: inherited `WOW_SIM_TIMEOUT_FIXTURE_CAPACITY=reserved` bypasses it; its value was not captured.

## Next diagnostics

[Reduced ProductChoice result](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/productchoice-declaration-probe-2026-10-11/README.md) is independently authenticated: compile0/run0; declaration returns self-anchor Err, separate sibling control resolves its points. No full-template/startup/native repair follows.

[Two isolated exact controls](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/ba-isolated-perf-controls-2026-10-11/README.md) independently pass once each on one sealed existing ba executable: startup3.68s<20s; talent average501.18µs/max595.91µs/rounded delta59.66µs within35/75/20ms. Saved units exit0 under90s whole-cgroup limits; no remaining processes observed. No timeout was exercised. Environment inherited without overrides, but historical input/environment equality is unproved. An isolated PASS neither repairs the loaded-suite failure nor proves resource causality. Preserve thresholds, assertions and vendor behavior; measure narrowed simulator-side costs before proposing fixes. No broad-suite replay for docs or these diagnostics.

[Separate talent-stage observations](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/test-perf/evidence/talent-stage-diagnostic-2026-10-11/README.md) now have independent coherence proof:54 observations, not acceptance cases; isolated total delta49.52µs/whole public OnUpdate49.30µs does not explain loaded failure. [Current CASC candidate prefix](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/casc-encoding-header-2026-10-11/README.md): one34-byte read finds BLTE at+30 and matching9-byte key prefix in data174; full-key/size layout differences are not checked by the asset-resolver. Independent2463 verifies cached metadata/primary-source limits/initial retention only; no effective historical lookup/payload/decode/root cause proved. Both diagnostics leave this frozen verdict unchanged.

### Talent handler-log observation — 2026-10-12

[Packet](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/test-perf/evidence/talent-handler-log-2026-10-12/README.md) owns detailed counts/timings and the final capture report; independent retention/link/privacy gate at `e95d9f1901ef270058f18dac788814863f070423` reports bounded PASS:57 copies/2,845,419 bytes,19 links/4 anchors,zero privacy findings; no binaries/images/replays. Retained-text inspection is not universal secret detection; private reports remain outside the packet. Independent2779 reports **bounded capture coherence PASS** by independent raw reconstruction for epoch `diagnostic-20261012T134247318942Z`, source `1d2b0ea30`. Original exact selector body PASS is credited once, not twice from parent/child summaries. New seal first authenticated once; actual compile/fmt PASS, default check REUSED only for unchanged production/library scope. Existing manifest deprecations remain; no warning-free claim.

Actual54 windows preserve20 baseline/10 settle/24 steady order. Counts are rounded inclusive attempted-dispatch records, including repeats and outside-window records; **zero OnPostUpdate records observed**, not proof none registered. Packet separates phase aggregates from outer tick averages and the first-settle ClassTalents spike outside STEADY. Neither counts nor sums are unique/top-level callbacks or pure Lua-body cost: helper/error/nested intervals can double-count, and logging perturbs measurement.

Existing logger0 override precedes parent process creation; source-backed environment inheritance reaches child before OnceLock initialization. No independent child environment/PID or cache-contents attestation; cache metadata is narrower proof. Parser `all_attempts_verified=false` remains conservative. Original setup/order and35/75/+20ms assertions unchanged; no vendor/production metrics or new runtime configuration. [Spec](../../specs/talent-handler-log-observation.md) records backed bounded requirements and saved development RED46/GREEN12 limits, excluding initial syntax error and cryptographic historical-RED implementation provenance.

Isolated one-thread conditions are not recorded equivalent to original16-slot suite inputs/scheduling. Observed PASS does not repair frozen24.47ms delta, identify root cause or establish native/performance acceptance. Frozen **15,038 PASS / 3 FAIL / 19 skipped** unchanged; full handoff OPEN.

### Startup observations — 2026-10-11

[Retained runtime packet](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/test-perf/evidence/startup-cost-diagnostic-2026-10-11/README.md) preserves earlier denied guards and one admitted execution. Independent2542's first-new-probe authentication, preflight/readability/format and Cargo-record/path-stat library limits are reused, not repeated.

[Independent2549 coherence PASS](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/test-perf/evidence/startup-cost-diagnostic-2026-10-11/runtime-independent-report.md):224 phases (220 ordered addon calls plus initialization/discovery/post-workarounds/startup-events), one summary and1497 file records. Enclosing3.948s; phase sum3.907s; observer residual41.218ms. Sequence/readiness true; terminal success0/empty cgroup/process`[]`. No returned or explicit stream errors observed; internal error collector unmeasured. These are observations, not acceptance cases.

Profiled/CASC0/warm-cache clocks and nested file timings are not exclusive costs, benchmark equivalence or loaded-failure attribution. The first cache hit's compile bucket includes lookup/replay and potentially pack initialization ([source boundary](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/test-perf/evidence/startup-cost-diagnostic-2026-10-11/first-cache-hit-boundary.md)); component durations remain unmeasured. No native/current-production/frozen-BA payload proof or repair. Frozen **15,038 PASS / 3 FAIL / 19 skipped / 15,041 executed** and separate accepted scopes remain unchanged; full handoff OPEN.

### Returned addon-load fields — 2026-10-12

[Distinct inner-timing packet](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/test-perf/evidence/startup-addon-inner-timing-2026-10-11/README.md) captures existing `LoadResult` timing/frame/cache fields and diagnostic counts discarded by the completed outer probe. No production/API/vendor instrumentation or old-probe replay.

[Independent2556 compile/source gate](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/test-perf/evidence/startup-addon-inner-timing-2026-10-11/compile-independent-report.md): **PARTIAL_COMPILE_SOURCE**, rustc0/source unchanged; new artifact authenticated once. Reuse receipt, no rebuild/rehash. Library provenance remains Cargo-record/path-stat. Earlier preparation report's wrong-destination [output gap](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/test-perf/evidence/startup-addon-inner-timing-2026-10-11/preflight-output-gap.json) is retained, not invented PASS.

[Source boundary](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/test-perf/evidence/startup-addon-inner-timing-2026-10-11/source-boundary.md): read-only bytecode mode before environment creation; profiling0/load-only, no post-load workarounds/startup events/readiness. Earlier05:04UTC guard denied (load33.79/RAM21.600GiB/no Cargo), with no submission/observations at that stage; preparation output GAP remains history.

[Independent2558 coherence PASS](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/test-perf/evidence/startup-addon-inner-timing-2026-10-11/runtime-independent-report.md):220 returned results plus complete summary,7260 nonnegative values, both timing identities220/220. Diagnostics:90 missing requirements/185 nil observations/zero warnings across49 rows—not unique identities or global errorlessness. First authentication/source/format reused; terminal success0/no remaining probe.

Accounting total4.112s, XML processing2.225s, Lua execution0.891s=compile+call. [XML timing source scopes](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/test-perf/evidence/startup-addon-inner-timing-2026-10-11/xml-timing-scope.md) explain inclusive recursive finalize counters versus top-level create timers; aggregate subset labels do not define an exclusive hierarchy. Distinct discovery/result names and zero rows retained literally. Load-only observations are not paired with old clocks or loaded-failure/native/repair proof. Frozen BA3FAIL/full goal OPEN unchanged.

### Canonical paired startup — 2026-10-12

[Current packet](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/test-perf/evidence/startup-canonical-paired-2026-10-12/README.md) owns exact results/provenance; independent2794 accepts bounded current-run coherence at source `490bc1080`. One focused serializer and one canonical startup body PASS; parent/child recaps are not additional body executions. Compilation/formatting PASS; default check reused only for unchanged production/library scope. Initial `59f3ff136` compile E0583 remains separate; new seal first authenticated once. Public retention/link/privacy independently passed at `da4bbc442` within retained-text scope.

[Contract](../../specs/startup-addon-paired-observation.md) preserves original path/readiness/strict budget. Same-call rows are now paired in this current execution, not combined from earlier outer/load-only probes. Allocation/recording remain inside startup time; JSON emission is outside it. Emission before failed assertions is source-only: this passing run did not execute that failure boundary. Diagnostic counts are not compatibility-clean proof; overlapping loader buckets are not exclusive elapsed costs.

Captured allowlist values are null, including `WOW_SIM_LOG_HANDLER_TIMINGS` and `XDG_CACHE_HOME`. Actual bytecode enable/disable controls, `ASSET_RESOLVER_CACHE_DIR`, and `WOW_SIM_VERBOSE` child values are unknown; captured `WOW_SIM_BYTECODE_CACHE_DIR` is not an actual reader. No cache-contents/full-policy attestation, historical equivalence, loaded-failure cause/repair or native acceptance. Frozen BA verdict and full handoff OPEN unchanged.

### Performance scheduling boundary — 2026-10-12

[Bounded source interpretation](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/test-perf/evidence/startup-addon-inner-timing-2026-10-11/perf-scheduling-boundary.md): retained command header confirms16 Nextest slots. Linux performance wrappers request exclusive workload gates, but `WOW_SIM_TIMEOUT_FIXTURE_CAPACITY=reserved` bypasses acquisition. Original reservation state/competitor scopes unrecorded; no assertion that these tests raced or contention caused failure. Named scheduling sources unchanged since frozen BA. No rerun or scheduler/threshold fix justified by isolated passes alone.

### Future step metadata — 2026-10-12

[Full-suite workflow](../../remote-builds.md#asynchronous-full-suite) documents `3eab34fb9`'s per-step `command` argv and single-key `environment` record. Unset capacity serializes as `null`; empty and literal `reserved` remain distinct strings. No other inherited environment is captured. This future source addition cannot reconstruct original BA reservation state, explain its failures, or establish current suite acceptance. [Independent metadata-only PASS](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/test-perf/evidence/full-suite-result-metadata-2026-10-12/independent-report.md): three new cases/six existing sandbox controls; installed controller byte-equal/mode0755. No scheduling change or suite rerun; frozen BA3FAIL/full handoff OPEN unchanged.

## Sources

- [Canonical paired observation contract](../../specs/startup-addon-paired-observation.md) and [current packet](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/test-perf/evidence/startup-canonical-paired-2026-10-12/README.md)

- [Full-suite workflow and metadata limits](../../remote-builds.md#asynchronous-full-suite)

- [Distinct returned-field preparation and compile/source evidence](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/test-perf/evidence/startup-addon-inner-timing-2026-10-11/README.md)
- [Startup observation wrapper and public runtime packet](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/test-perf/evidence/startup-cost-diagnostic-2026-10-11/README.md)
- [Saved report/counts/selected failures](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/query-final-suite-result-2026-10-11/README.md)
- [Startup fixture](../../../tests/startup_perf.rs)
- [Talent tick fixture](../../../tests/talent_panel_onupdate_perf.rs)
- [Asset wrapper and child fixture](../../../src/asset_resolver_config.rs)
- [Timeout/re-exec lifecycle](../../../tests/common/timeout_reexec.rs)

## See Also

- [[integrated-source-and-factory-proof-2026-10-09]] — acceptance authority and bounded profile matrices
- [[stock-api-startup-2026-10-10]] — separate stock/third-party startup receipts
- [[casc-asset-cache]] — cache layers and input boundaries
