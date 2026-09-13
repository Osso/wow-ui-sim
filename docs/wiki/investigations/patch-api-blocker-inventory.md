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
2. Follow `probe_plan` when present for row-specific observations, dependencies and remaining design gaps; otherwise use `probe_protocol` as routing only. Neither is an executable test specification.
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

## Concrete probe-design follow-up

**380 rows now link to 31 narrower plans** in the JSON. The other **741 rows retain routing protocols only**; complete key coverage does not mean complete probe design. All 31 plans remain `ready_to_execute: false`: native fixtures, an exact producer/consumer or an unresolved contract is missing. Plans specify observations, not guessed native outcomes.

| Plan family | Concrete distinction to capture |
|---|---|
| Cast/channel identities | Preserve nil tuple positions; compare castID and castBarID across consecutive/replaced casts without aliasing identities. |
| Unit realm/base sex | Same-realm versus cross-realm return values; displayed versus base sex across a real disguise fixture. |
| Numeric formatting/markup | Signed tie-boundary outputs and locale; isolated markup plus all 16 combinations of four pinned 12.0.0 controls, including stripNewlines. A later fifth control needs separate build proof. |
| Action charges/aura time | Populated producer transitions, recharge/refresh/removal and exact field relationships; no zero/default-table credit. |
| CVar/Perks/math publication | Compare both CVar spellings read-only; distinguish raw/ordinary lookup and base/target load phases from native removal proof. |
| Crafting fields | Identify a real populated structure producer before asserting a removed key; unrelated internal itemID tables are not substitutes. |
| PTR region/map structures | Find an actual consuming/exposing API; global nil or scalar key/time returns do not imply a missing public constructor. |

Investigation suggestions rejected: mutating the CVar is unnecessary for the recorded getter case-sensitivity gap; later cached cast/action signatures cannot replace the pinned 12.0.0 contract; a fifth cached StripHyperlinks control cannot expand the four-control 12.0.0 matrix without target-build evidence. Security enforcement remains deferred even when an ordinary producer observation is possible.

Independent refinement verification passed at `747017217` after correcting the four-control hyperlink matrix. Proof: `/tmp/verify-blocker-probe-plans-ledger.json` and `/tmp/verify-blocker-probe-plans-corrected-ledger.json`. The initial 35 links/13 plans and unchanged six-manifest baseline were checked; source review supports the selected dependencies, not runtime availability or all remaining semantic contracts. A further 144 transmog rows now link to six plans separating slot/outfit state, catalogs, actions/costs, custom sets, appearance sources and set filters. A further 201 pending housing/encounter rows link to 12 plans. Integration excluded 28 proposed assignments absent from the unresolved inventory and specialized one duplicate Edit Mode assignment; no resolved rows were reclassified. Housing plans were corrected against actual pending rows rather than unrelated removal examples. These family extensions await independent source review.

## Refresh and verification

The JSON records each source manifest's SHA-256 and exact row keys. On refresh, compare all six committed manifests: every `evidence-required` row must occur once, other statuses must not appear, literal gap text and evidence references must match, and every protocol must resolve. Reassess explicit cases rather than inferring semantics from keywords. Keep manifest classifications unchanged unless separate behavioral evidence justifies credit.

Independent consistency verification at `19b2a4920` passed: six committed manifest hashes unchanged from `88ccc385a`; all 1,121 keys occur exactly once with matching literal notes and distinct reference sets; protocol counts and links resolve. Repeated reference paths are deduplicated in 39 rows without losing distinct references. Proof: `/tmp/verify-patch-api-blocker-inventory-ledger.json`. Representative semantic checks do not establish all native contracts. Runtime tests from preceding slices are unchanged and were not rerun.

## Sources

- [Audit contract](../../specs/patch-api-audit-manifest.md) — evidence and bounded-credit requirements.
- [Inventory JSON](../../generated/patch-api-blockers.json) — exact row keys, source hashes, missing evidence and acquisition protocols.
- [12.0.0 manifest](../../../data/patch-api/12.0.0.json) — primary remaining occurrence set; other manifest paths and hashes are listed in the JSON.

## See Also

- [[patch-12-0-0-api-audit]] — accumulated bounded implementation proof.
- [[patch-12-0-5-api-audit]] — retained native-capture gaps.
- [[patch-api-audit-manifest]] — manifest system and provenance.
