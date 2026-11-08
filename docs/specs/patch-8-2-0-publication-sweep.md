# Patch 8.2.0 publication sweep

Audit Warcraft Wiki page 533235 revision 5142162 against current retail. Source: `data/patch-api/sources/8.2.0-*`. [Audit](../wiki/investigations/patch-8-2-0-api-audit.md).

## Requirements

- [x] Pin fetched current revision and retain every caption-table inventory and non-inventory occurrence with opt-in recipes.
- [x] Discover publication gaps through the prefork full-UI harness; apply later master registers chronologically, including the merged 8.2.5 register.
- [x] Model `C_VoiceChat.SetMasterVolumeScale` through existing voice-chat state for normalized settings-slider inputs; getter reflects writes, unrelated output volume remains unchanged. Reject invalid input without mutation to preserve the simulator state invariant; native out-of-range/security parity is not claimed.
- [x] Retire unused `C_UIWidgetManager.GetTextureWithStateVisualizationInfo` on retail only after whole-word cached/source/test consumer scans. Preserve Mists' existing lookup.
- [x] Account for every unresolved inventory/prose gap with exact reasons, without placeholder publication credit.
- [x] Prove all publication sweeps, targeted affected behavior, own source reproduction, all register reproductions, Python fixtures, format and warning-clean non-vendor Mists check. Preserve and report inherited saved-extract failures rather than rewriting prior sources.

## Remaining contracts

- [ ] Historical ledger records fifty-three exact inventory gaps and ten substantive prose contracts; exact reasons live in the audit's per-ID review. Literal fixture later became 52 after row135 removal; current observed 51 non-OK rows and independently audited row134-only reconciled owning PASS are recorded below; broader acceptance remains open.
- [x] Apply the merged 8.2.5 register; its six removals supersede six 8.2.0 publication gaps with bounded absence coverage.
- [ ] Inherited 12.0.5, 12.0.7 and 12.1.0 saved-extract reproduction failures remain unchanged, as in the completed 8.3.0 audit.

## Widget producer source checkpoint — 2026-10-14

`8645aaf0a` implements complete DoubleStatusBar26 / TextureAndText19 projections; `a62c81768` registers both and removes only `wt-global-api-C_UIWidgetManager.GetTextureAndTextVisualizationInfo-135` from known gaps. Requested ledger reconciliation is **54→53**; literal committed JSON arrays instead contain **53→52** entries across `a62c81768^`→`a62c81768`. This discrepancy is retained, not silently normalized or represented as a fresh sweep PASS. Historical counts below retain their original proof scope.

Row134 (`GetTextureAndTextRowVisualizationInfo`) was unchanged at this historical checkpoint; current observation follows below. DoubleStatusBar has zero evidenced publication rows; none invented. [TextureAndText owner](ui-widget-texture-and-text.md) and [DoubleStatusBar owner](ui-widget-double-status-bar.md) retain observed Retail RED4 PASS /8 FAIL, initial absence-only4 PASS, synthetic-fallback distinction and pending independent classification. Source is implemented but unverified: no behavioral GREEN, native/earlier availability UNKNOWN, no completion claim.

## Row134 owning observation — 2026-10-14

Original owning sweep at `c35ddb7ed`, against the unchanged 52-gap fixture, emitted **193 unique observed rows: 142 OK / 51 non-OK**. Historical outcome remains **FAIL, exit 1**. Raw failure reports no new gaps and exactly one resolved/stale gap: `wt-global-api-C_UIWidgetManager.GetTextureAndTextRowVisualizationInfo-134`; its emitted observation is `raw=function; lookup=function`, OK.

`bb9ea16fe` removes only that one fixture line, yielding the literal **51**-entry expected-gap array. Independent final saved-evidence audit confirms the after-reconciliation rerun at `bb9ea16fe`: raw exit 0, one primary PASS; three zero-case auxiliary summaries excluded. Full 193 objects exactly equal original `c35ddb7ed` output, with 142 OK / 51 non-OK matching the reconciled 51-entry fixture. No runtime behavior changed since before: only fixture-comparison outcome changed. After compile: 14.73s, 741 records, six manifest warnings, peak 3,442,733,056 bytes below 5G high. Bounded owning-after gate PASS; not broader acceptance. Historical row135-only 53→52 reconciliation and earlier ledger discrepancy above remain unchanged.

[Row contract](texture-and-text-row.md) owns behavior proof: independent PTR at `7f44eb773` passes eight unique cases, exit 0, including actual cached consumer and host secrets; earlier Retail eight row cases plus three controls remain separate. Publication presence is not those behavioral gates. The [row proof ledger](texture-and-text-row.md#known-gaps-current-cycle) owns the separate completed Retail normal-before and PTR normal diagnostic audits, verified PTR feature/cache-selection semantics, and independent final format/default-check PASS at `484e93c06`, including six warnings and the 987,136-byte high-threshold crossing. Startup-event stderr is suppressed by the source LuaErrors collector: complete saved logs cannot recover suppressed messages, so neither diagnostic establishes universal startup-clean behavior. Read-only WTF/layout-cache dependencies remain despite SavedVariables disabled. Public [whole-TEXT packet](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/ui-widget-texture-and-text-row-2026-10-14/README.md) is committed at `33cbb6baee0e8665770d1d678062eb37b92e6f09`; [row publication ledger](texture-and-text-row.md#known-gaps-current-cycle) owns exact counts, accepted-candidate byte equality, bounded candidate/canonical integrity/privacy/navigation PASS. No new code/runtime proof, native/cache-byte/executable-byte authentication or full-parent closure. All-profile/broad/provider/performance/credit acceptance remains open.

## Tests

`tests/patch_8_2_0_publication_sweep.rs`, `patch_8_2_0_publication_fixes.rs`, `patch_8_2_0_cached_surfaces.rs`, `patch_8_2_0_classic_surfaces.rs`; extractor/generator behavioral fixtures. Evidence and dynamic validator: `data/patch-api/evidence/8.2.0-session-2026-10-08/`.

## Exclusions

Publication alone is not signature, security, populated-output, historical or payload parity. No vendor/cache/Wowless edits, shims, Blizzard monkey-patches, full integration suite, push, merge or agents. Inherited saved-extract non-reproducibility remains recorded rather than silently rewritten.
