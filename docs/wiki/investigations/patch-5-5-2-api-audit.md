# Patch 5.5.2 API audit

Verified source: 2026-10-08. Warcraft Wiki pageid `686956`, revision `6778080` (2026-07-22T05:39:42Z), refetched through MediaWiki and pinned in `39bb21f41`. This is a Mists of Pandaria Classic resources-only stub. [Wikitext](../../../data/patch-api/sources/5.5.2-api-changes.wikitext), [provenance](../../../data/patch-api/sources/5.5.2-api-changes.provenance.json).

## Profile and register-chain decision

The page states `TOC: 50502`, an ancestor build in the `505xx` Mists Classic line. Audit against current `client-mists` interface 50504 and the `mists/AddOns` cache, following [[patch-5-5-4-api-audit]] and [[patch-5-5-3-api-audit]]. The register uses `--client-line mists-classic`; extraction uses `--canonical-patch-navigation`. No tool behavior changes.

Only later Classic registers 5.5.3 and 5.5.4 belong in this page's `later_registers`. Neither Classic register is added to a retail chain. Retail MoP-era 5.0–5.4 history is distinct from 2025–2026 Mists Classic.

## Harness and exact accounting

[Discovery test](../../../tests/patch_5_5_2_publication_sweep.rs) asserts the Mists profile/interface/cache, loads real cached SharedXMLBase and SharedXML without Lua errors, then runs the shared classifier. It does not use retail-only `prefork_full_ui`. No real API identities require additional publishers in this revision.

| Scope | Accounting | Proof boundary |
|---|---|---|
| Inventory / header counts | 0 / 0 | Empty-inventory proof only; no positive API coverage |
| Extract metadata | Four IDs: navigation, Resources heading, TOC, diff pointers | All metadata-only; no behavior credit |
| Gaps / retirements | 0 / 0 | No source-listed contracts or removals |
| Linked comparisons | Unexpanded | External boundaries, not silently covered |
| Publisher loading | Mists SharedXMLBase and SharedXML | Not full-Game startup or native parity |

[Coverage ledger](../../../data/patch-api/sources/5.5.2-page-coverage.json) accounts for every extract row. No runtime, shim, vendor, Wowless or generated WowlessData changes. Both later Classic inventories are also empty; no listed re-additions. Whole-word Mists cache scans exclude Documentation; complete src/tests scans use `/usr/bin/grep` and retain untruncated output. These are page-context scans, not positive consumer-free retirement proofs for invented identities.

## Verification

Requested Rust checks run asynchronously with complete logs under the own [evidence directory](../../../data/patch-api/evidence/5.5.2-session-2026-10-08/). Acceptance and the portable validator gate remain pending; no passing Rust claim yet. Reproduction already regenerates all 59 registers byte-identically; 56/59 extracts reproduce. The inherited 12.0.5, 12.0.7 and 12.1.0 extract failures remain unchanged. Shared sources are copied from pinned Git revisions into reproduction scratch, including transclusions.

## Sources

- [Evidence and validator](../../../data/patch-api/evidence/5.5.2-session-2026-10-08/).
- [Spec](../../specs/patch-5-5-2-publication-sweep.md).
- [Per-page procedure](../../../data/patch-api/evidence/handoff-laptop-to-agent-server/HANDOFF.md).

## See Also

- [[patch-5-5-3-api-audit]] — same-line successor.
- [[patch-5-5-4-api-audit]] — harness and ancestor-TOC decisions.
- [[patch-audit-validator-portability]] — Git-pinned shared inputs and own-session seals.
- [[client-profiles]] — separate client histories and source caches.
