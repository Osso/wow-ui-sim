# Patch 5.5.4 API audit

Verified source: 2026-10-08. Pageid `686958`, revision `6778083` (2026-07-22T05:41:22Z), refetched through MediaWiki. This is a **Mists of Pandaria Classic** resources-only stub, not a 2013 retail API inventory. Source: [pinned wikitext](../../../data/patch-api/sources/5.5.4-api-changes.wikitext), [provenance](../../../data/patch-api/sources/5.5.4-api-changes.provenance.json).

## Profile and register-chain decision

The page states `TOC: 50504`; [client profile code](../../../src/client_profile.rs) assigns `ACTIVE_INTERFACE_VERSION = 50504` to `client-mists`, whose source cache is `mists/AddOns`. Therefore expected publication is the Mists Classic surface. Retail's MoP-era 5.0–5.4 pages and the 2025–2026 Classic 5.5.x pages are different client histories. Version sorting cannot establish supersession between them.

5.5.4 does **not** belong in any older retail page's `later_registers`. Merged retail 6.2.x, 6.1.0, 6.0.2, 6.0.1 and 5.4.8 registers remain outside this client line; none belongs in a Mists supersession chain. Subsequent 5.5.3–5.5.0 audits should use `client_line: mists-classic` and Mists-profile tests; Classic Era 1.13–1.15 pages should use `client_line: classic-era` and the source-proven Era/Anniversary profile, not numerical inference. Ancestor-build TOCs in the `505xx` family remain the same Mists Classic client line: TOC `50503` is audited against the current `50504` profile, just as historical retail pages use current retail. See [[patch-5-5-3-api-audit]]. Historical unlabeled registers retain `retail` meaning. An explicitly different client line is excluded from supersession, even if accidentally supplied among later registers. A sweep must also reject an incompatible execution profile.

## Harness decision

[Prefork target declaration](../../../Cargo.toml) requires `client-retail`. The retained [Mists probe](../../../data/patch-api/evidence/5.5.4-session-2026-10-08/p554-mists-prefork-probe.log) fails before compilation with that feature requirement. Enabling retail alongside Mists would violate the mutually-exclusive profile contract. Do not weaken the prefork executable's retail conformance assertions merely for this page.

[Shared classifier](../../../tests/common/publication_sweep.rs) now parses a default-retail `ClientLine` enum, excludes mismatched later registers, and checks the executing profile. Three retail controls reproduced erroneous cross-line supersession and absent profile rejection before the fix.

The first integration attempt reused the [profile-aware prefork preload](../../../tests/common/prefork_full_ui_preload.rs). It failed loading `Blizzard_UIPanels_Game`: `Action[SetPoint] failed because[Cannot anchor to itself]` on generated frame `__Blizzard_UIPanels_Game_20250`. [Retained failed load](../../../data/patch-api/evidence/5.5.4-session-2026-10-08/p554-mists-discovery.log) proves this test-loader boundary, not a diagnosed native-game or production-startup bug. Runtime sources were unchanged; no adjacent layout fix or vendor patch was attempted.

The final [Mists integration case](../../../tests/patch_5_5_4_publication_sweep.rs) uses `common::env_with_shared_xml()` to load Mists `Blizzard_SharedXMLBase` and `Blizzard_SharedXML`, verifies both loaded without Lua errors, and asserts the active profile, interface `50504` and Mists cache selection before the shared classifier. This is sufficient for this page’s empty inventory, **not** full-Game/FrameXML startup proof. For 5.5.3–5.5.0 or Era pages with actual Lua-defined identities, load each identity’s real cached publisher; do not count absent publisher files or reuse this minimal loader as coverage by default. Publication is not native signature/output/security parity. An empty inventory cannot establish positive API coverage.

## Exact source accounting

- Inventory: zero entries, zero header counts, zero gaps.
- Extract: four metadata-only IDs (navigation, Resources heading, TOC, external diff pointers); no modeled behavior credit.
- Modeled gaps: none. Problematic API contracts: none stated by this revision.
- Linked GitHub comparisons are unexpanded external boundaries, not undiscovered identities silently counted as covered.
- Retirements: none; source lists no removed members. Whole-word Mists consumer and src/tests context scans are retained without truncation using `/usr/bin/grep`, excluding Documentation in the cache.

The opt-in extractor flag renders the positional `5.5.4` navigation value rather than mistaking the preceding `prev=5.5.3` parameter for the current patch. Defaults remain unchanged so prior extracts reproduce byte-for-byte.

## Integrated verification

Rebased onto master `bebcc5830`. [Integrated receipts](../../../data/patch-api/evidence/5.5.4-session-2026-10-08/integrated/) preserve 188 historical artifacts and map all nine original commits with patch IDs and original/rebased blob hashes. The historical validator replays every original invariant from those preserved inputs, including both validator self-seals against the preserved original bytes rather than the new wrapper. Both own validators pass in a fresh single-branch clone where all nine original commits are absent; no unreachable pre-rebase objects or live shared files are required.

Retail branch/master sweeps pass 56/56 each; [comparison](../../../data/patch-api/evidence/5.5.4-session-2026-10-08/integrated/gap-comparison.json) proves all 9,740 observations on all 55 retail pages exactly identical. Unlabeled retail registers remain byte-identical. Mists SharedXML/page/line tests pass 2/2; retail client-line controls pass 3/3. Mists `cargo check --tests` has zero non-vendor warnings; `cargo fmt --check` and all 87 Python fixtures pass. All 56 registers and 53/56 extracts reproduce with recorded flags; the three inherited extract errors are exactly unchanged. The rerun negative control rejects one injected row against the empty inventory (1 → 0). All 38 prior validators selected with `git ls-tree` at the pinned master pass. Empty inventory and SharedXML-only loading still confer no positive API or full-Game startup coverage. [Portability summary](../../../data/patch-api/evidence/5.5.4-session-2026-10-08/integrated/validator-gate-report.json): clean 40/40 and later-audit 41/41 PASS; integrated and historical own-log tampering are both rejected before exact restoration.

## Historical verification

Final Rust scope `9ca9cd746`: 52 retail sweep/factory cases pass; all 9,051 observations across 51 pinned retail pages are exactly unchanged. Mists SharedXML/page/line cases pass 2/2; retail client-line controls pass 3/3 after their three-case RED. Mists `cargo check --tests` has zero non-vendor warnings; all 82 Python fixtures and Cargo/explicit generated-module formatting checks pass. All 52 registers and 49/52 extracts reproduce; inherited 12.0.5, 12.0.7 and 12.1.0 extract failures remain exact source boundaries. The negative register adds one row to the empty inventory and correctly fails 1 → 0 row-count enforcement.

The evidence launcher originally saved the Mists line-control result under the page filename. Its first-match scanner was corrected to capture every output environment separately; [final page results](../../../data/patch-api/evidence/5.5.4-session-2026-10-08/p554-mists-evidence/patch_5_5_4_publication_sweep-results.json) are `{}`, while control observations retain their distinct file. No runtime sweep was changed to conceal that evidence bug.

The portable historical validator is committed with Git-pinned shared inputs and own-session seals. [Gate summary](../../../data/patch-api/evidence/5.5.4-session-2026-10-08/p554-gate-summary.json): clean: 32 passed / 0 failed, later_audit: 33 passed / 0 failed. The additional later-audit validator is the synthetic 9.9.9 case. Own sealed-source tampering fails before exact restoration. No full integration suite, native game probe, or runtime/API modifications are claimed.

## Sources

- [Evidence directory](../../../data/patch-api/evidence/5.5.4-session-2026-10-08/).
- [Coverage ledger](../../../data/patch-api/sources/5.5.4-page-coverage.json).
- [Spec](../../specs/patch-5-5-4-publication-sweep.md).
- [Handoff procedure](../../../data/patch-api/evidence/handoff-laptop-to-agent-server/HANDOFF.md).

## See Also

- [[client-profiles]] — distinct Classic runtime caches and interface versions.
- [[patch-audit-validator-portability]] — pinned historical proof, not live shared-file comparisons.
- [[patch-6-2-0-api-audit]] — retail evidence structure, not a supersession input to Mists.
