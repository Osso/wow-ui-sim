# Patch 5.5.4 API audit

Verified source: 2026-10-08. Pageid `686958`, revision `6778083` (2026-07-22T05:41:22Z), refetched through MediaWiki. This is a **Mists of Pandaria Classic** resources-only stub, not a 2013 retail API inventory. Source: [pinned wikitext](../../../data/patch-api/sources/5.5.4-api-changes.wikitext), [provenance](../../../data/patch-api/sources/5.5.4-api-changes.provenance.json).

## Profile and register-chain decision

The page states `TOC: 50504`; [client profile code](../../../src/client_profile.rs) assigns `ACTIVE_INTERFACE_VERSION = 50504` to `client-mists`, whose source cache is `mists/AddOns`. Therefore expected publication is the Mists Classic surface. Retail's MoP-era 5.0–5.4 pages and the 2025–2026 Classic 5.5.x pages are different client histories. Version sorting cannot establish supersession between them.

5.5.4 does **not** belong in any older retail page's `later_registers`. No placeholders for queued 6.1.0, 6.0.2, or 6.0.1 are needed here. This audit does not touch their worktrees or results. Subsequent 5.5.3–5.5.0 audits should use `client_line: mists-classic` and Mists-profile tests; Classic Era 1.13–1.15 pages should use `client_line: classic-era` and the source-proven Era/Anniversary profile, not numerical inference. Historical unlabeled registers retain `retail` meaning. An explicitly different client line is excluded from supersession, even if accidentally supplied among later registers. A sweep must also reject an incompatible execution profile.

## Harness decision

[Prefork target declaration](../../../Cargo.toml) requires `client-retail`. The retained [Mists probe](../../../data/patch-api/evidence/5.5.4-session-2026-10-08/p554-mists-prefork-probe.log) fails before compilation with that feature requirement. Enabling retail alongside Mists would violate the mutually-exclusive profile contract. Do not weaken the prefork executable's retail conformance assertions merely for this page.

The narrow alternative is a Mists integration case using the existing [profile-aware full-UI preload](../../../tests/common/prefork_full_ui_preload.rs), with assertions for active profile, interface `50504`, Mists cache selection and loaded UI, followed by the shared register-driven classifier. Publication is not native signature/output/security parity. An empty inventory cannot establish positive API coverage.

## Exact source accounting

- Inventory: zero entries, zero header counts, zero gaps.
- Extract: four metadata-only IDs (navigation, Resources heading, TOC, external diff pointers); no modeled behavior credit.
- Modeled gaps: none. Problematic API contracts: none stated by this revision.
- Linked GitHub comparisons are unexpanded external boundaries, not undiscovered identities silently counted as covered.
- Retirements: none; source lists no removed members. Whole-word Mists consumer and src/tests context scans are retained without truncation using `/usr/bin/grep`, excluding Documentation in the cache.

The opt-in extractor flag renders the positional `5.5.4` navigation value rather than mistaking the preceding `prev=5.5.3` parameter for the current patch. Defaults remain unchanged so prior extracts reproduce byte-for-byte.

## Verification

Targeted verification is pending. Required acceptance: all retail publication observations unchanged, Mists discovery and client-line tests, warning-clean non-vendor Mists `cargo check --tests`, every `tools/test_*.py`, historical source reproduction, formatting, and clean/later-audit portability gate. No full integration suite, native game probe, or runtime/API modifications are claimed.

## Sources

- [Evidence directory](../../../data/patch-api/evidence/5.5.4-session-2026-10-08/).
- [Coverage ledger](../../../data/patch-api/sources/5.5.4-page-coverage.json).
- [Spec](../../specs/patch-5-5-4-publication-sweep.md).
- [Handoff procedure](../../../data/patch-api/evidence/handoff-laptop-to-agent-server/HANDOFF.md).

## See Also

- [[client-profiles]] — distinct Classic runtime caches and interface versions.
- [[patch-audit-validator-portability]] — pinned historical proof, not live shared-file comparisons.
- [[patch-6-2-0-api-audit]] — retail evidence structure, not a supersession input to Mists.
