# Patch 9.0.5 publication sweep

Audit page 447309 revision 4298844 against current retail 12.1.0; publication is not behavior parity.

## Requirements

- [x] Retain pinned source, provenance flags, all 28 inventory and four extract occurrences.
- [x] Probe every inventory occurrence with later registers and exact reviewed gap IDs.
- [x] Close only cheap meaningful modeled gaps; record precise reasons for remaining gaps.
- [x] Preserve prior extracts in both modes and byte-identical registers.
- [x] Verify all publication sweeps, negative control, affected behavior, formatting, Mists warning boundary and startup `[]`.

## Exclusions

Historical reconstruction, placeholders, vendor/cache edits, canonical/sibling writes, agents/models, push and merge.

## Sources

- `data/patch-api/sources/9.0.5-*`
- `tests/patch_9_0_5_publication_sweep.rs`

## Retained boundaries

Fourteen exact publication gaps remain; per-ID reasons live in the linked audit. Added functions, widget method and event registration carry publication-only credit, not behavior parity. Main integration prepends 9.1.0; GamePadSmoothFacing should then leave the gap fixture.

- [Audit and sweep table](../wiki/investigations/patch-9-0-5-api-audit.md).
