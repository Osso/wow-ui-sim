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

Retail sweep/factory cases pass 56/56; every one of 9,740 observations on 55 retail pages equals the Git-pinned master baseline. The baseline runtime's src/tests/tools/Cargo/build scope is identical to master `4d046d99d`, so no redundant baseline rebuild is needed. Retail client-line controls pass 3/3; the 5.5.3 page and both 5.5.4 Mists cases pass 3/3. Both page outputs are `{}`. All 87 Python fixtures and final formatting pass.

All 57 registers and 54/57 extracts reproduce with recorded flags. The inherited 12.0.5, 12.0.7 and 12.1.0 extract failures remain exactly unchanged. Reproduction copies every wikitext input, including the 6.0.2 transclusion, from Git into scratch before invoking either tool there. No live shared-source input is required. Whole-word Mists cache scans exclude Documentation; complete src/tests scans use `/usr/bin/grep` and retain untruncated output. No removed identities or same-line re-additions are listed.

Mists `cargo check --tests` passes with zero non-vendor warnings. The injected-row negative control exits 101 at the expected 1 → 0 row-count boundary. At `1882e7c99`, `tools/check_patch_validators.py` passes 41/41 clean validators and 42/42 after the synthetic unrelated later audit. [Gate report](../../../data/patch-api/evidence/5.5.3-session-2026-10-08/gate-report.json) records the exact revision. Own sealed-log tampering is rejected before byte-identical restoration. No ignored or uncommitted acceptance inputs are needed; historical register/sweep sets and shared bytes are checked at recorded Git revisions, not against moving live files. [Proof ledger](../../../data/patch-api/evidence/5.5.3-session-2026-10-08/proof-ledger.md) records command/revision scopes and the targeted refresh after deleting an unused Mists-only import. No positive API or full-Game startup claim follows from passing an empty inventory.

## Sources

- [Evidence](../../../data/patch-api/evidence/5.5.3-session-2026-10-08/).
- [Spec](../../specs/patch-5-5-3-publication-sweep.md).
- [Profile code](../../../src/client_profile.rs).
- [Portability gate](patch-audit-validator-portability.md).

## See Also

- [[patch-5-5-4-api-audit]] — same-line successor and harness boundary.
- [[client-profiles]] — profile-scoped caches.
