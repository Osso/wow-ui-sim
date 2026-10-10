# Stock API startup — 2026-10-10

Two completed default Retail headless executions in receipt epoch `20261010T172719Z` passed the scoped API-registration/bootstrap and observed stock-startup gate. Warm-cache evidence only; no GUI/rendering or native-client acceptance.

## Observed coverage

| Execution | Completion proof | Runtime diagnostics | Result |
|---|---|---|---|
| `--no-addons --no-saved-vars lua-errors` | `[]` stdout; final CLEAN summary, zero unique/occurrence errors | 0 root, wrapper, other error/panic/failure, budget and warning diagnostics | exit 0 |
| `--no-addons --no-saved-vars --exec-lua "print('[CountApiStartupComplete]')" dump-tree --filter __CountApiStartup_NoMatchingFrame__` | Exactly one marker after startup events; Frame Tree header; completed exit | Same zero counts | exit 0 |

No marker was requested for `lua-errors`. The deliberately empty filtered tree is not startup proof. Counts describe emitted diagnostics, not invisible failures or exhaustive API correctness.

## Identity and limits

Successful guarded build reused a fresh normal `bin/wow-sim` artifact, not a test harness or newly compiled binary. Default features include GUI; no GUI execution or rendering was performed. Six distinct manifest-deprecation warnings plus one aggregate warning line remain; no warning-free build claim.

Both runs loaded 291 Blizzard addons with 1,569/1,569 bytecode-cache hits. Third-party addons and SavedVariables were disabled, but existing WTF EditMode layout-cache data was still read. Private character, realm and layout-selection identifiers are redacted from the retained report; raw runtime streams are not published.

Submission revision `9635468d994be53d1f3571a8005115a49864c18a`; independently observed HEAD `3ab3633cc8e08dbba22b361aaf01250a6a7a875a`. All 3,853 worker-hashed files matched despite HEAD movement. Tracked-scope before/after equality and unchanged sealed-binary checks do not establish complete source-to-artifact provenance. Runtime-cache contents, WTF bytes, CASC assets, external dependencies and inherited environment are not sealed. No cold-startup, native-WoW, all-profile, layout-correctness or current-HEAD acceptance claim.

## Sources

- [Sanitized independent report](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/count-api-startup/independent-report.md) — completed receipt inspection and bounded findings
- [Execution receipts](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/count-api-startup/execution-results.json) — two actual exits and artifact checks
- [Marker summary](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/count-api-startup/marker-summary.json) — completion, diagnostics and private-stream parent hashes
- [SHA-256 manifest](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/count-api-startup/SHA256.json) — retained identities and original parent-report hash

## See Also

- [[integrated-source-and-factory-proof-2026-10-09]] — broader ledger; unchanged by this scoped publication.
