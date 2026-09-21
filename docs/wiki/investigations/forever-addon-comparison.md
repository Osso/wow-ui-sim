# Forever addon comparison audit

A complete public CurseForge catalog capture provides the comparison corpus; it does not prove addon compatibility. The first evidence-backed correction is Forever-only `Enum.BagIndex` publication, motivated by BetterBags’ contiguous bank-tab enumeration and confirmed by the pinned 1.60.1.69913 API documentation.

## Catalog coverage

Commit `7eb74d91e` records the public CurseForge `1.60.1` / Forever catalog in [the comparison audit](../../forever-addon-comparison.md): 875 unique projects across all 44 rendered A–Z pages, with no duplicate or missing rows. Every captured page reported 875 projects and the ending page-one check retained that count and ordering. The live catalog was previously observed at 871, so this capture is a dated 875-project observation rather than a stable total.

File pairing, package/source diffing, consumer reproduction, and broader compatibility remain in progress. A Forever tag is a declaration, not an API contract or a passing simulator result.

## BagIndex correction

BetterBags commit `411a6f6ee1ea40eca8ac96927ccdd49a6aab3941` walks consecutive `Enum.BagIndex.CharacterBankTab_N` and `AccountBankTab_N` members. Pinned `BagIndexConstantsDocumentation.lua` confirms Forever’s character IDs `6..14`, account IDs `15..23`, and `BagIndexMeta {-3, 23, 27}`; shared publication instead left account IDs at Retail’s earlier positions.

Commit `bb83a4c0a` publishes the corrected values and metadata only under `client-wowforever`. Its two exact consumer-loop enum-shape tests were RED before the producer change and GREEN after it. This proves enum names, values, boundaries, disjointness, and metadata only. It does not load BetterBags, model bank state, prove purchased tabs or account-bank availability, add Warbank behavior, or establish native conformance.

## Sources

- [Forever comparison audit](../../forever-addon-comparison.md) — catalog provenance, scope, and incomplete comparison matrix
- [Forever finite constants spec](../../specs/forever-finite-constants.md) — BagIndex contract and test boundary
- [Forever running report](../../wowforever-1.60.1.md) — profile-wide committed behavior and proof boundaries
- `Blizzard_APIDocumentationGenerated/BagIndexConstantsDocumentation.lua` in the pinned Forever 1.60.1.69913 cache — authoritative enum values
- BetterBags `411a6f6ee1ea40eca8ac96927ccdd49a6aab3941` — motivating consumer loop
- `/tmp/forever-bag-index-development-ledger.json` — RED/GREEN command and revision evidence

## See Also

- [[forever-clean-startup]] — distinct sustained Blizzard-runtime proof
- [[client-profiles]] — Camelot/Forever profile selection
