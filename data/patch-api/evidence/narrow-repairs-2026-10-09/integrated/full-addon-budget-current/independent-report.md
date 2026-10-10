# Independent diagnostic verification — 2026-10-10

**COMPLETE: diagnostic receipt verified. FAIL: error-free full-addon startup.** Not pending: outcome and controller completion records exist. Read-only inspection; only this report written. No execution rerun, build, operations, delegation, source edits, quota changes, vendor/model changes, or commits performed.

## Receipt and binary association

Receipt directory: `/home/osso/.local/state/wow-ui-sim/verification/full-addon-budget-current/20261010T174112Z`. Evidence: `/home/osso/.local/state/wow-ui-sim/verification/full-addon-budget-current/20261010T174112Z/submission.json`, `/home/osso/.local/state/wow-ui-sim/verification/full-addon-budget-current/20261010T174112Z/outcome.json`, `/home/osso/.local/state/wow-ui-sim/verification/full-addon-budget-current/request.jsonl`, `/home/osso/.local/state/wow-ui-sim/verification/full-addon-budget-current/controller.stdout`.

Outcome: exit **0**, completion marker present at `/home/osso/.local/state/wow-ui-sim/verification/full-addon-budget-current/20261010T174112Z/startup.stderr:378`: `[FullAddonStartupComplete] nil table nil`. Marker proves requested startup reached exec-Lua, not functional completeness or zero errors. Logging request sets existing `WOW_SIM_LOG_HANDLER_TIMINGS=1000`; sound and SavedVariables disabled; third-party addons not disabled. Dump-tree filter intentionally matches no frame. Stdout reports 291 Blizzard addons, 9 Blizzard dependencies, 49/79 third-party addons loaded, 1 failed during loading, 0 load failures, 1 loaded with Lua errors, 3008 Lua files, 151 XML files, 4 warnings. These summary categories are preserved literally, not conflated.

Sealed executable: `/home/osso/.local/state/wow-ui-sim/verification/count-api-startup-current/20261010T172719Z/wow-sim-sealed`.
Independently recomputed SHA-256: **c9dd3209fbc3c10e8c40dc930cdf3890e8725b4bbe7a0ff8704fe0168956c552**, matching receipt; outcome records artifact unchanged across execution.

Build association: `/home/osso/.local/state/wow-ui-sim/verification/count-api-startup-current/20261010T172719Z/submission.json` identifies revision **9635468d994be53d1f3571a8005115a49864c18a**. Its compile result records exit 0/source_equal true; source-before and source-after maps compare equal. Compiler JSON identifies default retail executable `/home/osso/Projects/wow/wow-ui-sim/target/debug/wow-sim`, `fresh=true`, including `retail-12-0-5`. Diagnostic submission's HEAD was **d6f767b11b11a072a3bc3e8a1dd3395e7c4d97df**, NOT the executable's revision. Current execution-budget, env-runtime, loader Lua-file and handler-timing file hashes match original source snapshot. This is reused-artifact diagnostic proof, not a new build or proof of all current HEAD changes. Original receipt explicitly excludes external dependency source-to-artifact provenance and external runtime assets; no stronger attestation claimed.

## First failures: exhaustion on entry versus consumption

All **104** counter records have limit **10,000,000**, numeric before/after, no unavailable snapshots. **102** entered exhausted; **2** show additional cumulative consumption reaching limit. There are **4 file** and **100 handler** records.

| Owner / boundary | Records | Already exhausted on entry | Consuming records |
|---|---:|---:|---:|
| EnhanceQoL file | 4 | 3 | 1 |
| EnhanceQoL handler | 96 | 96 | 0 |
| AllTheThings file | 0 | 0 | 0 |
| AllTheThings handler | 4 | 3 | 1 |

First EnhanceQoL failure: `/home/osso/.local/state/wow-ui-sim/verification/full-addon-budget-current/20261010T174112Z/startup.stderr:121`, file `@Interface/AddOns/EnhanceQoL/Settings/GroupTools.lua`, **9,994,637 → 10,000,000** (delta **5,363**). Owner nearly exhausted before entry, but not exhausted; this file's dynamic scope consumed remaining allowance. Next three files enter at 10,000,000: FocusInterruptTracker.lua, Mouse/Init.lua, Food/Init.lua. First callback at line 137, frame #69142 / ADDON_LOADED: **10,000,000 → 10,000,000**. Associated error at line 138 identifies `@Interface/AddOns/EnhanceQoL/Core/DynamicAnchors.lua:997`. Therefore first callback fails with preexisting entry exhaustion, not measured callback consumption. All 96 EnhanceQoL callback records have zero delta.

First AllTheThings failure: `/home/osso/.local/state/wow-ui-sim/verification/full-addon-budget-current/20261010T174112Z/startup.stderr:286`, frame #48610 / PLAYER_LOGIN: **770,081 → 10,000,000** (delta **9,229,919**). Error at line 287 and timing line 290 associate `@Interface/AddOns/AllTheThings/lib/EventRegistration.lua:26`. This callback's dynamic scope consumes remaining allowance; not already exhausted on entry. Three later ADDON_LOADED calls, frame #50975, enter exhausted with zero delta.

Deltas are owner cumulative charges within the logged scope, not exclusive self-instructions, proof of an infinite loop, or exact culprit source statements. Error source identifies a callback definition; file receipt identifies chunk, not failing file line. Other callbacks' source definitions are not supplied by counter records.

## Per-callback inventory

Line numbers below refer to `/home/osso/.local/state/wow-ui-sim/verification/full-addon-budget-current/20261010T174112Z/startup.stderr`. Every callback limit is 10,000,000. First counters shown; all repeat records are entry-exhausted with zero delta.

| Owner | Callback | Records | First line | First before → after |
|---|---|---:|---:|---|
| AllTheThings | `frame=#48610 event="PLAYER_LOGIN"` | 1 | 286 | 770081 → 10000000 |
| AllTheThings | `frame=#50975 event="ADDON_LOADED"` | 3 | 291 | 10000000 → 10000000 |
| EnhanceQoL | `frame=#69142 event="ADDON_LOADED"` | 10 | 137 | 10000000 → 10000000 |
| EnhanceQoL | `frame=#69143 event="ADDON_LOADED"` | 10 | 141 | 10000000 → 10000000 |
| EnhanceQoL | `frame=#69143 event="PLAYER_ENTERING_WORLD"` | 1 | 322 | 10000000 → 10000000 |
| EnhanceQoL | `frame=#69155 event="PLAYER_LOGIN"` | 1 | 301 | 10000000 → 10000000 |
| EnhanceQoL | `frame=#69157 event="ADDON_LOADED"` | 10 | 142 | 10000000 → 10000000 |
| EnhanceQoL | `frame=#69157 event="PLAYER_LOGIN"` | 1 | 302 | 10000000 → 10000000 |
| EnhanceQoL | `frame=#69160 event="PLAYER_ENTERING_WORLD"` | 1 | 323 | 10000000 → 10000000 |
| EnhanceQoL | `frame=#69162 event="DISPLAY_SIZE_CHANGED"` | 1 | 280 | 10000000 → 10000000 |
| EnhanceQoL | `frame=#69162 event="PLAYER_LOGIN"` | 1 | 303 | 10000000 → 10000000 |
| EnhanceQoL | `frame=#69162 event="UI_SCALE_CHANGED"` | 1 | 283 | 10000000 → 10000000 |
| EnhanceQoL | `frame=#69163 event="ADDON_LOADED"` | 10 | 143 | 10000000 → 10000000 |
| EnhanceQoL | `frame=#69163 event="PLAYER_LOGIN"` | 1 | 304 | 10000000 → 10000000 |
| EnhanceQoL | `frame=#69164 event="ADDON_LOADED"` | 10 | 144 | 10000000 → 10000000 |
| EnhanceQoL | `frame=#69164 event="PLAYER_LOGIN"` | 1 | 305 | 10000000 → 10000000 |
| EnhanceQoL | `frame=#69165 event="ADDON_LOADED"` | 10 | 145 | 10000000 → 10000000 |
| EnhanceQoL | `frame=#69165 event="PLAYER_ENTERING_WORLD"` | 1 | 324 | 10000000 → 10000000 |
| EnhanceQoL | `frame=#69166 event="PLAYER_LOGIN"` | 1 | 306 | 10000000 → 10000000 |
| EnhanceQoL | `frame=#69168 event="ADDON_LOADED"` | 10 | 146 | 10000000 → 10000000 |
| EnhanceQoL | `frame=#69168 event="DISPLAY_SIZE_CHANGED"` | 1 | 281 | 10000000 → 10000000 |
| EnhanceQoL | `frame=#69168 event="PLAYER_LOGIN"` | 1 | 307 | 10000000 → 10000000 |
| EnhanceQoL | `frame=#69168 event="UI_SCALE_CHANGED"` | 1 | 284 | 10000000 → 10000000 |
| EnhanceQoL | `frame=#69171 event="BAG_UPDATE_DELAYED"` | 1 | 345 | 10000000 → 10000000 |
| EnhanceQoL | `frame=#69173 event="PLAYER_ENTERING_WORLD"` | 1 | 325 | 10000000 → 10000000 |
| EnhanceQoL | `frame=#69176 event="ADDON_LOADED"` | 10 | 147 | 10000000 → 10000000 |

## Limits, reset units, and logging semantics

Source evidence (matching original source snapshot): `/home/osso/Projects/wow/wow-ui-sim/src/lua_api/execution_budget.rs:8-19` configures 10,000,000 only for a previously unconfigured owner; file and callback scopes share owner cumulative budget. Lines 22-32 exempt PLAYER_LOGOUT / ADDONS_UNLOADING; lines 34-67 log any failed owner callback when logging enabled, not only errors proven quota-caused. `/home/osso/Projects/wow/wow-ui-sim/src/loader/lua_file.rs:215-241` applies same failed-file diagnostics. Actual adjacent errors and exhausted snapshots here support quota association.

`/home/osso/Projects/wow/wow-ui-sim/src/lua_api/execution_budget.rs:137-148` resets configured addon usage; `/home/osso/Projects/wow/wow-ui-sim/src/lua_api/env_runtime.rs:275-282` calls that reset before OnUpdate under retail-12-0-5. No per-file, per-event, or wall-clock reset there. Receipt does not exercise/prove a reset transition; counters remain exhausted across startup events. Source-level reset policy verified, runtime reset behavior not proven by this capture.

Pinned dependency is rilua **842e4d3ff8592af45a5a93058eddf93419d29012** in `/home/osso/Projects/wow/wow-ui-sim/Cargo.toml:26` and `/home/osso/Projects/wow/wow-ui-sim/Cargo.lock:4994`. Inspected cached dependency `/home/osso/.cargo/git/checkouts/rilua-fd5a0715e46b5888/842e4d3/src/vm/state/instruction_budget.rs:8-14,44-50,58-91,106-115`: used counts executed dispatch instructions, not elapsed time or native Rust work; charge checks limit before dispatch then increments; errors do not refund; reset zeros usage; nested owner scopes replace active owner; exemption pauses charging without refilling. Dispatch charging call at `/home/osso/.cargo/git/checkouts/rilua-fd5a0715e46b5888/842e4d3/src/vm/execute.rs:1049`. Cached dependency inspection is semantic evidence, not fresh dependency build provenance.

`/home/osso/Projects/wow/wow-ui-sim/src/lua_api/handler_timing.rs` makes presence enable logging, numeric value filter duration records. **1000 means milliseconds threshold for timing lines, not instruction limit.** Counter failures are not filtered by that duration threshold. Recorded AllTheThings 1138.683 ms and Leatrix_Plus 1033.938 ms are separate wall-time observations; no conversion from dispatch count, no native WoW quota/reset parity inference.

## Errors and coverage limits

Full stderr processed: **417 lines**. **7 explicit `Lua error:` entries**, **7 distinct explicit messages**, plus suppression summaries **95 + 3 = 98** additional occurrences, totaling **105 reported Lua error occurrences**. Counter reconciliation: 4 EnhanceQoL file + 96 EnhanceQoL handler + 4 AllTheThings handler = **104 budget-associated occurrences**, plus **1 non-budget error** at `/home/osso/.local/state/wow-ui-sim/verification/full-addon-budget-current/20261010T174112Z/startup.stderr:366`, `Interface/AddOns/EllesmereUI/EllesmereUI_UICore.lua:1224: attempt to call a nil value`. Suppression groups represent deduplicated owner budget messages, not proof every suppressed callback has first callback's source. Timing records and counter records are not extra errors.

Run completed despite errors. This receipt proves first-failure cumulative instruction accounting and entry-exhaustion distinction for observed owners, with error counts and sealed identity. It does not prove all 79 addons loaded, no Lua errors, native quota compatibility, live GUI/OnUpdate reset behavior, SavedVariables-enabled behavior, or current HEAD build correctness. No remediation authorized or performed.
