# Patch 5.0.1 API audit

Historical retail pageid **554410**, revision **5344081**, timestamp **2012-08-06T00:56:48Z**, supplied October 8, 2026. Entire source: `#REDIRECT [[Patch 5.0.4/API changes]]`. Destination belongs to separate **p504-page**; this audit never follows or reconstructs it.

## Coverage matrix

| Scope | Accounting | Proof boundary |
|---|---|---|
| API inventory | Zero entries/header counts | Existing default generator; no fabricated identities |
| Redirect | One `source-context-001`, metadata-only, no capabilities | Exact retained extract, no destination credit |
| Modeled behavior | Zero statements | No runtime changes or positive API parity claim |
| Publication gaps/problematic contracts | Zero source rows | Empty observations prove only harness execution |
| Retirements | Zero candidates/scans | No runtime retirement edits or absence-by-scan claim |

## Discovery boundary

[Source receipt](../../../data/patch-api/evidence/5.0.1-session-2026-10-08/source-response.json), [pin](../../../data/patch-api/evidence/5.0.1-session-2026-10-08/source-pin.json), [provenance](../../../data/patch-api/sources/5.0.1-api-changes.provenance.json) preserve the literal redirect. [Register](../../../data/patch-api/sources/5.0.1-wikitext-register.json), [extract](../../../data/patch-api/sources/5.0.1-api-changes.txt) and [ledger](../../../data/patch-api/sources/5.0.1-page-coverage.json) use existing default flags. No parser changes.

The retail sweep starts with one queued 5.0.4 empty placeholder, followed by actual 5.1.0 and all later retail registers. Classic 5.5.x registers are not successors in this historical line. Integrating 5.0.4 is coordinator work, not a missing contract in this redirect page.

## Verification

Own zero-row prefork **1/1** and all retail publication/factory cases **64/64** pass. Both own observations are exactly `{}`: harness execution only, not positive API proof. Dedicated fabricated entry fails at the exact **1 → 0** row-count boundary before API observation; no runtime or inventory edits. Eleven root Python programs pass **112/112** fixtures; supplemental 12.0.0 generator tests pass **7/7** with PyYAML supplied by `uv` after a retained missing-dependency failure. Total **119/119**. All **68 registers/65 extracts** and the supplemental 5.4.0 diff extract reproduce; exactly three inherited extraction failures (12.0.5/12.0.7 byte mismatches, 12.1.0 unsupported template) remain. Formatting and Mists `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` pass with **zero non-vendor warnings**; six inherited iced manifest warnings and summary remain unsuppressed.

[Command ledger](../../../data/patch-api/evidence/5.0.1-session-2026-10-08/p501-command-ledger.md) and compact tree-ID receipts pin exact command revisions and scopes. Historical validator and clean/later gate pending sealing. No full-suite, startup, native-client or modeled behavior claim.

## Sources

- [Publication spec](../../specs/patch-5-0-1-publication-sweep.md).
- [Pinned wikitext](../../../data/patch-api/sources/5.0.1-api-changes.wikitext).
- [Binding handoff](../../../data/patch-api/evidence/handoff-laptop-to-agent-server/HANDOFF.md).

## See Also

- [[patch-6-0-1-api-audit]] — redirect-only template.
- [[patch-5-1-0-api-audit]] — later historical retail page.
- [[patch-audit-validator-portability]] — sealed historical inputs.
