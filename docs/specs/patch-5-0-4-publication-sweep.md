# Retail Patch 5.0.4 publication sweep

## What it must do

- [ ] Probe every one of the 626 pinned main/diff occurrences against current retail.
- [ ] Apply real retail successors from 5.1.0 onward; exclude Classic 5.5.x.
- [ ] Classify each occurrence, main prose statement, and retained signature without equating publication with behavior.
- [ ] Preserve current consumers, later readditions, and Classic behavior.
- [ ] Retain portable, revision-pinned evidence and exact negative control.

## Current boundary

Discovery fixture initially has no reviewed gaps; its failing run must persist every observation before classification. No runtime change or retirement authorized by parsing alone. Source TOC is 50001, not 50004.

## Tests

- `tests/patch_5_0_4_publication_sweep.rs`

## Out of scope

Native 2012 gameplay parity, vendor edits, shims, coordinator integration/migration, push and merge.
