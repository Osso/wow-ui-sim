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

## Later four-profile normal captures — 2026-10-11 UTC

Separate [independent report](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/four-profile-stock-startup/independent-verification-report.md) admits receipt provenance, **not clean startup**. Epoch003325 uses four separately sealed normal binaries, with `WOW_SIM_CASC=0`, no third-party addons or SavedVariables; completion and `lua-errors` are distinct invocations.

| Profile | Completion exit / markers | Lua-error invocation | Acceptance |
|---|---|---|---|
| Wrath | 1 / 0 | exit1; no JSON result | Blocked before Blizzard load: source cache missing, automatic source sync cannot discover WoW installation |
| Mists | 0 / 1 | exit1;2 unique/2 occurrences | Completion only; self-anchor error plus ScriptErrors wrapper, one each |
| Era | 1 / 0 | exit1; no JSON result | Same missing-source/install abort; Lua compatibility unassessed |
| Anniversary | 1 / 0 | exit1; no JSON result | Same missing-source/install abort; native BCC selection still unverified |

Mists marker is in stderr, followed by Frame Tree progression. Empty filtered stdout alone proves nothing. [Exact Mists JSON](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/four-profile-stock-startup/mists/lua-errors.json) repeats `Cannot anchor to itself` on `__Blizzard_UIPanels_Game_20279`; two signatures are not two independently established causes. Candidate ProductChoice relativeKey attribution remains source-only. This capture does not execute or repair the [original hidden-state assertion](mists-cast-startup-visibility.md).

[Compiler summary](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/four-profile-stock-startup/compiler-summary.json): four Cargo exits0; each653/653 artifacts fresh,74 build-script records,1 successful build-finished. Cached materialization totals1.672370s across four recorded compile windows, not a fresh recompilation or controlled speedup. Each invocation retains six manifest deprecations; zero compiler-message warnings does not mean warning-free. Earlier jobs12 integration costs remain separate. Compile cgroups quota12/16GiB; runtime logs2CPU, not measured core saturation or affinity.

All3,846 tracked proof-scope hashes match captured endpoints/current inspection, not dependency/data/vendor/cache/WTF/environment closure. Mists uses1094/1094 bytecode-cache hits and existing read-only EditMode cache. `WOW_SIM_CASC=0` does not prohibit source synchronization; missing-profile captures attempted it and aborted. Source caches were not historically snapshotted; main observed Wrath/Era/Anniversary absent and Mists present. [Retained summaries/excerpts](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/four-profile-stock-startup/retention-summary.json) exclude raw private runtime streams. No cold-state/GUI/assets/native/full-addon/all-profile cleanliness or parent completion.

## Sources

- [Sanitized independent report](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/count-api-startup/independent-report.md) — completed receipt inspection and bounded findings
- [Execution receipts](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/count-api-startup/execution-results.json) — two actual exits and artifact checks
- [Marker summary](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/count-api-startup/marker-summary.json) — completion, diagnostics and private-stream parent hashes
- [SHA-256 manifest](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/count-api-startup/SHA256.json) — retained identities and original parent-report hash

## See Also

- [[integrated-source-and-factory-proof-2026-10-09]] — broader ledger; unchanged by this scoped publication.
