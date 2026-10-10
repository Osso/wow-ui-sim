# Saved three-owner epoch audit

**PASS — bounded target evidence. Not globally clean startup.**

## Identity and seals
- Epoch `20261010T042920Z`; recorded source `0148437a7868b0a25d0ce2fca504378d4558b5ee`.
- Compile exit **0**, Cargo `build-finished.success=true`; 657 compiler artifacts, 74 build-script records, one completion record. One selected `wow-sim` artifact, `fresh=false`, test profile, **default Retail** features (including `default`, `client-retail`, `retail-12-0-5`). Six manifest deprecation warnings remain.
- One saved exact ignored target invocation: `startup::tests::trusted_finite_budget_for_selected_addons_with_shared_media`. **1 PASS, 0 failures, exit 0**, 30 filtered out (`stdout:4713`). No new execution.
- Recorded build-end, invocation-before and result-after executable SHA-256 all equal `b48a89c7985bb7b42e77db8f6d05ccbc95264a43c7adc90160125d61e69ebb6d`. Both raw runtime stream hashes match result.json. This audits historical recorded seals, not a new executable measurement.
- Compile-bracketing source maps: **3839 entries, equal**. Selected relevant revision sources independently hash-match their map entries. Maps exclude external path dependencies, data outside tracked src/tests/build/config scope, inherited environment, external runtime cache/addon files, and untracked index. These are not complete source-input seals or runtime-bracketing source seals.

## Owner evidence
All three phases retain `limit=100000000`; each owner records `quota_errors=0` in every phase.

| Owner | Before init used | After load + GC used | After settle used |
|---|---:|---:|---:|
| EnhanceQoL | 0 | 12645062 | 0 |
| AllTheThings | 0 | 771025 | 0 |
| EnhanceQoLSharedMedia | 0 | 18488983 | 0 |

Evidence: `stderr:12-17`, `2848-2853`, `11733-11739`. SharedMedia `encountered=true loaded=true` at final boundary. Positive load counts prove metered load work for all three, including SharedMedia. Final zeros are **reset counter snapshots, not zero work** or cumulative settle-work totals. No trigger/callee inference.

## Remaining GLOBAL errors — payload-free
Full runtime stdout (317318 bytes / 4714 lines) and stderr (1499270 bytes / 11739 lines) read privately; compile streams also fully read. **1 observed global Lua error occurrence, 1 safe signature/class**, not filtered to the selected owners:

| Count | Safe location | Class | Evidence |
|---:|---|---|---|
| 1 | `Interface/AddOns/EllesmereUI/EllesmereUI_UICore.lua:1224` | attempt to call a nil value; unidentified callee | `probe-execution/stderr:4165` |

No payloads, arguments, or traceback text reproduced. No gamepad callee guess. `stdout:4638` reports zero addon loading-error totals, but does not erase this later global error. **327 nil-global debug lookup lines are diagnostics, not thrown-error counts.** No handler/file budget-error tags or instruction-exhaustion markers found. Per-owner zero quota-error snapshots do not imply global cleanliness. Saved streams do not include a complete final serialized global error-count inventory; occurrence count is explicitly stream-observed.

## Compiled source and scope
`src/bin/wow_sim/startup.rs:26-28` retains the default caller; the ignored test calls the extracted initializer and ordinary headless settle path. CargoJSON proves default Retail test-target compilation, not a separate normal-main runtime run. `src/loader/lua_file.rs:223-228` and `src/lua_api/execution_budget.rs:48-51` gate budget diagnostic snapshots/emission using `handler_timing::is_enabled`; helper definition `src/lua_api/handler_timing.rs:12-14` is feature-gated on `retail-12-0-5`, present in this compiled artifact. Invocation sets timing env to `0` (enabled). The helper/caller is not inferred from an alternate profile.

## Resource and provenance limits
- `CARGO_BUILD_JOBS=12`. Saved own build cgroup `cpu.max=1200000 100000` (**1200%**) and `memory.max=17179869184` (**16 GiB**), before and after. Peak memory 9959763968 bytes; build CPU usage 174713534 usec, 590 periods, 2 throttled periods, 729 throttled usec. Before CPU counters absent; no delta claimed. No runtime cgroup snapshot supplied.
- Parent cap not sealed in these files; may now be **20 rather than 12**. No current inspection, bottleneck inference, or speedup claim.
- No contemporaneous cache/addon/Blizzard UI content run seals supplied. No cache audit performed. A later cache audit would not prove historical run inputs unchanged; bytecode-cache hit statistics are not run seals.
- Old two-owner proof remains separate and untouched; no evidence imported or claims retroactively changed.

No reruns, builds, tests, polling, delegation, or operational changes. Only saved-file audit and read-only revision inspection. No native, parity, full-startup-clean, or production-budget claims.

`aggregate.json` contains structured evidence; `hashmanifest.json` hashes all 15 supplied input files plus report and aggregate. Manifest excludes itself to avoid circular hashing. Raw streams remain private in their original locations.
