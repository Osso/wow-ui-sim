# Saved 5aa actual-delta audit

verified: 2026-10-10

Derived from the independent saved-artifact report and saved 5aa JSON; no suite execution or raw vendor logs published. Exact source: `5aa6cb277eb5919e9276100c023f1c98809c7bc9`; comparator: `c6bc87c120e43199dadeb299691511b03a81408a`.

| Scope | c6bc PASS/FAIL | 5aa PASS/FAIL | 5aa exit |
|---|---:|---:|---:|
| Integration | 10679/2 | 10680/2 | 100 |
| Prefork (main and auxiliary) | 2324/1 | 2324/1 | 1 |
| Library | 1996/0 | 1997/0 | 0 |
| Total | 14999/3 | 15001/3 | FAIL |

Integration skips remain 19. Skipped identities are absent from saved streams; identical skipped identity sets are unproven. Independent named-outcome comparison reports two additions, zero removals, zero changed shared statuses.

## Exact bounded PASS evidence

- Library `loader::tests::lua_loading::toc_normal_lua_files_share_varargs_table_only_within_each_addon`: added PASS, saved 5aa line 15194. Actual TOC order, exactly two varargs, within-addon private-table sharing and cross-addon isolation.
- Integration `secure_group_headers::cached_auto_hide_child_rectangle_extends_hover_set`: added PASS, line 9043. Cached vendor dependency closure, tainted registration, disjoint target/child rectangles, treatment/control dwell beyond TTL, exit from both rectangles and Lua error boundaries. Existing `cached_auto_hide_enter_leave_expires_after_duration` and `cached_auto_hide_unregister_cancels_pending_expiry` remain PASS, lines 9031/9073.
- Prefork `patch_12_0_0_deprecated::patch_12_0_0_deprecated_retirement_and_successors`: PASS, line 13817 (c6bc 13804). One executed case covering exactly 21 deprecated extract occurrences, not 21 tests. Checks native retirement, cached absence or loaded alias identity and seeded successor probes. Prior c6bc Minimap wrong-case substitution is excluded; Minimap is not this evidence.

## Unchanged failures

- `method_diff_coverage::diff_methods_extra_snapshot_matches_current_metatable_surface`
- `method_diff_coverage::diff_methods_missing_snapshot_matches_current_metatable_surface`
- `blizzard_garrison_ui_loads::blizzard_garrison_ui_loads_explicitly_via_load_addon_without_errors`

Saved JSON calls Garrison a `new_failures` entry; independent identity comparison shows it already failed at c6bc. No repair or waiver inferred.

## Limits

Overall FAIL remains. Later party commit `439260ee585b44f1d07c79ed4ce2ea77649f1ead` and current-party 439/40 coverage receive no credit. No current-HEAD, native WoW parity, all-profile, clean-startup, warning-free, CI acceptance or deployment claim. Offline/locked commands do not seal external caches, path dependencies, inherited environment or source-to-binary provenance. No explicit no-addons/no-SavedVariables environment attestation. Independent report's frozen-source observations are attributed evidence, not a fresh source audit here.

## Private inputs

- `/home/osso/.local/state/wow-ui-sim/verification/fullsuite-5aa-independent/report.md`
- `/home/osso/Projects/wow/full-suite-results/5aa6cb277eb5919e9276100c023f1c98809c7bc9.json`

[Receipt](receipt.json) retains saved JSON timing/exits and report-derived counts; [manifest](manifest.json) binds published bytes and private input hashes. Raw logs remain private.
