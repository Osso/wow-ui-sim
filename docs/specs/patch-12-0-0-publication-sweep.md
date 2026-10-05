# Retail 12.0.0 supplemental publication sweep

Publication/absence breadth evidence for the [raw wikitext register](../../data/patch-api/sources/12.0.0-wikitext-register.json), revision 6747189. This supplements, not replaces, the older occurrence audit. [Investigation](../wiki/investigations/patch-12-0-0-api-audit.md#wikitext-supplement) owns observations and review classifications.

## What it must do

- [x] Retain raw revision provenance and reproducibly parse all 1,010 consolidated rows, including six uncollapsed script-object kinds; every printed inventory count must match.
- [x] Preserve byte-identical 12.0.5, 12.0.7 and 12.1.0 registers. The legacy 12.1.0 register omits optional metadata; regenerate it with `--inventory-only`.
- [x] Probe all rows in one fully loaded cached Game environment; require the non-OK ID set to equal the reviewed known-gap set exactly.
- [x] Apply later 12.0.5, 12.0.7 and 12.1.0 add/remove supersession, in order. Changed rows do not alter publication expectations.
- [x] Support `P1200_SWEEP_REGISTER` for a full scratch register and `P1200_SWEEP_OUT` for every observation, including failed runs. One flipped unsuperseded row must produce exactly one new gap.

## How it works

- [Shared sweep contract](patch-12-0-7-publication-sweep.md): classifier, cached Game preload, later-register supersession, deprecation fallback attribution.
- Script-object kinds are probed through their Lua constructors; abstract curve base uses the scalar curve, Region methods use a Texture. Constructor publication does not prove implementation semantics.

## Implementation inventory

- `tools/gen_patch_wikitext_register.py`: raw table parser, inventory-only output option for legacy serialization.
- `tools/test_gen_patch_wikitext_register.py`: uncollapsed script-object table fixture.
- `tests/common/publication_sweep.rs`: shared classification and exact gap comparison.
- `tests/patch_12_0_0_publication_sweep.rs`: source inputs and three later registers.
- `tests/data/patch_12_0_0_sweep_known_gaps.json`: 198 reviewed non-OK source IDs.
- `src/c_api/patch_retired_members.rs`: 64 exact removed namespace keys gated from `retail-12-0-0`; later removals retain later epoch gates.

## Tests asserting this spec

- Python inventory fixture and byte comparisons against all three committed later registers.
- `patch_12_0_0_publication_sweep`, separately filtered, plus one-row negative control.

## Known gaps (current cycle)

Final result: 812 OK / 198 reviewed non-OK. [Evidence and row-by-row review](../../data/patch-api/evidence/12.0.0-session-2026-10-05/) record all observations and proposed statuses; older ledger stays unchanged.

- [ ] 148 raw-absent autostub-only API entries and 19 absent added globals/members need actual publication/backing semantics.
- [ ] 19 removed globals remain raw functions without Deprecated-file attribution; successor/alias and loaded-consumer review is needed before deletion.
- [ ] Five callback/no-script/internal event rows lack a supported probe path; registration/delivery eligibility remains unproven.
- [ ] Four event removals are also listed as additions by the page. Preserve both rows as metadata conflicts, not runtime removals.
- [ ] Two added CVars lack getter/default publication; one removed CVar is a case-only scope rename resolved by case-insensitive getters. No default or scope semantics were invented.

## Transmog outfit closure contract

The assigned transmog follow-up adds behavioral proof beyond publication: catalog creation and rename, displayed selection/locks, per-outfit saved slot contents, pending appearance and situation overlays, synchronous documented events, atomic apply with collection eligibility and host pricing. Invalid applications must preserve pending state and money. Native pricing/caps, persistence and automatic situation-trigger evaluation remain unverified; local choices are marked `INFERRED` in code. Publication fixtures are retired only after the sweep observes real registrations.

Slot topology must use `Enum.TransmogOutfitSlot`, not zero-based inventory slots. Both shoulder locations are cacheable; displayed groups include the secondary shoulder only when the outfit split is enabled. Armor/weapon option collection queries use enum categories; category and illusion queries read collection state. Custom-set name validation and creation share one policy and enforce a host-configurable capacity. Set-filter reset restores checked collected/uncollected/PvE/PvP defaults (inferred).

Set catalogs and slot-source membership are explicit simulator inputs; imports stage collected set alternatives or inventory-indexed custom-set appearance triples into the pending model. Available-set queries obey collection and PvE/PvP filters. Pending price queries derive from host slot pricing when no explicit cost snapshot is provided; failed applies preserve money and pending contents. Outfit pickup sets cursor payload (including equipped-gear ID zero). Atlas assignment and reversible custom-set hyperlink encoding are inferred; native hyperlink interoperability and dropping outfits onto action bars are not claimed.

Situation UI categories/groups/options are explicit host metadata; option values must reflect pending and saved outfit selections. Equipped weapon-option queries inspect actual equipped item inventory types, not viewed-option preferences. No automatic native situation catalog or artifact option is fabricated.

Tests: `tests/transmog_outfit_situation_catalog.rs`, `tests/transmog_outfit_lifecycle.rs`, `tests/transmog_outfit_slots.rs`, `tests/transmog_outfit_transactions.rs`, `tests/transmog_outfit_catalog_inputs.rs`. Implementation: `src/c_api/c_transmog_outfit_info/`, `c_transmog_collection.rs`, `c_transmog_sets.rs`.

## Out of scope

Behavior, signatures, output shapes, secrecy/security, callback-event delivery, defaults/mutability, native-client parity, strict historical epoch builds, Enums/Structures sections, and club model changes. CVar default mismatches are diagnostic only. Contradictory added/removed rows remain separate observations rather than being silently deduplicated.
