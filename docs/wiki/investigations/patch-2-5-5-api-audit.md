# Patch 2.5.5 Classic/TBC SOURCE audit

Bounded source/contract slice of the frozen full registry through 1.0.0. Base `acf7fbfe9b07dc342b8078cb1c54b702151a23d6`, owned branch `p255-source`, worktree `/home/osso/.worktrees/wow-ui-sim-p255-source`. Identity validated 2026-10-09 before copying; source-only, not runtime/native acceptance.

## Provenance and literal identity

Frozen page **686953**, revision **6838475**, timestamp **2026-08-20T15:48:09Z**. Raw **399 bytes**, SHA-256 `1ded7ad90515d5437907eccd15d367592d10ef876414fa6801b8dd53d15177e2`; response SHA-256 `131f2318b95b29a9140f76c4fde87d9c3ed0564df7fbd5486cae8035075059bc`. Original committed manifest/input pair untouched. Own [evidence](../../../data/patch-api/evidence/2.5.5-session-2026-10-09/) retains exact response, source pin, full frozen manifest/registry, copied historical tools and actual configured inputs.

Literal source: “This is the pre-patch for” the linked **Burning Crusade Classic Anniversary Edition**, TOC **20505**. This explicitly establishes the Classic/TBC source boundary, not historical retail 2.x or an assumption based on version/profile name. The linked page's content, native build state and pre-patch behavior are unexpanded/unmeasured.

## Exact source coverage matrix

[Ledger](../../../data/patch-api/sources/2.5.5-page-coverage.json) preserves every nonblank raw row; blank lines 2/4 carry no capabilities.

| Raw line / ID | Literal statement | Proof level |
|---|---|---|
| 1 / source-context-001 | `apichanges`, positional 2.5.5, prev=2.5.4 | Metadata navigation; no supersession |
| 3 / prose-undated-003 | Linked Burning Crusade Classic Anniversary pre-patch assertion | **UNPROVEN** state/native/linked contract; literal client identity retained |
| 5 / source-context-005 | Resources heading | Metadata |
| 6 / source-context-006 | TOC 20505 | Metadata source expectation; separate **UNPROVEN** native correspondence |
| 7 / source-context-007 | Two external 2.5.4..2.5.5 diff links | Metadata resources; two **UNPROVEN** unexpanded linked contracts |

**Five nonblank rows: four metadata, one UNPROVEN summary. Four explicit UNPROVEN contract records: linked pre-patch assertion, native interface expectation, two external diffs. Zero explicit API occurrences, signatures, inventory header counts, removals or content transclusions.** Navigation is the only template; no hidden content expansion. Source row count and contract count intentionally differ: the resource row contains two links.

Each contract preserves its target and leaves unspecified member identities, arguments, returns, event triggers/payloads, state transitions, security rules and native equivalence null. No publication register or meaningful literal model can be derived: no concrete API input/output or state transition is specified. Zero local inventory is not positive API/native/full-UI parity. Linked resources are not treated as empty sources or fabricated inventories.

## Separate Classic/TBC history

Only a same-line successor can establish supersession. Frozen **2.5.6**, page 685353/revision 6778086/time 2026-07-22T05:42:00Z, TOC 20506, prev=2.5.5 is retained as a **reference-only pending main integration** record with its original frozen response/raw/pin. Its editorial revision predates the 2.5.5 editorial revision; chronology of patch navigation is distinct from editorial timestamp ordering. No register/model/native closure transferred from the read-only 2.5.6 template. `later_registers=[]`; pending reference has `supersession_credit=false`.

Retail 3.x, Cata 4.x, Wrath Classic 3.4.x and Era 1.x are foreign histories, not supersession. The full registry ends at 1.0.0; preserving it does not mean auditing all its pages in this slice. Main owns 2.5.6 integration and future same-line reconciliation.

## Actual configured interfaces, separately observed

[Historical observation](../../../data/patch-api/evidence/2.5.5-session-2026-10-09/profile-observation.json) derives from copied base Rust interface/cache arms, complete Cargo feature graph and committed manifests; replay does not consult current runtime files.

| Feature/cache | Configured interface | Manifest paths / TOCs / Mainline TOCs |
|---|---:|---:|
| client-retail / retail | 120100 | 4041 / 394 / 62 |
| client-ptr / ptr | 120105 | 4025 / 374 / 45 |
| client-wrath / wrath | 38001 | 3981 / 360 / 56 |
| client-mists / mists | 50504 | 3981 / 360 / 56 |
| client-era / era | 11507 | 3981 / 360 / 56 |
| client-anniversary / anniversary | 11507 | 3981 / 360 / 56 |
| client-wowforever / wowforever | 16001 | 4398 / 349 / 2 |

Anniversary **11507 is not source 20505**. No inspected configured interface matches 20505; this is not a blanket unsupported-client/API diagnosis. Manifest paths are not TOC contents or loaded-client observations. Wrath/Mists/Era/Anniversary manifests are identical at this base, including Mainline TOC paths; no client correspondence inferred. Cache, runtime, model, native and full UI measurements **not performed**.

Copied generator supports retail/mists-classic/classic-era; copied publication classifier additionally supports wrath-classic but not this TBC line. No shared classifier/default-retail/profile edits merely to force wrong-profile measurements. Existing extractor flags `--text-only --canonical-patch-navigation` reproduce plaintext without a local adapter or linked expansion.

## Historical proof and ownership

Own source tests exercise external serialized accounting, not runtime implementation shape. RED retained: required derived summary fails against the exact empty validator; test/stub/hash/log scope saved. Implementation, seals and source fixtures committed before GREEN verification. At `6566d762f`, SOURCE fixtures GREEN **10/10**, own historical validator exit **0** with **29 seals**. [Source proof ledger](../../../data/patch-api/evidence/2.5.5-session-2026-10-09/source-proof.json) records full tested revision, argv/cwd/logs and exact tested input hashes. Receipts add three seals (32 total) without changing tested bytes; no fresh test rerun or broad/final gate claimed.

Historical replay uses relocated `__file__`, copied original source/tools/configuration/ledger/proof and byte seals, not original Git objects, target, current tools/source/runtime or live cache. Serialized ledger and GREEN-log tamper controls must fail at their exact seals and restore original bytes/hashes. Archive freezes its own seal map; later receipt additions do not retroactively change proof scope.

No Rust/runtime/profile/shared tool/vendor/cache/Wowless edit, native probe, delegation/model CLI, operation, push/merge/deploy. Main owns integration and native/final-gate acceptance. Unavailable matching configured interface remains a measurement boundary, not an API failure claim.

## Sources

- [Raw](../../../data/patch-api/sources/2.5.5-api-changes.wikitext), [plaintext](../../../data/patch-api/sources/2.5.5-api-changes.txt), [ledger](../../../data/patch-api/sources/2.5.5-page-coverage.json).
- [Own spec](../../specs/patch-2-5-5-source-accounting.md), [historical evidence](../../../data/patch-api/evidence/2.5.5-session-2026-10-09/).
- Read-only templates: canonical 3.4.3 source accounting, sibling `p256-source` source accounting. No foreign contracts copied as proof.

## See Also

- [[patch-3-4-3-api-audit]] — separate Classic line/source-only limits.
- [[client-profiles]] — configured architecture, not native correspondence.
