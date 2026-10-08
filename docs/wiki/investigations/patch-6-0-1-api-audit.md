# Patch 6.0.1 API audit

Fresh MediaWiki pageid **3058** fetch on **2026-10-08** pins revision **31159**, timestamp **2014-08-22T01:10:34Z**. Its entire content is `#REDIRECT [[Patch 6.0.2/API changes]]`. This is a redirect audit, not an inventory reconstructed from its destination. The separately queued `p602-page` audit owns 6.0.2.

## Coverage matrix

| Scope | Accounting | Proof boundary |
|---|---|---|
| API inventory | Zero entries and header counts | Existing default generator; no invented API rows |
| Redirect context | One metadata-only `source-context-001` | Existing default extractor; no capabilities credited |
| Modeled behavior | No source statements | No runtime/model changes required or credited |
| Problematic contracts and publication gaps | None in this page | Destination content excluded |
| Retirements | Zero candidates and zero member scans | No absence-by-scan claim |

## Discovery and retirement decisions

[Source provenance](../../../data/patch-api/sources/6.0.1-api-changes.provenance.json) and [complete fetch](../../../data/patch-api/evidence/6.0.1-session-2026-10-08/p601-fetch.json) preserve the literal redirect without following it. [Register](../../../data/patch-api/sources/6.0.1-wikitext-register.json), [extract](../../../data/patch-api/sources/6.0.1-api-changes.txt) and [ledger](../../../data/patch-api/sources/6.0.1-page-coverage.json) use existing defaults; no extractor/generator changes.

The prefork sweep has the three required one-line placeholders first: 6.0.2, 6.1.0, 6.2.0. Merged 6.2.2, 6.2.4 and every newer register follow. An empty passing sweep proves harness execution, not target-page API parity.

[Retirement decisions](../../../data/patch-api/evidence/6.0.1-session-2026-10-08/p601-retirement-decisions.json) retain complete register-tree snapshots at recorded master, p602-page, p610-page and p620-page revisions. No members exist to scan for qualified/bare retail-cache calls, `pcall(Name, ...)`, or `and Name then`. The untruncated whole-word `/usr/bin/grep` retirement-marker probe is retained explicitly as a marker probe, not caller proof. No runtime registration, retirement gate, vendor, Blizzard, Wowless or WowlessData changes.

## Verification

Discovery passes **1/1** with zero observations. All publication sweeps plus factory pass **52/52**. All four `tools/test_*.py` programs pass **4/36/32/8** tests. **51/51 registers** and **48/51 extracts** reproduce; three inherited outcomes remain unchanged: 12.0.5 and 12.0.7 byte mismatches, 12.1.0 unsupported-template error. Rust formatting and Mists `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` pass; **zero non-vendor warnings**, with six existing iced manifest warnings and their summary left unsuppressed. Complete logs and revision/scope receipts are in [evidence](../../../data/patch-api/evidence/6.0.1-session-2026-10-08/).

Initial discovery/all-sweep failures (JSON map supplied instead of the required expected-gap list) are retained under `initial-failure/`. Corrected fixture `[]` passes both filters and the Mists recheck. Python/reproduction/format proof remains valid because its scoped inputs did not change. [Command ledger](../../../data/patch-api/evidence/6.0.1-session-2026-10-08/p601-command-ledger.md) records revisions, invalidation and results. Own validator passes: 51 registers, 48 extracts, 52 sweep/factory cases and 29 prior-validator identities. `tools/check_patch_validators.py 352ebb471` passes **30/30 clean validators** and **31/31 after a synthetic later audit** (including the synthetic validator), zero failures. [Gate summary](../../../data/patch-api/evidence/6.0.1-session-2026-10-08/p601-validator-gate-summary.json) retains exact revisions and the complete report hash. No full suite, startup parity or separate area regression is claimed: no runtime areas changed.

## Validator contract

[Validator](../../../data/patch-api/evidence/6.0.1-session-2026-10-08/validate.py) reads shared source, fixtures, tools and wiki at recorded Git revisions, never live shared-file digests. Register/sweep sets use `historical_registers`/`historical_sweep_tests`; prior-validator sets use `git ls-tree`. Counts derive from retained files. Only committed own-session records are sealed. No absolute cwd/target equality and no ignored scratch dependencies. [Receipt-scope correction](../../../data/patch-api/evidence/6.0.1-session-2026-10-08/p601-receipt-scope-note.md) records why unrelated initial fixture hashes are not credited; the runner now reads committed Git blobs and rejects dirty tracked code inputs. The fresh-checkout/later-audit gate passed at sealed revision `352ebb471`; subsequent changes only retain that report and update documentation. No proof inputs changed. Wiki index/log preserve all baseline text and grow 2691 → 2695 / 418 → 426 lines. Three queued-page placeholders remain intentionally pending integration; no own-page contracts are unfinished.

## Sources

- [Publication spec](../../specs/patch-6-0-1-publication-sweep.md).
- [Pinned wikitext](../../../data/patch-api/sources/6.0.1-api-changes.wikitext).
- [Binding handoff](../../../data/patch-api/evidence/handoff-laptop-to-agent-server/HANDOFF.md).

## See Also

- [[patch-audit-validator-portability]] — historical input and portability gate rules.
- [[patch-6-2-2-api-audit]] — navigation-only stub comparison.
- [[patch-7-0-1-api-audit]] — redirect-only precedent.
