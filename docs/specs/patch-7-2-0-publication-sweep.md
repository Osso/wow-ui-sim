# Patch 7.2.0 publication sweep

Audit the pinned [source](../../data/patch-api/sources/7.2.0-api-changes.provenance.json), revision 6767100. The [wiki audit](../wiki/investigations/patch-7-2-0-api-audit.md) records evidence and implementation details.

## What it must do

- [x] Probe all three named identities: MaskTexture, Texture:SetVertexOffset and C_EquipmentSet; distinguish publication from behavior parity.
- [x] Construct texture regions through frame region factories, not CreateFrame with a region name.
- [x] Prove cached mask attachment, duplicate suppression, retrieval and removal; independent vertex-offset state updates.
- [x] Prove cached equipment-set creation, rename, specialization assignment and deletion.
- [x] Prove all three named current-retail addons load, without claiming historical implementations.
- [x] Preserve every source occurrence and exact problematic-contract reason; do not infer member retirements from domain removal summaries.
- [x] Validate complete historical register scope, source reproduction and receipt hashes without absolute checkout-path equality.

## How it works

- [Audit](../wiki/investigations/patch-7-2-0-api-audit.md).
- [Widget system](../widget-system.md).
- [Addon loading](../addon-loading-pipeline.md).

## Implementation inventory

- `tools/gen_patch_wikitext_register.py`: opt-in widget summaries and named namespace migration parsing.
- `tests/common/publication_sweep.rs`: shared observable publication probes.
- `data/patch-api/sources/7.2.0-*`: pinned source, register and occurrence ledger.
- `data/patch-api/evidence/7.2.0-session-2026-10-08/validate.py`: historical read-only evidence validation.

## Tests asserting this spec

- `tests/patch_7_2_0_publication_sweep.rs`.
- `tests/patch_7_2_0_behavior.rs`.
- `tools/test_gen_patch_wikitext_register.py`.

## Known gaps (current cycle)

- [ ] Vertex offsets are stored but currently have no rendering consumer; visual deformation is not proved.
- [ ] Page supplies neither an old equipment API member catalog nor exact deprecated-wrapper behavior; complete historical migration parity is not proved.
- [ ] Voice chat removal names a domain without a member list; no API retirement can safely be derived.
- [ ] Mac movie recording removal has no member/platform contract; no macOS recording model/native probe is available.

## Out of scope

Native 2017 parity, expansion of linked pages into new requirements, unspecified method/security/return contracts, vendor edits, full integration suite, push and merge. 7.2.5 integration belongs to the merge owner; retain its first-position placeholder until its register merges.
