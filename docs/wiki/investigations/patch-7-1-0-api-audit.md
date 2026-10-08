# Patch 7.1.0 API audit

Verified source: 2026-10-08. Refetched pageid 389232 pins revision **3742660**, timestamp **2022-01-31T18:50:28Z**. Ten inventory identities plus twenty-one extract rows account for all **31 IDs**. Current-retail publication has **zero gaps** after later supersession; five substantive prose contracts remain pending.

## Coverage matrix

| Statement | Bounded coverage | Missing / problematic boundary |
|---|---|---|
| GetPhysicalScreenSize | Existing physical-display model and screen-mode tests | No historical 7.1.0 policy claim |
| GetDetailedItemLevelInfo / GetItemInfo | 10.2.6 removal recognized; exact cached deprecated C_Item aliases observed; catalog binding, expansion and base level/link query tested | Item-set ID and crafting-reagent returns are constant defaults, not modeled metadata; full four-return parity pending |
| SetClipsChildren / DoesClipChildren / XML clipChildren | Independent state toggle; inherited initial flag and explicit false override before OnLoad | Not pixel/hit-test clipping proof |
| XML intrinsic | Existing known intrinsic alias tested separately | Custom registered P710ClipIntrinsic rejected by runtime tag resolver; failed probe retained |
| Five personal-nameplate CVars | Explicit 12.0.0 removals supersede additions; value/default absence observed | Historical visibility/delay/alpha semantics not credited |
| ScrollingMessageFrame implemented in Lua | Current runtime maps to MessageFrame plus ScrollingMessageFrame Lua template | No pinned old implementation or historical migration proof |
| Mouse OnEnter/OnLeave eligibility | Full statement and bug qualification retained | Page explicitly identifies a pre-release bug; no final-client policy capture justifies changing shared hover dispatch |
| TitleRegion removal | Exact-name scans and later-register check retained | Bare object name lacks native factory/method contract; no guessed retirement |

## Discovery and existing models

Opt-in `--top-level-api-bullets` captures both new globals. Additive `--legacy-widget-cvar-bullets` separately captures two linked Frame methods, five nested CVar names and underscore-linked changed GetItemInfo. XML attributes and bare TitleRegion are retained as prose rather than invented global publications. No extractor behavior changes.

Publication discovery passes with all ten observations. Five personal-nameplate CVars are absent after 12.0.0 removal. Both item globals are current cached deprecation aliases to their C_Item successors; exact target identity and unmodified source provenance are observed, not treated as native historical publication.

New bounded tests exercise existing real `Frame.clips_children` state and actual item catalog fields. Existing screen tests distinguish physical pixels from UI units and preserve display/scale updates. No runtime source, C API registration, shim, backing-model placement or vendor behavior is changed.

The first cached behavior experiment fails `CreateFrame('P710ClipIntrinsic')` despite a registered XML intrinsic declaration. Root cause: `resolve_runtime_widget_type` uses the finite `widget_type_for_tag` alias map, not arbitrary intrinsic declarations from the template registry. Preserve the failed harness, log, revision and receipt in [intrinsic discovery](../../../data/patch-api/evidence/7.1.0-session-2026-10-08/p710-intrinsic-discovery-context.json). The final passing clipping test uses ordinary inherited templates and does **not** claim custom intrinsic coverage. No special-case constructor was added. Existing known ContainedAlertFrame support remains separately tested.

## Retirement decision

**No runtime retirements.** `/usr/bin/grep -rnE` whole-word `\bTitleRegion\b` scans retain complete retail (excluding `*Documentation*`) and src/tests output: zero exact-name matches. Qualified/bare forms coincide for this unqualified object name. Whole-name caller scanning includes indirect `pcall(Name, ...)` and `and Name then` uses, not just direct calls.

Recorded master and unmerged p720-page revisions contain no TitleRegion re-additions. [Scan receipt](../../../data/patch-api/evidence/7.1.0-session-2026-10-08/p710-retirement-scans.json) pins complete register sets; the p720 register is present in its Git snapshot. Related GetTitleRegion/CreateTitleRegion scans find nine retail lines, all GetTitleRegion Lua-mixin consumers, and no src/tests lines; those names are not silently treated as retired. Unknown runtime CreateFrame kinds are rejected, but that alone does not prove the historical native TitleRegion region/method contract. No runtime surface is removed.

## Historical proof status

Evidence directory: [7.1.0-session-2026-10-08](../../../data/patch-api/evidence/7.1.0-session-2026-10-08/).

Initial publication discovery passes; custom-intrinsic experiment fails exactly as retained above. All 45 registers and 42 of 45 extracts reproduce. Inherited 12.0.5/12.0.7 mismatches and 12.1.0 unsupported-template error remain unchanged; no all-green extract claim. All 229 original source inputs and old extraction-mode outcomes remain protected. Parser/extractor/validator fixtures pass 28/34/8. All 21 prior validators pass.

Final targeted commands pass: **45 publication sweeps plus factory (46/46)** across **8,879 observations**; final own bounded scope **3/3**, screen mode **9/9**, C_Item **100/100**, known intrinsic **1/1**. One-row Frame:SetClipsChildren negative control changes exact publication gaps **0 → 1**, expected exit 1. Failed custom-intrinsic experiment and the original driver exit 1 remain retained, not relabeled green.

Python fixtures **28/34/8**, final `cargo fmt --check`, and requested Mists tests-check pass. Mists has **zero non-vendor warnings**; six inherited iced manifest deprecations remain unsuppressed. All **96 old extraction-mode outcomes**, **229 inputs**, **45 registers**, and the exact three inherited extract failures are preserved. Own read-only validator passes; combined prior/own status **22/22**. [Command ledger](../../../data/patch-api/evidence/7.1.0-session-2026-10-08/p710-command-ledger.md) retains exact commands, code revisions, exits and log hashes. Publication/regression scopes are unchanged by the final test-only clipping correction; final bounded and format receipts separately cover it. Mists excludes the retail-only corrected probe.

Historical register/sweep scope is fixed at `7778521e8`, not current globs or receipt-derived subsets. Counts come from retained files. No cwd/target equality gate. [Portability proof](../../../data/patch-api/evidence/7.1.0-session-2026-10-08/p710-validator-portability.json) passes in an independent shared clone, still passes with an extra audit register/sweep, rejects protected-input whitespace tampering, and passes after restoration. All 22 validators pass there without modifying evidence. Wiki preservation is checked against sealed revision `df8f26875`, so later legitimate wiki edits do not rewrite historical proof. The [final wiki-scope proof](../../../data/patch-api/evidence/7.1.0-session-2026-10-08/p710-validator-wiki-scope-portability.json) also passes after changing an old wiki entry alongside the expanded page-file set; protected-source tampering still fails. Original wiki index/log text is preserved intact, growing 2655 → 2659 and 372 → 376 lines.

No full integration suite, CASC texture tests, agents/model CLIs, other-worktree edits, push or merge. No __pycache__ retained. No `src/` changes against master: no widely-used runtime callers or registrations changed, so a separate addons-enabled master/branch startup comparison was not required or claimed.

## Integration boundary

Rebased onto merged 7.2.0 master `aa57dd8f830ec9c53e2f9c9dac5c7a69b5b042ec`. The real 7.2.0 register now appears first in `later_registers`; the placeholder is gone. Historical validator register/sweep scope remains pinned at `7778521e8`; integrated runtime/register/sweep scope is separately pinned at `50c69faa3938e23b3e339d47c0ac2f7f0c7bdc6d`.

[Integrated source reproduction](../../../data/patch-api/evidence/7.1.0-session-2026-10-08/integrated/source-reproduction-summary.json) verifies **46/46 registers** and **43/46 extracts**, retaining exactly the three historical extract failures. Own publication remains **zero gaps**; all ten observations are byte-equivalent to the historical result. The shared 7.2.0 probe-factory change does **not** resolve arbitrary intrinsic tags: the [archived custom-intrinsic experiment](../../../data/patch-api/evidence/7.1.0-session-2026-10-08/integrated/intrinsic-discovery-context.json) still fails exactly with `unknown frame type 'P710ClipIntrinsic'`. No supersession closure or runtime fix is credited. Exact-master all-sweep comparison and remaining integrated checks are pending.

## Sources

- [Pinned wikitext and provenance](../../../data/patch-api/sources/7.1.0-api-changes.provenance.json).
- [Occurrence ledger](../../../data/patch-api/sources/7.1.0-page-coverage.json).
- [Gap review](../../../data/patch-api/evidence/7.1.0-session-2026-10-08/p710-gap-review.json).
- [Publication spec](../../specs/patch-7-1-0-publication-sweep.md).
- [Binding procedure](../../../data/patch-api/evidence/handoff-laptop-to-agent-server/HANDOFF.md#per-page-integration-procedure-main-thread).

## See Also

- [[patch-7-2-5-api-audit]] — later-page template and integration procedure.
- [[patch-audit-validator-portability]] — historical, checkout-independent proof scope.
