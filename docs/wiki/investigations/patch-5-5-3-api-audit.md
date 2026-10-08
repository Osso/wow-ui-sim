# Patch 5.5.3 API audit

Verified source: 2026-10-08. Pageid `686957`, revision `6778082` (2026-07-22T05:40:25Z), pinned by commit `27c1712b2`. This Mists Classic resources-only stub lists no API identities. [Source](../../../data/patch-api/sources/5.5.3-api-changes.wikitext), [provenance](../../../data/patch-api/sources/5.5.3-api-changes.provenance.json).

## Profile and register-chain decision

TOC 50503, an ancestor build of the 50504 profile; same client line. The explicit main-thread resolution authorizes `client_line: mists-classic`, audited against current `client-mists`. This matches historical retail pages audited against current retail. The original [blocker record](../../../data/patch-api/evidence/5.5.3-session-2026-10-08/p553-profile-blocker.json) retains its stopped-state history and adds the resolution rather than erasing it.

Only 5.5.4 belongs in this page's `later_registers`. No Classic register is added to any retail chain. Neither runtime interface constants nor vendor/cache contents change.

## Harness and exact accounting

The [Mists test](../../../tests/patch_5_5_3_publication_sweep.rs) follows the [5.5.4 harness decision](patch-5-5-4-api-audit.md): current Mists profile/interface/cache assertions, real cached SharedXMLBase and SharedXML loaded without Lua errors, then shared publication classification. It does not use retail-only `prefork_full_ui`.

| Scope | Accounting | Proof boundary |
|---|---|---|
| Inventory entries / header counts | 0 / 0 | No positive API coverage |
| Extract metadata | Four IDs: navigation, Resources heading, TOC, diff pointers | Metadata-only; no behavior credit |
| Gaps / retirements | 0 / 0 | Source supplies no contracts or removals |
| Linked comparisons | Unexpanded | External boundary, not silently covered |
| Publisher loading | Mists SharedXMLBase and SharedXML | Not full-Game startup or native parity |

The register uses `--client-line mists-classic`; extraction uses existing opt-in `--canonical-patch-navigation`. No tool behavior changes. [Coverage ledger](../../../data/patch-api/sources/5.5.3-page-coverage.json) accounts for every extract row.

## Verification

Pending committed proof and portability gate. Evidence will retain revision-scoped commands, complete logs, source reproduction and branch/master observation comparison. No positive API or full-Game startup claim follows from passing an empty inventory.

## Sources

- [Evidence](../../../data/patch-api/evidence/5.5.3-session-2026-10-08/).
- [Spec](../../specs/patch-5-5-3-publication-sweep.md).
- [Profile code](../../../src/client_profile.rs).
- [Portability gate](patch-audit-validator-portability.md).

## See Also

- [[patch-5-5-4-api-audit]] — same-line successor and harness boundary.
- [[client-profiles]] — profile-scoped caches.
