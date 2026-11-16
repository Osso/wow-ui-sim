# Limited listfile reproducibility

The listfile generator now verifies an immutable community source, authoritative overrides, effective request selections and the expected artifact against a committed provenance file. A full-source regeneration check fails on drift rather than rewriting the CSV.

## Findings

At `892fd3e83`, current master already reproduced 147,015 rows byte-identically from release `202611141741` (source SHA-256 `c95a29658b1274d8c7cb33f1d629ffe8be8b6dcb2b7bb9177414dcd5a48e0e21`). The output SHA-256 is `338b4f7894ba061c5cc007e4e13dc30cb88c930ea598eb9763018c3973cd651e`. No CSV rows were hand-edited or changed.

Two existing Python assertions predated the explicit canonical-casing change in `2f88b2cab`. They now assert the exact canonical display paths and retain normalized-key/FDID checks. The coordinator read PR33.md and answered lt-58c4 question 2: the report recorded six missing rows from unpinned local default inputs but neither row texts nor the source identity/hash; that input is gone. The original rows are **UNRECOVERABLE**. **INFERRED cause:** use of unpinned local inputs; the exact byte-identical pinned-source reproduction is independently VERIFIED. Do not conflate this inferred historical cause with the separately proven stale test expectations.

## Sources

- [Listfile contract](../../specs/limited-listfile-reproducibility.md) — pins, regeneration, tests and historical boundary.
- [Retail metadata refresh](../../updating-blizzard-ui-to-a-new-patch.md#retail-70783-metadata-refresh--november-15-2026) — recorded full-source identity and unchanged limited output.
- Task lt-58c4 question 2, answered by agent-server-wow-ui-sim on 2026-11-16 — report contents, unrecoverable original rows, and explicitly inferred historical cause.

## See Also

- [[casc-asset-cache]] — runtime CASC lookups and pinned UI caches.
