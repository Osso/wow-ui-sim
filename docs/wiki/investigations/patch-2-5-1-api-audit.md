# TBC Classic Patch 2.5.1 SOURCE audit

Frozen page 71215/revision 701879, timestamp `2022-02-06T21:22:09Z`, audited as separate TBC Classic history. SOURCE/configuration accounting only; no runtime/native proof or unsupported-API diagnosis. [Spec](../../specs/patch-2-5-1-source-accounting.md); [literal ledger](../../../data/patch-api/sources/2.5.1-page-coverage.json).

## Provenance and boundaries

Identity/hash checked before implementation against committed [manifest](../../../data/patch-api/source-cache/legacy-2026-10-09/manifest.json) and response. Wikitext: 24,490 bytes, SHA-256 `994581e0b655316f1038f136ee4a9dd49fa4eabffc3ddcbeb92b0b13e7939daf`; response `de095f3c391a05dbeef9d43d32e10bbda74fed3570d4c2c2817bc1c42b60a230`. Full frozen registry has 101 entries through 1.0.0; registry hash `e357f60af2c745b7797ab8f9e7ac151345cddb6bf43de7786ee25785ee92e91c`.

Source literally says `* TOC: <code>20501</code>` and `{{apichanges|2.5.1|prev=1.13.7|next=2.5.2}}`. It does not spell out a client-line label. `tbc-classic` is the bounded audit label based on that context, not a new shared runtime classification. No configured profile selected. Navigation from 1.13.7 does not grant Era 1.x supersession.

Read-only templates: canonical 3.4.3 source accounting and sibling 2.5.6 evidence. Own historical copies of generator/extractor/config/registry/response permit reproduction without sibling branches or Git artifacts. Original source cache, shared tools and runtime untouched.

## Literal coverage matrix

| Source feature | Exact coverage | Proof level |
|---|---:|---|
| Global API | 301 added / 38 removed | Names/direction only; all 339 contracts UNPROVEN |
| Widgets | 36 added / 8 removed | 40 methods plus four added scripts; all 44 contracts UNPROVEN |
| Events | 33 added / 9 removed | 42 names; payload/trigger/order/security UNPROVEN |
| CVars/commands | 116 added / 32 removed | 100/18 CVars plus 16/14 commands; defaults/effects/permissions UNPROVEN |
| Remaining source | Three linked summary rows and 51 metadata rows | Links unexpanded; every nonblank literal row retained |

627 literal rows in total, 576 UNPROVEN and 51 metadata-only. Eight numerical headers reconcile. 573 occurrences are not 573 unique names: direction, section, source line and command/script kind preserved, including `graphicsTextureFiltering` on both sides and `UpdateWindow` as global and command. Source contains zero explicit signatures and zero content transclusions. Navigation template is context, not expanded content. Empty-signature inventory is not positive API parity.

API name rows cannot establish arguments, returns, errors, state transitions or security. Script/event rows cannot establish payload or dispatch lifecycle. CVar rows cannot establish defaults, accepted values or persistence. Command rows cannot establish syntax or effects. Added/removed publication itself has not been measured.

## Linked contracts

- Diffs row retains both exact compare URLs; neither diff payload is retained. Changed members/signatures, model/state/security contracts UNPROVEN.
- Deprecated APIs row retains literal `Deprecated_2_5_1.lua` classic-branch URL, not a pinned linked payload. Wrappers, migration semantics and removed-member behavior UNPROVEN.
- Community notes row retains literal consolidated-notes wiki URL, not its revision/content. Unenumerated summary changes, state/security/native semantics UNPROVEN.

No linked content reconstructed; local literal source ledger preserves full URLs even though supplemental plaintext renders link labels. No public/current network fetch used.

## Historical configured profiles

Verified from own copied `src/client_profile.rs`, Cargo feature graph and committed manifest bytes at base `f95eed96e`, not from live caches or executable probes.

| Profile | Configured interface | Manifest paths / TOCs |
|---|---:|---:|
| Retail | 120100 | 4041 / 394 |
| PTR | 120105 | 4025 / 374 |
| Wrath | 38001 | 3981 / 360 |
| Mists | 50504 | 3981 / 360 |
| Era | 11507 | 3981 / 360 |
| Anniversary | 11507 | 3981 / 360 |
| Forever | 16001 | 4398 / 349 |

Manifest entries are committed paths, not observed loaded files or TOC contents. Anniversary 11507 is not presumed TBC 205xx. No dedicated matching profile is evidenced by this copied configuration; that is not a blanket unsupported-API diagnosis. Cache/runtime/model/native/full-UI measurements not performed. Native profile correspondence remains UNPROVEN.

## Successors and ownership

2.5.2, 2.5.3, 2.5.4, 2.5.5 and 2.5.6 are pending same-TBC-history main-integration references only, without register or supersession credit. Retail 3.x, Cata 4.x, Wrath 3.4 and Era 1.x never supersede this ledger. No retirement or runtime change.

Main owns integration, actual successor evaluation and any later native matching-client measurement/profile decision. This branch does not run native probes, broad/check/lint/profile/startup/full-suite/final gates or operations.

## SOURCE proof ledger

[Original proof receipts](../../../data/patch-api/evidence/2.5.1-session-2026-10-09/source-proof.json) record exact revision, argv, scope and logs. Initial tests committed at `08a957127`: nine SOURCE assertions RED because owned literal ledger absent. At `985fd2c95`, owned SOURCE fixtures GREEN 9/9, including all 627 single-row omission controls. At `eb48b0fc6`, historical validator PASS. At `c1391db3f`, fresh copied SOURCE replay PASS, both serialized ledger/log tamper controls rejected at exact seals and original bytes/hashes restored. [Consolidated proof ledger](../../../data/patch-api/evidence/2.5.1-session-2026-10-09/proof-ledger.json); [portable receipt](../../../data/patch-api/evidence/2.5.1-session-2026-10-09/portable-proof.json). No native/final acceptance credit.

Historical replay retains 29 original source/config/tool/ledger/test/log seals unchanged. Archive contains those originals plus the seals map (30 members): 272,042 bytes, SHA-256 `091d3eff74d81539ef4b9de552bc9b69994e81f9775b413f83a15a7ffd3765cf`. Fresh isolated Python process uses `PATH=/nonexistent`, empty PYTHONPATH and a copied HOME; copied tree has no Git, target, current runtime/current tools/current root config. Historical config/tools exist only under owned evidence. No mutable original checkout reads are needed by the relocated validator.

Ledger tamper changes linked contract native equivalence; log tamper fabricates native credit. Both exit 1 at the exact file seal. Exact original bytes and SHA-256 values restored; no original seal, log or source-proof receipt rewritten afterward. [Receipt seals](../../../data/patch-api/evidence/2.5.1-session-2026-10-09/receipt-seals.json) separately bind later validator/portable logs, proof ledgers, archive and original seals map. Seals are integrity assertions against committed maps, not external cryptographic authenticity or native correctness.

## Main integration — 2026-10-09

[Actual same-TBC successor comparison](../../../data/patch-api/evidence/2.5.1-session-2026-10-09/integrated/successor-comparison.json) retains three section/symbol row pairs with 2.5.2, two with 2.5.3 and three with 2.5.4. Integrated 2.5.5/2.5.6 have zero explicit API identities. These source directions earn no runtime retirement, modeled behavior or native compatibility credit. Original 29 seals and seven separate receipt seals remain unchanged; SOURCE replay gate pending. Source TOC 20501 remains distinct from configured Anniversary 11507.

## Integrated evidence retention — 2026-10-09

[Integrated proof and revision boundaries](integrated-source-and-factory-proof-2026-10-09.md) retain independent evidence separately from original source seals. Integration is not new runtime/native proof; historical log entries remain unchanged.

## Sources

- [Frozen original wikitext](../../../data/patch-api/source-cache/legacy-2026-10-09/2.5.1-wikitext.txt).
- [Owned source pin/response/config snapshots](../../../data/patch-api/evidence/2.5.1-session-2026-10-09/source-pin.json).
- [Full historical registry](../../../data/patch-api/evidence/2.5.1-session-2026-10-09/frozen-registry.json).
- [Configured profile observation](../../../data/patch-api/evidence/2.5.1-session-2026-10-09/profile-observation.json).

## See Also

- [[patch-3-4-3-api-audit]] — source-only template, distinct Wrath history.
- [[client-profiles]] — runtime profile architecture; not native TBC proof.
