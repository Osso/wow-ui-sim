# Patch 9.1.5 publication sweep

Account for Warcraft Wiki page 219137 revision 5920444 against current retail 12.1.0. Publication is not behavior parity.

## Requirements

- [x] Retain source, revision, provenance generator flags, every inventory/extract/context occurrence.
- [x] Probe all inventory rows with later 9.2.0–12.1.0 registers and exact reviewed gap IDs (9.2.0 added at integration).
- [x] Close only cheap meaningful modeled gaps; retain precise model/source boundaries for others.
- [x] Before any retirement, retain qualified/bare cached-retail scans; preserve live/deprecation consumers and classic profiles.
- [x] Preserve prior extracts in both modes and registers byte-identically with recorded flags.
- [x] Retain all publication sweeps, negative control, affected behavior tests, formatting, Mists warning boundary and startup `[]`.

## Exclusions

Historical reconstruction, placeholders, vendor/cache edits, sibling/canonical writes, agents/models, push and merge.

## Sources

- [Audit](../wiki/investigations/patch-9-1-5-api-audit.md).
- `data/patch-api/sources/9.1.5-*`.
- `tests/patch_9_1_5_publication_sweep.rs`.

## Known gaps

- [ ] Forty-seven exact publication failures need their recorded models or scope decisions.
- [ ] Eight substantive extract contracts, event producers/payloads and native behavior parity remain unproven.
- [x] Integration prepended 9.2.0; the 47 exact gaps are unchanged.
