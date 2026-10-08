# Patch 6.1.0 API audit

Pinned pageid **123523**, revision **1216027** (`2015-05-01T23:34:38Z`), refetched on 2026-10-08. This is a substantive page, not a stub or redirect. Four explicit API occurrences and fourteen retained extract occurrences are accounted separately. The automated-diff transclusion is retained as an **unexpanded source boundary**, not audited API inventory.

## Coverage matrix

| Contract | Result | Proof boundary |
|---|---|---|
| Three legacy death-recap API names | Bounded current-retail publication/absence | Later 12.0.0 removals supersede the original additions; cached Blizzard aliases are attributed to exact C_DeathRecap targets. No event/link behavior credit. |
| SendChatMessage publication | One retained publication gap | Later removal expects absence, but simulator chat registration and startup restore this log-only function. Whole-word cached/source/test callers retained; no consumer patch or retirement. |
| Death recap presence/events/link | Three problematic extract rows | Generated C_* defaults have no full recap-event state or legacy link-format contract. KillingBlowInfo is not a timestamped event history. |
| Collections consolidation / automatic death watch / SocialUI | Three problematic extract rows | Historical addon consolidation is not callable API behavior; automatic combat ingestion and external Twitter/auth/screenshot-storage lifecycles lack bounded contracts. |
| TGA/PNG crash / invalid UTF-8 chat rejection | Two problematic extract rows | Do not reproduce a historical client crash. Chat page does not specify client error versus silent/server rejection; log-only conversion is not native rejection proof. |

The ledger has **18 IDs: 3 bounded publication rows, 9 audit-pending rows, 6 metadata-only rows**. Pending includes the SendChatMessage inventory gap and eight substantive extract rows. No newly modeled runtime APIs, shims, Blizzard patches or retirements were added. Existing state-backed GetKillingBlows/GetMostRecentDeathRecap tests are adjacent regression evidence, not implementations of the named legacy event/link APIs.

## Extraction and integration

- `--colon-api-bullets` separately parses standalone `::* {{api|...}}` additions under New. Existing `--prose-api-links` captures SendChatMessage as changed, not added.
- `--retain-patch-diff-reference` implementation and argparse/call hunks are byte-identical to p624-page commit `1687abf9b`. Its borrowed fixture initially failed because the current extractor normalizes headings; only that expected heading was corrected. The historical failure is retained.
- No default parser behavior changed. Recorded flags reproduce the new artifacts; old source bytes and extraction-mode outcomes are protected at pinned revisions.
- Sweep begins with placeholders for **6.2.0, 6.2.2, 6.2.4**, then 7.0.1, 7.0.3 and all later registers. These branches were inspected read-only at recorded commits; no worktree was entered or changed.

## Retirements and callers

No source removal members; no retirement candidate or removal code. Required per-member retirement scans are therefore vacuous. Supplemental scans of all four named APIs use `/usr/bin/grep -R -n -w`, exclude `*Documentation*` in the retail cache, and retain untruncated outputs. The caller scan includes all src/tests references, including indirect pcall/guard references. The retirement receipt records the complete master/p620/p622/p624 register sets at fixed revisions.

## Verification

Discovery exposed exactly one SendChatMessage absence mismatch; it is retained in the known-gap fixture, not hidden behind a shim. Targeted proof passes:

- `cargo test --test prefork_full_ui -- patch_6_1_0`: 1/1; all publication sweeps: 50/50 (49 pages plus factory), 9,020 observations. Missing-API negative control fails as required, increasing gaps 1 → 2 with three unaffected observations identical.
- Recap UI/alias prefork cases: 8/8; seeded C_DeathRecap integration probes: 6/6; bare legacy-absence integration case: 1/1.
- Python generator/extractor/validator fixtures: 32/36/8 pass. All 49 registers reproduce; 46 extracts reproduce, with only unchanged inherited 12.0.5/12.0.7/12.1.0 failures. 249 earlier source artifacts and 105 mode outcomes preserved.
- `cargo fmt --check` and `cargo check --no-default-features --features sound,gui,casc,client-mists --tests`: pass, zero non-vendor warnings. Six existing iced_wgpu manifest deprecations remain unmodified.
- All 30 pre-audit validators and own validator pass. Own validator also passes a clean detached checkout at `c0c48de70` and a synthetic later commit expanding registers/sweeps and changing src/tools/tests/other-page sources/evidence. Tampered own discovery log is rejected; temporary worktree removed. The byte-identical master `tools/check_patch_validators.py` gate (source revision `e958b7d38`) also passes on `0804aeada`: 27 clean validators, 28 after its synthetic later audit. This gate selects `validate.py`; the initial 30-case matrix also includes four `validate_integrated.py` gates.

No full integration suite. No runtime source changed, so addons-enabled startup comparison is not required by the widely-used-path condition.

## Validator portability

`validate.py` resolves register/sweep scope and prior validator inventory at recorded Git revisions. Source/test/tool/other-audit inputs read pinned Git blobs, never moving repo files or other audits' live evidence. Only own retained session receipts and artifacts are checked live. No gitignored scratch file is required. Counts are derived from files; no cwd/target equality gate. Wiki preservation is proved at a fixed documentation commit, allowing later additions.

## Sources

- [Pinned source/provenance](../../../data/patch-api/sources/6.1.0-api-changes.provenance.json) and [coverage ledger](../../../data/patch-api/sources/6.1.0-page-coverage.json).
- [Evidence](../../../data/patch-api/evidence/6.1.0-session-2026-10-08/) — fetch, discovery, caller scans, problematic cases and targeted proof.
- [Publication spec](../../specs/patch-6-1-0-publication-sweep.md).
- [C_DeathRecap model](../../../src/c_api/c_death_recap.rs) and [chat implementation](../../../src/lua_api/globals/message_verbs.rs).

## See Also

- [[patch-audit-validator-portability]] — historical proof must survive later page merges.
- [[patch-7-0-3-api-audit]] — audit/evidence template.
- [[patch-7-0-1-api-audit]] — distinguishes an actual redirect page.

## Rebased integration (2026-10-08)

Master `787b47591` supplies the merged 6.2.0/6.2.2/6.2.4 registers. Commit `59ccbbc6a` replaces the ordered placeholders with those real registers; no runtime source changed. The extractor and its fixture follow master byte-for-byte. The generator retains both `--colon-api-bullets` (6.1.0) and `--indented-api-lists` (6.2.4).

[Integrated evidence](../../../data/patch-api/evidence/6.1.0-session-2026-10-08/integrated/) preserves historical receipts separately. Six rewritten commits have stable patch IDs and blob mappings; eight changed historical blobs are retained within this session rather than requiring unreachable pre-rebase commits. Historical inventory scopes remain frozen. All 52 registers reproduce; 49 extracts reproduce with the same inherited 12.0.5/12.0.7/12.1.0 failures. Final sweep, negative-control, comparison and portability receipts follow in this directory.
