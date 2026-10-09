# Patch 5.5.1 API audit

Verified source: 2026-10-08. Warcraft Wiki pageid `686954`, revision `6778077` (2026-07-22T05:39:04Z), refetched through MediaWiki. Mists of Pandaria Classic resources-only stub, not retail MoP. [Pinned wikitext](../../../data/patch-api/sources/5.5.1-api-changes.wikitext), [provenance](../../../data/patch-api/sources/5.5.1-api-changes.provenance.json).

## Profile and chain

The page states `TOC: 50501`, an ancestor build in the same `505xx` client line as current `client-mists` interface 50504. Follow [[patch-5-5-4-api-audit]] and [[patch-5-5-2-api-audit]]: generate with `--client-line mists-classic`, extract with `--canonical-patch-navigation`, and execute against the Mists cache. Only later Classic registers 5.5.2, 5.5.3 and 5.5.4 appear in `later_registers`. Retail histories and registers remain untouched.

## Exact accounting and harness

| Scope | Accounting | Proof boundary |
|---|---|---|
| Inventory / header counts | 0 / 0 | Empty-inventory proof; no positive API coverage |
| Extract | Four metadata IDs: navigation, Resources heading, TOC, diff links | No behavior credit |
| Gaps / retirements | 0 / 0 | No source-listed contracts or removals |
| Linked GitHub comparisons | Unexpanded | External boundaries, not silently covered identities |
| Cached publisher loading | Mists SharedXMLBase and SharedXML | Not full-Game startup or native parity |

[Discovery test](../../../tests/patch_5_5_1_publication_sweep.rs) follows the merged 5.5.2 harness, asserting Mists profile/interface/cache and real SharedXML loading without Lua errors before classification. No real API identities require additional publishers. `prefork_full_ui` remains retail-only. No runtime, shim, vendor, Wowless or WowlessData changes.

[Coverage ledger](../../../data/patch-api/sources/5.5.1-page-coverage.json) accounts for every extract row. Same-line successor inventories are also empty. Whole-word cache scans exclude Documentation; complete src/tests context scans use `/usr/bin/grep` with untruncated output and receipts. No invented retirement candidates or positive consumer-free retirement claims.

## Verification

Runtime proof is in progress. All 60 registers reproduce byte-identically; 57/60 extracts reproduce. Inherited failures for 12.0.5, 12.0.7 and 12.1.0 remain identical to pinned prior evidence. All 87 Python fixtures and Cargo formatting pass. Branch/master sweeps, Mists 5.5.1–5.5.4, Mists warning check, negative control, portable gate and tamper control remain pending.

Baseline is pinned master `0469e5592`. Shared-file validation uses recorded Git revisions; mutable comparisons stay inside this audit's [session directory](../../../data/patch-api/evidence/5.5.1-session-2026-10-08/). Prior-validator selection comes from `git ls-tree`, not a live global count. Wiki index/log must preserve their initial 2,743/496 lines and preceding-commit byte counts before every commit.

## Sources

- [Evidence and validator](../../../data/patch-api/evidence/5.5.1-session-2026-10-08/).
- [Spec](../../specs/patch-5-5-1-publication-sweep.md).
- [Per-page integration procedure](../../../data/patch-api/evidence/handoff-laptop-to-agent-server/HANDOFF.md).

## See Also

- [[patch-5-5-2-api-audit]] — direct Classic successor and harness pattern.
- [[patch-5-5-4-api-audit]] — profile, chain and ancestor-TOC decisions.
- [[patch-audit-validator-portability]] — committed inputs and Git-pinned shared proof.
- [[client-profiles]] — distinct Classic histories and caches.
