# Cataclysm Classic 4.4.2 source audit

Source accounting only, verified source identity 2026-10-08. Pinned Warcraft Wiki pageid `619773`, revision `6303388`, timestamp `2025-04-25T00:45:23Z`; source commit `4c6c87e8ffe9c201ad0b62c56571aa376e151671`. Literal TOC **40402**. This is Cataclysm Classic, not historical retail Cataclysm or Mists Classic. Source accounting completion does not resolve its API contract.

## Exact row/proof matrix

The [ledger](../../../data/patch-api/sources/4.4.2-page-coverage.json) accounts for every nonblank source line; blank lines 2 and 5 are retained in source/plaintext but create no capability rows.

| Source ID | Wikitext/text line | Content | Classification | Proof limit |
|---|---:|---|---|---|
| `source-context-001` | 1 | 4.4.2 navigation, prev=4.4.1, next=5.5.0 | metadata-only | Source reference; no supersession credit |
| `source-context-003` | 3 | Summary heading | metadata-only | Editorial only |
| `prose-undated-004` | 4 | Modern auction house and associated `C_AuctionHouse` APIs enabled in Cataclysm Classic | **UNPROVEN** | Unsupported Cata profile; no runtime/native proof |
| `source-context-006` | 6 | Resources heading | metadata-only | Editorial only |
| `source-context-007` | 7 | Literal TOC 40402 | metadata-only | Source build metadata, not a runtime probe |
| `source-context-008` | 8 | Two 4.4.1..4.4.2 GitHub comparisons | metadata-only | External diffs unexpanded |

Totals: **six rows, five metadata-only, one UNPROVEN namespace contract**. Zero enumerated API occurrences, inventory header counts or removal statements. No register created, no modeled gap count or positive publication/behavior credit. The namespace mention is substantive; zero member inventory does not make this a resources-only page.

## Source proof versus API proof

The sole substantive statement is retained literally: “The modern auction house and its associated C_AuctionHouse APIs has been enabled for Cataclysm Classic.” It names a namespace, not a member set. Individual methods, signatures, argument/return contracts, event payloads, security restrictions and auction transitions cannot be inferred from that sentence.

The committed MediaWiki response is 674 bytes, SHA-256 `c0604a21b721a67b5fde2d429b72e7f10f5e272758718c1343c94c9bb5839eb4`. Its returned main-slot content equals the saved 390-byte wikitext exactly, SHA-256 `484b49adb4301ad901914389cf37f9849c21d62eea20797d0ad2fefbaa2b8669`. Revision/page/title/timestamp are checked against actual response fields, not merely copied provenance. Plaintext is 226 bytes, SHA-256 `b93f153298af6230550f920084fee802bead9357929541505fa5538e2dcd7442`. No new network retrieval is claimed.

[Source proof validator](../../../data/patch-api/evidence/4.4.2-session-2026-10-08/validate.py) checks these bytes and the exact serialized row contract; in-memory negative controls reject changed revision, source bytes, namespace coverage credit and omitted external-reference row. The existing extractor reproduces plaintext with `--patch 4.4.2 --text-only --canonical-patch-navigation --check`. [Acceptance receipt](../../../data/patch-api/evidence/4.4.2-session-2026-10-08/source-proof.json): all five source-proof tests and plaintext reproduction pass at `b5ee99e5b`. Generic extractor seeds its substantive row as `audit-pending`; the audited ledger explicitly classifies it **UNPROVEN**. Later docs/receipt changes leave the tested data/validator/extractor scope unchanged. These are source-accounting tests only, never behavioral compatibility substitutes.

## Runtime and native boundaries

[ClientProfile](../../../src/client_profile.rs) has Retail, Ptr, Wrath, Mists, Era, Anniversary and WowForever variants, **no Cataclysm Classic variant/profile/cache selection**. No Cata runtime loaded; no retail or Mists stand-in used. Namespace publication, cached Cata auction UI loading, signatures, outputs, event dispatch, security, auction operations and native parity all remain unproven. No native Cataclysm client observation was captured. Existing auction code under another profile cannot establish Cata compatibility. Cargo builds/runtime tests are unnecessary for this source-only change and are not claimed.

The [5.5.4 audit](patch-5-5-4-api-audit.md) separates Mists Classic 505xx from historical retail; version sorting and `next=5.5.0` do not establish compatible runtime supersession. No retail or Mists `later_registers` entry is added.

## Coordinator tooling note

[Register generator](../../../tools/gen_patch_wikitext_register.py) accepts only `retail`, `mists-classic`, `classic-era`; [shared classifier](../../../tests/common/publication_sweep.rs) models those three lines and defaults unlabeled registers to retail. A future Cata runtime audit would require explicit new client-line classification and a supported profile. Reported here for coordinator ownership, not implemented or redesigned. The ledger's `cataclysm-classic` label describes source identity only; its distinct `patch-source-accounting/v1` schema is not a runtime register. No unlabeled/default-retail register was generated.

## Sources

- [Pinned wikitext](../../../data/patch-api/sources/4.4.2-api-changes.wikitext), [returned response](../../../data/patch-api/evidence/4.4.2-session-2026-10-08/source-response.json), [provenance](../../../data/patch-api/evidence/4.4.2-session-2026-10-08/source-pin.json).
- [Source-only ledger](../../../data/patch-api/sources/4.4.2-page-coverage.json), [spec](../../specs/patch-4-4-2-source-accounting.md).
- [Handoff](../../../data/patch-api/evidence/handoff-laptop-to-agent-server/HANDOFF.md): prior audit context; its runtime/delegation/merge workflow is excluded by this task's explicit source-only scope.

## See Also

- [[patch-5-5-4-api-audit]] — profile/client-line isolation decisions.
- [[client-profiles]] — available runtime profiles, not Cata support.
