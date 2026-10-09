# Patch 3.4.3 Wrath Classic API audit

Bounded source/profile accounting, verified 2026-10-09 against base `23c930d837530e197d5f728e07336f81847e0710`. Frozen Warcraft Wiki page `152751`, revision `5983024`, timestamp `2024-03-07T08:49:48Z`, literal TOC **30403** identifies Wrath Classic—not retail Wrath in 2009. No new network retrieval or linked-page expansion.

## Exact coverage matrix

The [ledger](../../../data/patch-api/sources/3.4.3-page-coverage.json) preserves every nonblank wikitext/plaintext line; blank lines 2 and 6 create no capability rows.

| Source ID | Line | Statement | Proof |
|---|---:|---|---|
| source-context-001 | 1 | Navigation: prev=3.4.2, next=4.4.0 | Metadata only; no supersession |
| source-context-003 | 3 | Summary heading | Metadata only |
| prose-undated-004 | 4 | Many API changes pulled in from retail 10.1.7 | **UNPROVEN**: unspecified subset; linked page unexpanded |
| prose-undated-005 | 5 | Collection APIs added for mounts, pets, toys, heirlooms | **UNPROVEN**: no named APIs or native behavioral contracts |
| source-context-007 | 7 | Resources heading | Metadata only |
| source-context-008 | 8 | TOC 30403 | Source identity, not measured runtime build |
| source-context-009 | 9 | Two external diffs | Unexpanded resources; classic_ptr link is moving, not a pinned inventory |

**Seven source rows: five metadata-only, two UNPROVEN prose limits. Zero explicit API occurrences, inventory headers, or removals.** No register, positive publication credit, modeled gap count, retirements or native parity. Zero inventory establishes neither API completeness nor behavior. No cheap meaningful model can be derived: neither bullet specifies an input/output, member identity, state transition, event payload or security rule. Collection publication would still not establish collection behavior.

## Profile and cache evidence

Unlike [4.4.x Cataclysm source audits](patch-4-4-2-api-audit.md), Wrath is an existing supported profile. [ClientProfile](../../../src/client_profile.rs) selects `client-wrath`, `Wrath`, `wrath/AddOns` and configured interface **38001**. Source TOC **30403** and configured **38001** are recorded separately, not treated as identical builds or compatibility proof.

The [historical observation](../../../data/patch-api/evidence/3.4.3-session-2026-10-09/profile-observation.json) records this host's cache file paths/hashes: 42 files under only `Blizzard_APIDocumentation` and `Blizzard_APIDocumentationGenerated`; no SharedXML or collection publisher files. The committed Wrath manifest exists and is hash-referenced separately. No sync, vendor edit, runtime load, compilation, startup or native probe was attempted. This is an incomplete local publisher-cache boundary, **not absent Wrath support** and not a diagnosed runtime failure. The supported profile plus documentation-only cache cannot support the [5.5.4 SharedXML publication harness](patch-5-5-4-api-audit.md); no retail/Mists stand-in is used.

## Separate client history and tooling

The ledger labels `wrath-classic`/`wrath` explicitly. Retail 10.1.7 is a summary reference, not a complete Wrath member set or successor. Navigation to Cata 4.4.0 describes the client transition, not a compatible Wrath publication supersession. Mists 5.5.x is also separate. `later_registers` is empty: no retail, Cata or Mists successor overrides the two Wrath limits.

[Register generator](../../../tools/gen_patch_wikitext_register.py) and [publication classifier](../../../tests/common/publication_sweep.rs) support retail, Mists Classic and Classic Era but not Wrath Classic. Their default-retail path is unsuitable. Extending an enum for zero enumerated members would add abstraction without positive proof; no shared classifier/generator/runtime edit is made. Own source/profile accounting instead rejects incorrect profile/client-line labels, foreign successors and invented publication/behavior credit. This does not repair the shared harness's inability to classify a future nonempty Wrath register; coordinator owns that limitation when such an inventory exists.

Existing `--text-only --canonical-patch-navigation` extractor flags suffice. Both bullets remain substantive `audit-pending` seeds, resolved to explicit **UNPROVEN** audited statuses; resources remain metadata. No new extractor flag is required.

## Immutable historical proof

Frozen response/wikitext identity and both manifest hashes were validated **before copying**. Response: 747 bytes, SHA-256 `594cacde59ac51e99109e28fea49ea153a1969994032064f83b8e3dfad5b3b05`. Wikitext: 463 bytes, SHA-256 `1849b20140e6c83b62c4a1b5a14adba2143de8ce01d9a368b7a52d166d0f5e6e`. Exact manifest row retained in `source-pin.json`; source cache remains untouched.

[Validator](../../../data/patch-api/evidence/3.4.3-session-2026-10-09/validate.py) replays sealed own response, source, ledger, profile observation and copied historical extractor/generator. Counts/status totals are derived from inputs, not global register lists or fixed receipts. No dependency on original Git objects, live cache contents or later shared-tool edits. Profile/cache observations remain historical facts, not assertions about the host at replay time.

[Targeted tests](../../../data/patch-api/evidence/3.4.3-session-2026-10-09/test_source_accounting.py) exercise exact external serialized accounting and reject changed revision/source, each omitted row, fabricated prose credit, wrong client/profile, foreign successor, invented API count and native/runtime credit. These are source-accounting tests, not simulator compatibility tests. [Proof ledger](../../../data/patch-api/evidence/3.4.3-session-2026-10-09/source-proof.json) records exact revision/scope, commands, exits and sealed logs after implementation. Main owns broad sweeps/check/lint/readability/coverage/startup/final gates; none run here. No push, merge, deployment, delegation, provider/model/retry change, or other-worktree edit.

## Targeted proof ledger

At `84274cf675b7e9cf7f8048d758ad07297beb5023`: source-accounting GREEN **7/7** (22 mutation subcases), recorded-flag extraction exit 0, and own historical replay exit 0. Development RED first failed the missing derived accounting assertion against a temporary empty validator; exact log retained. Altering the sealed GREEN log later fails the exact seal with exit 1; original bytes restored and hash checked. Receipt/docs/seal-entry additions leave the tested source, historical tools, validator, test fixture and profile observation bytes unchanged. Replay derives seven rows/five metadata/two UNPROVEN, no runtime observations; its sealed-input count grows with retained receipts and is not fixed acceptance data.

No Rust changes or Cargo formatting; Python manually formatted because ruff/black are unavailable. No formatter installation, lint or broad gate attempted. All commands used the owned absolute worktree cwd. Main still owns integration and final verification, not this bounded source audit.

## Sources

- [Frozen source](../../../data/patch-api/sources/3.4.3-api-changes.wikitext), [plaintext](../../../data/patch-api/sources/3.4.3-api-changes.txt), [pin](../../../data/patch-api/evidence/3.4.3-session-2026-10-09/source-pin.json), [response](../../../data/patch-api/evidence/3.4.3-session-2026-10-09/source-response.json).
- [Spec](../../specs/patch-3-4-3-source-accounting.md), [evidence](../../../data/patch-api/evidence/3.4.3-session-2026-10-09/).

## See Also

- [[patch-5-5-4-api-audit]] — separate Mists history and publisher-loaded empty-inventory limits.
- [[patch-4-4-2-api-audit]] — source-only Cata boundary, not Wrath support classification.
- [[client-profiles]] — profile/cache selection.
