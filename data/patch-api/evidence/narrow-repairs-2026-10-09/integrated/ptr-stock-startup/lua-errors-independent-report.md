# Independent missing-only PTR stored-Lua-error CLI receipt audit

**PASS — exact public stored-error CLI result for this distinct PTR command-mode execution: `[]`, 0 records, exit 0.** This closes the missing command-mode receipt only; preceding normal dump's same-in-memory error state remains UNPROVEN. Native-client and all-profile acceptance remain unproven.

Verified 2026-10-10. Evidence: `20261010T184101Z/`. Followed `/home/osso/AgentConfig/skills/verify/SKILL.md` as verifier. Read-only receipt/source/cache inspection; no simulator rerun, dump, build, tests, operations or delegation. Only this report written.

## Actual result and full stderr

- `lua-errors-result.json` records `/usr/bin/env -u WOW_SIM_LOG_HANDLER_TIMINGS /usr/bin/timeout 90 <evidence>/wow-sim-sealed --no-addons --no-saved-vars lua-errors`, exit **0**, artifact unchanged. Distinct command, not repeated dump/build. Timeout bound is 90 seconds; receipt contains no independent wall-clock duration. Final timed startup log is 2.299s, not total execution duration.
- Independently parsed all stdout: exactly **3 bytes**, `[]\n`; valid JSON array, **0 records**. Not empty/missing output. Matches receipt `json_record_count=0`, `stdout_empty=false`.
- Read entire stderr: **14,675 bytes**, **219 newline-delimited lines** (read tool reports 220 including terminal empty line). Final five lines, verbatim:

```text
=== Final observed Lua error summary ===
Status: CLEAN
Lua errors: 0 unique, 0 occurrence(s)
Attributed owners: 0
Unattributed Lua errors: 0 occurrence(s)
```

- Full capture has **0** `Lua error:` emissions, WARN/warning/panic diagnostic matches, handler-budget-error or file-budget-error records. These emission counts alone do not establish errorlessness: `src/lua_errors.rs:27-38` suppresses stderr during command startup collection. The parsed stored-state JSON and final summary provide the requested evidence.
- Logs explicitly say `SavedVariables loading disabled`, `Addon loading disabled`, and `Sound disabled`. Separate read-only WTF EditMode cache was loaded despite ordinary SavedVariables being disabled. Font setup logs `casc=false`; `src/bin/wow_sim/startup_trace.rs:22-27` deliberately selects `new_without_casc()` for LuaErrors. Therefore this command mode is not identical to normal dump/runtime setup.

## Artifact and profile binding

Independent SHA-256 of 414,545,832-byte sealed executable:

```text
68a834b307bbd1a31af8e84880d8eecee02aeecf4fada526a93583eb6a72ff34
```

Matches both `artifact.json` and `lua-errors-result.json`. Saved compiler JSON contains exactly **1** executable wow-sim compiler-artifact; executable path and feature array match `artifact.json`. Revision recorded: `b048251f38b2d53e33b3365590cd227d40fc07d0`. Features include `client-ptr`, `retail-12-1-5`, `gui`, `casc`, `sound`; exactly one `client-*` selector, no `default` or `profile-retail`. `src/client_profile.rs:165-174` selects `ClientProfile::Ptr` for that combination. This binds receipt argv to the same sealed PTR artifact, not a complete reproducible-build provenance chain or authenticated execution attestation.

Receipt/capture hashes independently computed:

| File | Bytes | SHA-256 |
| --- | ---: | --- |
| lua-errors-result.json | 617 | `7eb051ea169fd42ce166dd0b89cd01d0cf9a81f38d85dc64b8e264f1214d07b1` |
| lua-errors.stdout | 3 | `37517e5f3dc66819f61f5a7bb8ace1921282415f10551d2defa5c3eb0985b570` |
| lua-errors.stderr | 14675 | `a82a82af3db0ba92fbdf16aea67c8f91187367067e6fc2cb56d56ddb5caec729` |

## Saved-map equality against current files

Independently parsed and compared saved maps, then SHA-256 hashed every covered current file:

| Scope | Before/after | Current covered files |
| --- | --- | --- |
| Repository source maps | **3,853 entries each, identical** | **3,853 match; 0 missing/hash mismatches** |
| `/home/osso/.cache/wow-ui-sim/blizzard-ui/ptr/AddOns` maps | **4,027 entries each, identical** | **4,027 match; 0 missing/hash mismatches** |

Current cache file inventory independently enumerated: **4,027 files**, **0 extras**, **0 missing** versus saved map. Source coverage includes `data/blizzard-ui-files/ptr.txt`; source equality covers saved keys only, not untracked additions or external/path dependencies. These existing maps and present-time checks do not newly bracket the distinct lua-errors execution or exclude transient changes between observations. Runtime logs do not independently establish the exact absolute cache directory opened; inherited environment/runtime inputs are not fully sealed.

## Public-state contract and limits

`src/bin/wow_sim/main.rs:493` dispatches LuaErrors; `:549-558` calls the collector and exits 1 for a non-clean result. `src/lua_errors.rs:77-83` collects unique errors, emits final summary, serializes the array and returns whether it is empty. `:214-223,268-280` reads simulator error state and its count map; JSON represents that public deduplicated projection. `:86-112` produces summary totals and owner attribution from stored state. No raw-memory inspection or invariant audit of every internal error container was performed.

**Established:** this recorded bounded CLI execution returned exact public stored-error output `[]`, with 0 unique errors / 0 occurrences reported, using the hash-matched sealed PTR artifact; covered source and specified PTR cache currently equal saved maps.

**Not established:** same in-memory errors as preceding normal dump (separate process and startup mode), unsuppressed callback diagnostics, native WoW parity, GUI/rendering correctness, all-profile compatibility, cold/no-cache startup, complete environment/input provenance, or a warning-free build. Prior `independent-report.md` remains unchanged; its normal-dump stored-state limitation is not retroactively replaced by this CLI result.
