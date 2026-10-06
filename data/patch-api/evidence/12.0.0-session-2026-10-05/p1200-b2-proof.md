# B02/B04/B05/B06 producer proof

Active goal: prove assigned 12.0.0 extract DTO parents/deltas against cached Retail documentation and consumers; fix missing fields from simulator state. No vendor, coverage-ledger, deprecated-successor or retirement-fallout edits. Required verification: local debug subsystem filters before/after, four isolated publication sweeps, startup Lua errors.

## B02

`6a8e37342` baseline: test_premade_groups 18, major_faction_renown_rewards 9, private_aura_anchors 18, structure_appearance 3 all passed. New AdvancedFilter save/copy test failed against unchanged master production code: SaveAdvancedFilter was namespace fallback, not a state mutation. Implemented copied/validated filter inputs in src/c_api/c_lfg_list_filter.rs.

| source_id | test |
|---|---|
| structures-AdvancedFilterOptions-070 | patch_12_0_0_struct_shapes::advanced_filter_parent_and_playstyle_roundtrip |
| structures-AdvancedFilterOptions-071 | patch_12_0_0_struct_shapes::advanced_filter_parent_and_playstyle_roundtrip |
| structures-AdvancedFilterOptions-072 | patch_12_0_0_struct_shapes::advanced_filter_parent_and_playstyle_roundtrip |
| structures-AdvancedFilterOptions-073 | patch_12_0_0_struct_shapes::advanced_filter_parent_and_playstyle_roundtrip |
| structures-AdvancedFilterOptions-074 | patch_12_0_0_struct_shapes::advanced_filter_parent_and_playstyle_roundtrip |
| structures-MajorFactionRenownRewardInfo-108 | major_faction_renown_rewards::major_faction_renown_rewards_publish_identity_and_optional_fields (full cached parent shape) |
| structures-MajorFactionRenownRewardInfo-109 | major_faction_renown_rewards::major_faction_renown_rewards_publish_identity_and_optional_fields; do_not_leak_across_pair_keys; return_independent_snapshots |
| structures-PrivateAuraIconInfo-112 | private_aura_anchors::nonempty_optional_bindings_publish_flattened_icon_dimensions; optional_icon_and_duration_bindings_validate_every_required_field_atomically |
| structures-PrivateAuraIconInfo-113 | private_aura_anchors::nonempty_optional_bindings_publish_flattened_icon_dimensions |
| structures-TransmogAppearanceSourceInfoData-122 | patch_12_0_0_struct_shapes::appearance_parent_and_old_casing_absence; patch_12_0_5_structure_inputs::structure_appearance_complete_records_and_illusion_values |
| structures-TransmogAppearanceSourceInfoData-123 | patch_12_0_0_struct_shapes::appearance_parent_and_old_casing_absence |

PrivateAuraIconInfo is input-only: full binding acceptance and per-field rejection prove the parent; flattened snapshot is not claimed to be an icon DTO or rendering proof. Tests construct arbitrary data, not live record claims. B02 introduced no inferred defaults beyond documented false/zero defaults.

## Progress

- [x] B02 post-commit proof
- [ ] B04 listing inputs/results
- [ ] B05 interaction/faction/schedule/set DTOs
- [ ] B06 viewer and cooldown transitions
- [ ] Publication sweeps and startup

## B04

New test fails on unchanged master LFG producer (`39b7cfe9d` only changed filter code): fractional generalPlaystyle was truncated and accepted. Active listing implementation moved into src/c_api; strict enum validation, copied updates, optional preference omission. INFERRED: enum None=0 is omitted from nilable active/search output, while input default is None.

All six IDs use patch_12_0_0_lfg_playstyle::lfg_general_playstyle_roundtrip_and_parent_shapes: structures-LfgEntryData-098, structures-LfgEntryData-099, structures-LfgListingCreateData-100, structures-LfgListingCreateData-101, structures-LfgSearchResultData-102, structures-LfgSearchResultData-103. Full current cached parent shapes, concrete Learning/Expert listing and separate search record values, optional omission, detached snapshots and invalid-enum atomicity. Does not imply that one's own listing is automatically a search result.

B02 post-commit: struct_shapes 2, reward 9, private_aura_anchors 18, structure_appearance 3, test_premade_groups 18 passed.
