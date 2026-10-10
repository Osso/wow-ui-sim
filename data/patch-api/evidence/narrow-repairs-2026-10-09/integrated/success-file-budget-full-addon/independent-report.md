# Independent full-addon attribution proof — 2026-10-10

**PASS: bounded capture and exact parser attribution. Startup repair/clean startup NOT established.** Read-only receipt/source inspection; no builds, tests, checks, simulator executions, operational changes, or delegation. Receipts became available within the bounded wait.

## Source, artifact, cache, completion

Evidence root: `20261010T183739Z/` beside this report.

- Submission revision `b048251f38b2d53e33b3365590cd227d40fc07d0`; loader and execution-budget committed blobs at implementation `0e4bf4161` match captured SHA-256 values. All 3,853 captured tracked-source paths match before/after and independently read current bytes. Scope follows the worker's tracked-file selection, not an entire-input attestation.
- `compile-result.json`: exit 0, source equality true, no stream errors. Full compiler JSON contains one executable artifact for wow-sim and successful build-finished. Artifact features match receipt, including default/client-retail/retail-12-0-5; compiler `fresh=false`. This is a newly emitted normal default-Retail artifact, not proof every dependency was rebuilt.
- Sealed executable SHA-256 independently recomputed: `4981999d11f8ebac61113f91ee89ba32d81ef1536cba45fa34ac023a2ccefd5b`. Matches artifact receipt and currently retained original executable. Runtime receipt records unchanged artifact and source.
- Vendor-cache maps: 4,044 paths, before/after equal. Independent full current cache enumeration/hashing equals after-map. Worker after-map rehashes the before-list rather than independently enumerating new files; current enumeration strengthens endpoint evidence, not a continuous seal.
- Runtime submission invokes sealed binary through timeout 90, full-addon loading, no SavedVariables, sound disabled, existing diagnostic opt-in 1000. Runtime exit 0 and exactly one completion marker observed in actual streams; `outcome.json` records CAPTURED. Third-party summary reports 49/79 loaded, one failed during loading, zero load failures, one loaded with Lua errors, four warnings. These categories are distinct. Completion is not error-free acceptance.

## Aggregate diagnostics and private attribution

Entire stderr inspected privately: 3,470 lines; stdout: 49 lines.

| Diagnostic | Structurally valid records |
|---|---:|
| File success | 3,064 |
| File error | 4 |
| Handler error | 91 |
| Total | 3,159 |

Zero malformed diagnostic records. All observed limits 10,000,000; usage numeric and within-record nondecreasing. Of 95 error diagnostics, 93 enter already exhausted; two increase usage. Error tags describe failed execution, not automatic error-cause classification.

Main-invoked parser receipt: exit 0. Independently reconstructed each file record from raw stderr and compared all 3,068 records exactly: physical line, kind, decoded owner/file, limit, before/after and delta. Exact owner grouping and per-owner counts verified privately; 50 owners. Public summary matches reconstructed counts: 3,064 success, four error, zero rejected, zero continuity anomalies. Adjacent same-owner file counters have no observed gaps or limit changes. Preserved parser input equals raw stderr byte-for-byte. Private output directory mode 0700; three parser files mode 0600. No owner names, chunks, frame identities, error payloads or per-owner data published here.

## Historical comparison and limits

Earlier saved diagnostic has 104 error counter records (four file, 100 handler); current has 95 (four file, 91 handler). This is an exact tag-count comparison only. Earlier 105 errors are a separate historical error inventory, not the same metric and not a current baseline acceptance result. Changed source epochs and unsealed runtime inputs prevent attributing the count difference to a fix. Successful-file logging fills an attribution gap; it does not establish repaired startup or reduced error causes.

Counters are owner-cumulative dispatch instructions. File deltas include synchronous dynamic-scope work, not exclusive file cost; nested scopes can overlap. Do not sum deltas as exclusive totals or translate them into elapsed time/native quota parity. 1000 is the duration logging threshold in milliseconds, not instruction quota. File evidence excludes untainted execution and pre-execution failures; handler tags cover failed owner-budgeted callbacks, not all handlers or all startup errors.

Third-party addon bytes/configuration remain unsealed. NoSavedVariables does not eliminate WTF state: stdout explicitly reports EditMode layout-cache loading. Inherited environment is not fully sealed; explicit environment entries alone do not prove all overrides absent. External dependency/source-to-artifact provenance, untracked inputs, other runtime assets, transient in-window changes, native parity, GUI behavior, all-profile acceptance and reset-period runtime behavior remain unproven. Report proves this captured receipt boundary only; startup remains unfixed/unaccepted.
