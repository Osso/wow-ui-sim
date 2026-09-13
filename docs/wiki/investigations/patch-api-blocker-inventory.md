# Patch API blocker inventory

Verified snapshot: **2026-09-13**, source revision `88ccc385a`. [Row inventory](../../generated/patch-api-blockers.json) maps every `evidence-required` occurrence to its literal manifest gap, source references, proposed probe protocol and dependency. **1,121 rows; no status or compatibility-credit changes.** This is an evidence-acquisition inventory, not proof that all remaining behavior is impossible to model.

## Coverage

| Manifest | Rows |
|---|---:|
| 12.0.0 | 1,095 |
| 12.0.5 probes | 4 |
| 12.0.7 | 0 |
| 12.1 behaviors | 12 |
| 12.1 FrameXML | 0 |
| 12.1.5 | 10 |

Counts exclude `best-effort` residual gaps and `exception-requested` rows. Retail 12.0.0 remains **2313 best-effort / 1095 evidence-required / 2 exceptions**. Eight Mists AccountStore failures remain unresolved; this inventory supplies no Mists proof.

## How to use a row

1. Resolve `(manifest, id)` and read `missing_evidence` with `basis_references`. Notes are copied from the manifest, not newly established native facts.
2. Follow `probe_protocol` into the shared protocol definition for the next observation and its dependency. Protocols propose acquisition methods; they are not executable test specifications.
3. Establish concrete inputs, producer/consumer and target build before writing assertions. If these are absent, record that probe-design blocker rather than inventing values.
4. Separate ordinary behavior from secret/security/VM obligations before execution. A mixed row is not wholly excluded merely because its notes mention secrets.
5. Capture observations, then reassess only the demonstrated scope. Native absence, simulator state, rendering and lifecycle remain distinct proof levels.

## Work queues

Protocol assignment uses literal change/owner fields, with explicit later-patch cases. It is a routing aid, **not semantic classification of every row**.

| Proposed protocol | Rows | Missing prerequisite / next observation |
|---|---:|---|
| API/state | 456 | Establish behavioral inputs/model; observe named returns and state changes. Signature-only cases remain design-blocked. |
| Structure producer | 318 | Identify populated producer/consumer; vary input and observe exact field relationships. |
| Publication/removal | 233 | Capture exact old symbol across base/target load phases; distinguish field producers, CVar lookup and globals. |
| Event producer | 88 | Trigger an actual transition; capture tuple, order, multiplicity and unchanged-state control. |

The removal queue includes **192 enum/constant/CVar rows**. Local full-LoD simulator loading is useful simulator evidence, not native removal timing or dynamic-publication proof. Record failed/skipped addon loads; incomplete loading is not absence proof.

### Explicit later-patch cases

| Protocol | Rows | Next observation / blocker |
|---|---:|---|
| Deferred security | 15 | Exact forbidden/private/secret boundaries remain outside this phase. |
| Access predicate | 1 | Locate annotated API; `RequiresTimedSignalMapAccess` is not an ordinary constructor. Enforcement deferred. |
| Region parameters | 5 | Locate the constructor consuming `CreateRegionParams`; observe each supplied field's effect. |
| Timed entry | 3 | Locate map entry producer/consumer; establish key/time semantics from populated transitions. |
| Window transition | 1 | Capture actual maximize/restore signal; equal dimensions alone cannot identify transitions. |
| Store population capture | 1 | Existing capture never called the missing entry point. Find a build/state with callable population; observe non-secret UI state separately from protection. |

These explicit deferrals are **not an exhaustive count of security-related rows**. Earlier inventory drafts incorrectly treated all `C_` rows mentioning secrets as excluded and routed region parameters through timed-map probes; neither inference is retained.

## Refresh and verification

The JSON records each source manifest's SHA-256 and exact row keys. On refresh, compare all six committed manifests: every `evidence-required` row must occur once, other statuses must not appear, literal gap text and evidence references must match, and every protocol must resolve. Reassess explicit cases rather than inferring semantics from keywords. Keep manifest classifications unchanged unless separate behavioral evidence justifies credit.

Inventory consistency verification is pending. Runtime tests from preceding slices are unchanged and need no rerun for this documentation-only snapshot.

## Sources

- [Audit contract](../../specs/patch-api-audit-manifest.md) — evidence and bounded-credit requirements.
- [Inventory JSON](../../generated/patch-api-blockers.json) — exact row keys, source hashes, missing evidence and acquisition protocols.
- [12.0.0 manifest](../../../data/patch-api/12.0.0.json) — primary remaining occurrence set; other manifest paths and hashes are listed in the JSON.

## See Also

- [[patch-12-0-0-api-audit]] — accumulated bounded implementation proof.
- [[patch-12-0-5-api-audit]] — retained native-capture gaps.
- [[patch-api-audit-manifest]] — manifest system and provenance.
