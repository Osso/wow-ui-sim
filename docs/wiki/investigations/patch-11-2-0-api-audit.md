# Patch 11.2.0 API page audit

Page 636685, revision 6726773 (May 25, 2026), retrieved October 6, 2026. Audit accounts for 162 inventory occurrences and 82 non-inventory rows. Current default retail carries 12.1.0; publication evidence is not a reconstructed 11.2.0 client or native parity proof.

## Source and supersession

All eight inventory header counts match. Seven existing registers regenerate byte-identically. Every later register, 11.2.5 through 12.1.0, applies chronologically; latest add/remove wins. Four publication expectations reverse, all OK. Seven later sweeps preserve exact observation maps. Twenty-eight existing source/register/coverage/known-gap files remain unchanged against starting master.

## Root causes and bounded fixes

Initial cached sweep: 124 OK / 38 gaps. Missing readers of existing combat/gradient fields, four absent fog CVar defaults, five namespace-autostub retirements and unconditional Browser NavigateTo publication account for twelve rows. Existing simulator mechanisms fix them; no new service/gameplay model or Blizzard rewrite. Helpers and registrations use `retail-12-0-0`, preserving classic contracts. Five behavioral tests fail before the initial fixes and pass after them. Removed bank-tab cost is identified by final per-direction review and reproduces in repeated ordinary lookup before retirement. Empty gradient `(0, 0)` remains explicitly inferred policy.

Two retired globals remain defects, not accepted aliases: GetLootMethod uses a legacy string/zero-index return contract unlike its numeric/nil successor; SendChatMessage registration/post-load chat activation replaces cached forwarding provenance. Both need lifecycle/consumer proof, not nil guards. Other retained namespace rows need explicit backing producers; generic callable lookup is not modeled publication. Mists-only GetTalentInfo cannot establish the retail query contract.

## Coverage matrix

| Statements/features | Count | Proof / boundary |
|---|---:|---|
| Inventory add/change publication | 75 | Partial-development-green; signatures/output/security/behavior unproven |
| Unsuperseded source removals | 57 | Bounded shared policy: 46 absence/rejection, 11 exact cached deprecated wrappers/aliases |
| Later-superseded inventory OK | 4 | Metadata-only; no historical credit |
| Inventory gaps | 26 | 24 missing explicit namespace producers, two global-retirement defects |
| Extract contractual candidates | 65 | Audit-pending, five ranked follow-up batches |
| Extract editorial rows | 17 | Metadata-only, no runtime credit |

244 unique IDs: 75 partial / 57 bounded / 91 pending / 21 metadata. Ledger remains in-progress because behavioral and producer gaps remain; page-accounting/publication audit is complete. Deprecated wrapper/alias acceptance is not raw absence or successor behavior.

## Development proof

[Spec and proof table](../../specs/patch-11-2-0-publication-sweep.md#local-proof--october-6-2026) record all eight isolated sweep passes at runtime `2bd77f4d4`, unchanged later observations, exact 26-gap fixture and negative control (26 → 27, expected exit 101). Five state/retirement tests, eight extractor fixtures and two register fixtures pass. Formatting, retail check, Mists test check without non-vendor warnings and bounded exit-0 startup `[]` pass. Initial RED and intermediate borrow-check failure remain recorded; no independent agent/model/native acceptance.

Scout priorities: TOC locales/mixins/security documentation (4), font scaling/rect invalidation/StaticPopup (8), bank/tab enums and BankTabData (34), crafting/expansion/LFG/BG DTOs (9), remaining enum additions (10). Every pending/editorial source ID appears once; no behavioral credit. Existing TOC code hardcodes enUS; locale parity needs multi-locale loading proof rather than source-presence assertions.

No existing coverage ledger, catalog shop implementation, 11.2.7 fixture, vendor/Blizzard/Wowless source, canonical checkout or other worktree modified. No push, merge, agents or model CLIs.

## Sources

- [Publication contract](../../specs/patch-11-2-0-publication-sweep.md)
- [Source provenance](../../../data/patch-api/sources/11.2.0-api-changes.provenance.json)
- [Coverage ledger](../../../data/patch-api/sources/11.2.0-page-coverage.json)
- [Per-ID gap review](../../../data/patch-api/evidence/11.2.0-session-2026-10-06/p1120-gap-review.json)
- [Extract scout](../../../data/patch-api/evidence/11.2.0-session-2026-10-06/p1120-extract-scout.md)
- [Proof ledger](../../../data/patch-api/evidence/11.2.0-session-2026-10-06/p1120-proof.json)
- [Artifact validator](../../../data/patch-api/evidence/11.2.0-session-2026-10-06/p1120-validate.py)

## See Also

- [[patch-11-2-5-api-audit]] — template and next supersession boundary.
- [[patch-11-2-7-api-audit]] — later-page publication boundary.
- [[client-profiles]] — supported runtime profiles and retail epochs.
