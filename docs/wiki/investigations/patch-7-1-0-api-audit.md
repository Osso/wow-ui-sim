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

## Proof status

Evidence directory: [7.1.0-session-2026-10-08](../../../data/patch-api/evidence/7.1.0-session-2026-10-08/).

Initial publication discovery passes; custom-intrinsic experiment fails exactly as retained above. All 45 registers and 42 of 45 extracts reproduce. Inherited 12.0.5/12.0.7 mismatches and 12.1.0 unsupported-template error remain unchanged; no all-green extract claim. All 229 original source inputs and old extraction-mode outcomes remain protected. Parser/extractor/validator fixtures pass 28/34/8. All 21 prior validators pass.

Final targeted receipts and own portable-validator seal are pending while scoped Cargo gates run. No full integration suite, CASC texture tests, agents/model CLIs, other-worktree edits, push or merge. No __pycache__ retained.

## Integration boundary

7.2.0 remains unmerged. One-line placeholder appears first in `later_registers`, before 7.2.5 and every merged later register. Main-thread integration must replace it with the real 7.2.0 register and refresh only attributable supersessions/proof. Historical validator register/sweep sets remain scoped to a recorded revision; future pages do not expand old proof scope.

## Sources

- [Pinned wikitext and provenance](../../../data/patch-api/sources/7.1.0-api-changes.provenance.json).
- [Occurrence ledger](../../../data/patch-api/sources/7.1.0-page-coverage.json).
- [Gap review](../../../data/patch-api/evidence/7.1.0-session-2026-10-08/p710-gap-review.json).
- [Publication spec](../../specs/patch-7-1-0-publication-sweep.md).
- [Binding procedure](../../../data/patch-api/evidence/handoff-laptop-to-agent-server/HANDOFF.md#per-page-integration-procedure-main-thread).

## See Also

- [[patch-7-2-5-api-audit]] — later-page template and integration procedure.
- [[patch-audit-validator-portability]] — historical, checkout-independent proof scope.
