# Patch 11.1.7 API page audit

Page 628473, revision 6726774 (May 25, 2026), retrieved October 7, 2026. Audit accounts for 48 inventory occurrences and 27 non-inventory rows. Default retail carries 12.1.0, not a reconstructed 11.1.7 client. Evidence directory retains the requested `11.1.7-session-2026-10-06` name; host clock also stamped October 6. Neither changes revision or proof scope.

## Source and supersession

All eight inventory header counts match. Every later register, 11.2.0 through 12.1.0, applies chronologically; latest add/remove wins. Seven added ClientSettings CVars have later removals, all OK. Eight later registers regenerate byte-identically. Thirty-two existing raw sources, registers, coverage ledgers and gap fixtures remain unchanged against starting master `ffa0f1875`.

## Root causes and bounded fixes

Initial cached sweep: 36 OK / 12 gaps. Two C_Debug methods were registered unconditionally after retirement. Move retained legacy output under `src/c_api/c_debug.rs`, exclude its registration on supported retail epochs, and mark removed keys against namespace autostubs. No cached Blizzard deprecation wrapper is deleted. Two graphics commands were missing from the existing typed console catalog; add Command records, not CVars. Execution and native graphics metadata remain unmodeled. Two behavioral tests reproduce four defects before implementation, then pass.

An initial assisted-slot candidate fails compilation because the presumed action_spell_id field does not exist. Existing next_cast_spell_id is a recommendation, not the rotation-action identity. Candidate is withdrawn rather than fabricating a model. Remaining eight gaps: assisted-action identity; three call-profiler APIs; opted-in addon-table access/security; currency-transfer lifecycle; cooldown availability/failure reason; ButtonHeader widget DTO producer. [Per-ID review](../../../data/patch-api/evidence/11.1.7-session-2026-10-06/p1117-gap-review.json) records exact reasons and source boundaries.

## Coverage matrix

| Statements/features | Count | Proof / boundary |
|---|---:|---|
| Unsuperseded add/change publication | 28 | Partial-development-green; signatures/output/security/behavior unproven |
| Unsuperseded removals | 5 | Bounded shared policy: strict absence/registration rejection, zero alias acceptances |
| Later-superseded inventory OK | 7 | Metadata-only; no historical credit |
| Inventory gaps | 8 | Missing explicit producers/policy; generic callable lookup is not modeled publication |
| Non-inventory contracts / editorial context | 16 / 11 | Pending ranked scout / metadata-only, no runtime credit |

75 unique IDs: 28 partial / 5 bounded / 24 pending / 18 metadata. Page-accounting and publication audit complete; ledger remains in-progress because behavioral and producer gaps remain. Summary preallocation statement stays pending: existing table.create value tests do not establish internal allocation/native fidelity. Scout batches: summary 3, enums 7, DTOs 6; every extract ID assigned once.

## Development proof

[Spec and proof table](../../specs/patch-11-1-7-publication-sweep.md#local-proof) record nine isolated passing sweeps at runtime/test revision `210e4e23c`, exact eight-gap fixture and one-row negative control (8 → 9, expected exit 101). Two bounded behavior tests pass after RED. Eleven extractor/register fixtures, extraction reproduction and formatting pass. Mists test check has zero errors/non-vendor warnings; six pre-existing iced manifest warnings and vendor summary remain unsuppressed. Separate default retail build and bounded exit-0 startup return `[]`. Changed Rust lines manually audited for readability.

Seven later observation maps match retained 11.2.0 evidence exactly. 11.2.7 differs only by 27 closures already on starting master, explicitly reconciled against its follow-up outcomes; current fixture has 94 gaps, not the older 121. No later fixture change by this audit. Local Cargo logs are ignored artifacts referenced by revision-scoped proof; retained source/result/ledger validation is explicit. No full suite, agents/models, push, merge, canonical checkout edits, vendor edits or Blizzard monkey-patching.

## Sources

- [Publication contract](../../specs/patch-11-1-7-publication-sweep.md)
- [Source provenance](../../../data/patch-api/sources/11.1.7-api-changes.provenance.json)
- [Coverage ledger](../../../data/patch-api/sources/11.1.7-page-coverage.json)
- [Extract scout](../../../data/patch-api/evidence/11.1.7-session-2026-10-06/p1117-extract-scout.md)
- [Proof ledger](../../../data/patch-api/evidence/11.1.7-session-2026-10-06/p1117-proof.json)
- [Artifact validator](../../../data/patch-api/evidence/11.1.7-session-2026-10-06/p1117-validate.py)

## See Also

- [[patch-11-2-0-api-audit]] — template and next supersession boundary.
- [[patch-11-2-7-api-audit]] — pre-existing follow-up closures.
- [[console-command-registry]] — modeled catalog, not graphics command execution.
- [[client-profiles]] — supported profiles and retail epochs.
