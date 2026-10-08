# Patch 9.1.5 publication sweep

Account for Warcraft Wiki page 219137 revision 5920444 against current retail 12.1.0. Publication is not behavior parity.

## Requirements

- [ ] Retain source, revision, provenance generator flags, every inventory/extract/context occurrence.
- [ ] Probe all inventory rows with later 9.2.5–12.1.0 registers and exact reviewed gap IDs; leave a 9.2.0 integration placeholder.
- [ ] Close only cheap meaningful modeled gaps; retain precise model/source boundaries for others.
- [ ] Before any retirement, retain qualified/bare cached-retail scans; preserve live/deprecation consumers and classic profiles.
- [ ] Preserve prior extracts in both modes and registers byte-identically with recorded flags.
- [ ] Retain all publication sweeps, negative control, affected behavior tests, formatting, Mists warning boundary and startup `[]`.

## Exclusions

Historical reconstruction, placeholders, vendor/cache edits, sibling/canonical writes, agents/models, push and merge.

## Sources

- [Audit](../wiki/investigations/patch-9-1-5-api-audit.md).
- `data/patch-api/sources/9.1.5-*`.
- `tests/patch_9_1_5_publication_sweep.rs`.
