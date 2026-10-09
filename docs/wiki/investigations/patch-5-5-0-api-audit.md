# Patch 5.5.0 API audit

Source refetched for the 2026-10-08 audit: Warcraft Wiki pageid `628475`, current revision `6303391`, timestamp `2025-04-25T00:47:08Z`. This is the Mists of Pandaria Classic launch page, following Cataclysm Classic 4.4.2, not retail MoP. [Pinned response](../../../data/patch-api/evidence/5.5.0-session-2026-10-08/p550-source-response.json), [wikitext](../../../data/patch-api/sources/5.5.0-api-changes.wikitext) and [provenance](../../../data/patch-api/sources/5.5.0-api-changes.provenance.json).

## Profile and client history

The page states TOC `50500`: an ancestor in the `505xx` Mists Classic line, audited against current `client-mists` interface `50504` and profile-selected `mists/AddOns` cache. Generate with `--client-line mists-classic`; retain navigation with `--canonical-patch-navigation`. Only Classic 5.5.1–5.5.4 registers belong in this page's `later_registers`. No Classic register was added to a retail page. The summary's retail 11.1.7 link is not a supersession-chain input.

## Exact coverage

| Source scope | Accounting | Proof level |
|---|---|---|
| Identity inventory / header counts | 0 / 0 | Empty-inventory classification, no positive API coverage |
| Extract metadata | 5 IDs | Navigation, two headings, TOC and diff links; no runtime credit |
| Summary `prose-undated-004` | 1 problematic gap | Broad synchronization through Mainline 11.1.7 names no identities, signatures, outputs or security transitions |
| Modeled additions / retirements | 0 / 0 | No source-listed API contract or removal |
| Linked retail page and GitHub comparisons | Unexpanded boundaries | Not silently promoted into verified API inventories |

[Coverage ledger](../../../data/patch-api/sources/5.5.0-page-coverage.json) accounts for all six extract IDs. Unlike 5.5.1–5.5.4, this page contains a summary assertion, but it still contains no real identity inventory. No runtime model change is justified by that unbounded assertion. The identity-gap fixture remains empty; this does not close the problematic prose row. Whole-word `/usr/bin/grep` context scans retain complete Mists-cache output (excluding Documentation) and src/tests output with command/hash/count receipts. No absence-based retirement claim is made: there are no candidates to scan or remove, and successors list no re-additions.

## Loader decision and guidance for Classic Era

[Discovery test](../../../tests/patch_5_5_0_publication_sweep.rs) asserts the Mists profile, current interface, cache selection, and successful real cached `Blizzard_SharedXMLBase`/`Blizzard_SharedXML` loading without Lua errors before the classifier. This establishes the execution environment and empty inventory only. It does **not** establish full-Game startup, positive Lua-defined publication, the Mainline synchronization claim or native signature/output/security parity.

The retail-only `prefork_full_ui` target must not be used under Mists or weakened to accommodate Classic. [[patch-5-5-4-api-audit]] retains the full Mists prefork-style load failure at `Blizzard_UIPanels_Game`, SetPoint "Cannot anchor to itself". That is a loader boundary, not a reason to patch vendor Lua or invent positive coverage.

For 1.13–1.15 Classic Era pages with real identities, choose the source-proven Era/Anniversary profile and current matching cache, not numerical patch ordering. For each Lua-defined identity, locate and load its **real cached publisher** (SharedXML/FrameXML prerequisites plus the defining/consuming Blizzard addon as required); record publisher files, actual successful loading and exact identity observations. The minimal loader used here is not positive coverage for those pages. For Rust-provided `C_*` members, query the namespace in the matching profile directly; publication still does not prove behavioral parity. If a publisher cannot load, retain the exact failure and classify that identity as a loader-blocked gap. Never count a missing publisher file as covered. Do not monkey-patch cached sources or add compatibility shims as shortcuts.

## Verification

Pinned master `7a292c9d0fc7458c648eceb6cf67a036b2e22ac8` versus branch runtime/test revision `c9e249647`: retail sweeps/factory pass 58/58 each; all 9,817 observations across 57 retail pages are exactly identical. Mists 5.5.0–5.5.4 page/line cases pass 6/6, retail client-line controls 3/3. Mists `cargo check --tests` passes with zero non-vendor warnings (seven unchanged vendor-manifest warning lines). All 89 Python fixtures and formatting pass. All 62 registers reproduce byte-identically; 59/62 extracts reproduce and inherited 12.0.5/12.0.7/12.1.0 failures are exactly unchanged. Injected row fails at the expected count boundary, 1 → 0, exit 101. Own validator passes with 153 sealed session inputs. Clean/later-audit gate pending the evidence commit; retained receipts and pinned shared inputs already pass own validation. No widely shared source path changed, so the conditional full integration/lib/addons-enabled startup regression requirement is not triggered.

## Sources

- [Evidence and validator](../../../data/patch-api/evidence/5.5.0-session-2026-10-08/).
- [Spec](../../specs/patch-5-5-0-publication-sweep.md).
- [Per-page integration procedure](../../../data/patch-api/evidence/handoff-laptop-to-agent-server/HANDOFF.md).

## See Also

- [[patch-5-5-1-api-audit]] — immediate Classic successor and portable evidence pattern.
- [[patch-5-5-4-api-audit]] — client-line, ancestor TOC and full-load boundary.
- [[patch-audit-validator-portability]] — committed own evidence and Git-pinned shared inputs.
- [[client-profiles]] — distinct Classic runtime caches and histories.
