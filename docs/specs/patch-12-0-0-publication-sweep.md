# Retail 12.0.0 supplemental publication sweep

Publication/absence breadth evidence for the [raw wikitext register](../../data/patch-api/sources/12.0.0-wikitext-register.json), revision 6747189. This supplements, not replaces, the older occurrence audit. [Investigation](../wiki/investigations/patch-12-0-0-api-audit.md#wikitext-supplement) owns observations and review classifications.

## What it must do

- [ ] Retain raw revision provenance and reproducibly parse all 1,010 consolidated rows, including six uncollapsed script-object kinds; every printed inventory count must match.
- [ ] Preserve byte-identical 12.0.5, 12.0.7 and 12.1.0 registers. The legacy 12.1.0 register omits optional metadata; regenerate it with `--inventory-only`.
- [ ] Probe all rows in one fully loaded cached Game environment; require the non-OK ID set to equal the reviewed known-gap set exactly.
- [ ] Apply later 12.0.5, 12.0.7 and 12.1.0 add/remove supersession, in order. Changed rows do not alter publication expectations.
- [ ] Support `P1200_SWEEP_REGISTER` for a full scratch register and `P1200_SWEEP_OUT` for every observation, including failed runs. One flipped unsuperseded row must produce exactly one new gap.

## How it works

- [Shared sweep contract](patch-12-0-7-publication-sweep.md): classifier, cached Game preload, later-register supersession, deprecation fallback attribution.
- Script-object kinds are probed through their Lua constructors; abstract curve base uses the scalar curve, Region methods use a Texture. Constructor publication does not prove implementation semantics.

## Implementation inventory

- `tools/gen_patch_wikitext_register.py`: raw table parser, inventory-only output option for legacy serialization.
- `tools/test_gen_patch_wikitext_register.py`: uncollapsed script-object table fixture.
- `tests/common/publication_sweep.rs`: shared classification and exact gap comparison.
- `tests/patch_12_0_0_publication_sweep.rs`: source inputs and three later registers.
- `tests/data/patch_12_0_0_sweep_known_gaps.json`: reviewed non-OK source IDs.

## Tests asserting this spec

- Python inventory fixture and byte comparisons against all three committed later registers.
- `patch_12_0_0_publication_sweep`, separately filtered, plus one-row negative control.

## Known gaps (current cycle)

- [ ] Initial RED findings require row-by-row review; no behavior credit follows from publication.

## Out of scope

Behavior, signatures, output shapes, secrecy/security, callback-event delivery, defaults/mutability, native-client parity, strict historical epoch builds, Enums/Structures sections, and club model changes. CVar default mismatches are diagnostic only. Contradictory added/removed rows remain separate observations rather than being silently deduplicated.
