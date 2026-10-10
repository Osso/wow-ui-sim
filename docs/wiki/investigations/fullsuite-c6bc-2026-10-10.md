# Saved fullsuite c6bc — accounting and main erratum

Verified 2026-10-10. Saved source `c6bc87c120e43199dadeb299691511b03a81408a`: **14,999 PASS / 3 unchanged FAIL / 19 integration skips**. Overall FAIL; parent remains OPEN. This page owns the c6bc-versus-bc58 delta and corrected exact retirement lookup, not current runtime acceptance.

## Saved scope and delta

Execution window: October 10, 2026, 11:36:26–11:59:56 -0500. Saved execution scopes:

| Scope | PASS | FAIL | Skips | Exit |
|---|---:|---:|---:|---:|
| Integration | 10,679 | 2 | 19 | 100 |
| Prefork, including auxiliary groups | 2,324 | 1 | None reported | 1 |
| Library | 1,996 | 0 | 0 | 0 |

Compared with `bc58b43b577f1ac6647d4765d50e43afa36302ca`: +3 executed/PASS library cases, no added failing identities, no resolved failures, and unchanged integration/prefork identity sets. Prefork main group alone is not its total scope. Skipped identities are unavailable in the saved stream.

All three added library cases PASS under `iced_app::mouse::mouse_test_modules::registration_tests::`:

- `lua_create_frame_applies_xml_template_click_edges_before_onload_and_dispatch` — saved log line 14683.
- `xml_click_templates_replace_inherited_edges_before_onload_and_physical_dispatch` — line 14689.
- `xml_register_for_clicks_applies_literal_edges_preserves_default_and_allows_lua_mutation` — line 14690.

## Unchanged failure identities

- `method_diff_coverage::diff_methods_extra_snapshot_matches_current_metatable_surface`
- `method_diff_coverage::diff_methods_missing_snapshot_matches_current_metatable_surface`
- `blizzard_garrison_ui_loads::blizzard_garrison_ui_loads_explicitly_via_load_addon_without_errors`

The JSON receipt lists Garrison under `new_failures`; that is runner-relative metadata, **not a newly failing identity versus bc58**. Neither snapshot nor Garrison failure is repaired or waived by this accounting.

## Main erratum: literal retirement PASS

Main independently read actual saved log lines 13799–13810. Line **13804** reads:

`test patch_12_0_0_deprecated::patch_12_0_0_deprecated_retirement_and_successors ... ok`

That exact saved prefork case is PASS. The independent audit's Minimap/cached21 lookup is **incorrect as the answer to this requested case**, not a correct alias or replacement. The sanitized original audit remains preserved for traceability, alongside the explicit [main erratum](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/fullsuite-c6bc/main-erratum.md); its lookup must not be endorsed.

## Source and acceptance boundary

c6bc includes XML production `33d62d705` and the three added library cases. It does not establish coverage of later TOC private-table changes (`9d074f6a8`), cast-count changes (`84dcf2723`), or later AutoHide changes. Existing selector names do not transfer proof to later changed code or assertions.

This is saved-source accounting, not current-HEAD PASS, native WoW parity, all-profile coverage, clean startup, deployment health or CI acceptance. The unchanged six distinct manifest warnings remain; no warning-free claim. Receipt SHA and retained hashes do not seal historical source-to-binary provenance, external dependencies, environment or runtime caches. No tests/builds/reruns/operations were performed for this publication.

## Sources

- [Sanitized original independent audit](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/fullsuite-c6bc/independent-report.md) — saved count/delta audit; erroneous retirement lookup explicitly superseded.
- [Saved receipt JSON](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/fullsuite-c6bc/receipt.json) — source identity, timing, exits and failure metadata.
- [Main erratum](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/fullsuite-c6bc/main-erratum.md) — direct main read of literal line 13804.
- [Hash manifest](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/fullsuite-c6bc/hash-manifest.json) — retained derivatives and private source hashes; no raw/vendor log retained.

## See Also

- [[integrated-source-and-factory-proof-2026-10-09]] — integrated proof SSOT; cross-link integration owned by main/count-doc agent.
