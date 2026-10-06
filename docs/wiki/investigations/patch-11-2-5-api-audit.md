# Patch 11.2.5 API page audit

Retained page 641912, revision 6726772 (May 25, 2026), retrieved October 6, 2026. Pipeline accounts for 163 inventory occurrences and 73 non-inventory rows. Publication proof observes current default retail 12.1.0, not a reconstructed 11.2.5 client.

## Source and supersession

All eight inventory header counts match. Existing six registers regenerate byte-identically. Apply 11.2.7, 12.0.0, 12.0.1, 12.0.5, 12.0.7 and 12.1.0 chronologically; latest later add/remove wins and changed rows preserve publication. Four expectations reverse, all OK; two are CVars removed in 11.2.7. The 11.2.7 sweep already lists only newer registers and needed no modification.

## Root cause and bounded fix

Initial sweep: 116 OK / 47 gaps. `GetAllowRecentAlliesSeeLocation` and `SetAllowRecentAlliesSeeLocation` already use simulator boolean state, synchronous change events, strict validation, secret caller checks and independent environments. Module, registration and state were gated to Forever only. Widen those existing gates to `retail-12-0-0`; classic publication remains excluded. Five existing tests reproduce nil-global failures, then pass unchanged behavioral assertions. Intermediate omitted-state-gate compile failure remains explicit in proof; no namespace producer or new backing model was invented.

Non-inventory seeding mislabeled nested Deprecated resource links and the build comparison as contractual prose. A new fixture fails first; page-specific classification fixes metadata while preserving all later extraction behavior. The socketing relocation summary and enum/structure parents/members remain pending.

## Coverage matrix

| Statements/features | Count | Proof / boundary |
|---|---:|---|
| Inventory add/change publication | 92 | Partial-development-green; signatures/output/security/behavior unproven |
| Source removals | 22 | Bounded shared removal policy: seven absent, fifteen exact cached deprecated aliases |
| Superseded inventory OK | 4 | Metadata-only; current later expectation, no historical credit |
| Inventory gaps | 45 | 42 namespace producer gaps, three unsupported 3D methods |
| Extract contractual rows | 58 | Audit-pending, five ranked follow-up batches |
| Extract editorial rows | 15 | Metadata-only, no runtime credit |

236 unique IDs: 92 partial / 22 bounded / 103 pending / 19 metadata. Removed-global deprecation aliases are deliberately distinguished from raw absence and from successor behavior. Generic namespace lookup functions are not explicit publication and never count as modeled producers.

## Development proof

At runtime revision `57b50a24c`, all seven isolated sweeps pass with exact gap baselines. Six later observation maps are unchanged. Negative control adds exactly one gap/changed observation (45 → 46; expected exit 101). Five location preference tests, seven extractor fixtures and two parser fixtures pass. Formatting, retail check, warning-free non-vendor Mists test check and startup exit-0 `[]` pass. No independent/native acceptance.

Scout prioritizes socket relocation (one row), cooldown DTO/flags (five), AddOn/quest/Mythic+ DTOs (ten), trait enums/increased ranks (fifteen), then other enum additions/renames (27). Exact IDs, investigation paths and observable proof requirements are retained in the scout; no behavioral credit awarded.

No vendor/Blizzard/Wowless files, existing coverage ledgers, concurrent 11.2.7 namespace producers or its known-gap fixture changed. No push, merge, delegation or external model use.

## Sources

- [Publication contract and final proof](../../specs/patch-11-2-5-publication-sweep.md)
- [Source provenance](../../../data/patch-api/sources/11.2.5-api-changes.provenance.json)
- [Coverage ledger](../../../data/patch-api/sources/11.2.5-page-coverage.json)
- [Per-ID review](../../../data/patch-api/evidence/11.2.5-session-2026-10-06/p1125-gap-review.json)
- [Extract scout](../../../data/patch-api/evidence/11.2.5-session-2026-10-06/p1125-extract-scout.md)
- [Proof ledger](../../../data/patch-api/evidence/11.2.5-session-2026-10-06/p1125-proof.json)

## See Also

- [[patch-11-2-7-api-audit]] — later-page template and supersession boundary.
- [[client-profiles]] — runtime profile and supported retail epochs.
