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
- [x] B04 listing inputs/results
- [x] B05 interaction/faction/schedule/set DTOs
- [x] B06 viewer and cooldown transitions
- [x] Publication sweeps and startup

## B04

New test fails on unchanged master LFG producer (`39b7cfe9d` only changed filter code): fractional generalPlaystyle was truncated and accepted. Active listing implementation moved into src/c_api; strict enum validation, copied updates, optional preference omission. INFERRED: enum None=0 is omitted from nilable active/search output, while input default is None.

All six IDs use patch_12_0_0_lfg_playstyle::lfg_general_playstyle_roundtrip_and_parent_shapes: structures-LfgEntryData-098, structures-LfgEntryData-099, structures-LfgListingCreateData-100, structures-LfgListingCreateData-101, structures-LfgSearchResultData-102, structures-LfgSearchResultData-103. Full current cached parent shapes, concrete Learning/Expert listing and separate search record values, optional omission, detached snapshots and invalid-enum atomicity. Does not imply that one's own listing is automatically a search result.

B02 post-commit: struct_shapes 2, reward 9, private_aura_anchors 18, structure_appearance 3, test_premade_groups 18 passed.

## B05

Four runtime RED failures against unchanged master producers: missing MajorFactionData.description; ItemInteractionInfo nil-only namespace fallback; fabricated TransmogSetInfo missing expansionID; scheduler leaked nested source tables. B05 GREEN at 0608c9571: four tests passed. Catalog-default test was stale: master passes fabricated set zero; current empty catalog returns nil (documented MayReturnNothing). Migrated that expectation in e1ccc0588; transmog_set 16 passed.

Lua-owned scheduler `_state` is retained as explicit host input, with copied nested DTOs under src/c_api/c_event_scheduler.rs. Unknown-member callable-nil fallback removed, not promoted into modeled C API. Existing unsupported continent-name query remains isolated in temporary workarounds. Scheduler demo events are explicitly INFERRED, not server records; no live scheduling service claim.

| source_id | proven-by-test (patch_12_0_0_state_dtos module) |
|---|---|
| structures-ItemInteractionFrameInfo-096 | item_interaction_parent_and_combined_flags |
| structures-ItemInteractionFrameInfo-097 | item_interaction_parent_and_combined_flags |
| structures-MajorFactionData-104 | major_faction_parent_and_optional_companion |
| structures-MajorFactionData-105 | major_faction_parent_and_optional_companion |
| structures-MajorFactionData-106 | major_faction_parent_and_optional_companion |
| structures-MajorFactionData-107 | major_faction_parent_and_optional_companion |
| structures-ScheduledEventInfo-116 | scheduled_event_parent_and_independent_snapshots |
| structures-ScheduledEventInfo-117 | scheduled_event_parent_and_independent_snapshots |
| structures-ScheduledEventInfo-118 | scheduled_event_parent_and_independent_snapshots |
| structures-TransmogSetInfo-124 | transmog_set_parent_variants_and_flags |
| structures-TransmogSetInfo-125 | transmog_set_parent_variants_and_flags |

MajorFactionData model lives in C API and is re-exported for old Rust callers. New required unseeded metadata uses empty description/highlights and zero bounty/false toast (INFERRED catalog absence); companion/effect IDs are optional. GetSetInfo/GetVariantSets publish existing catalog rows. grantAsPrecedingVariant is metadata; no native grant-service parity asserted.

## B06

Viewer positive parent test passed against unchanged master producer. Cast recovery test failed: no SPELL_UPDATE_COOLDOWN dispatch. Cooldown snapshot derives optional recovery and GCD metadata from the same clock and intervals used for duration. Cast producers publish SPELL_UPDATE_COOLDOWN after state commits. INFERRED: shared GCD models start recovery; isOnGCD identifies GCD as the winning interval, not a longer spell cooldown. Unknown GCD state omits metadata; known expired GCD reports false. Native per-spell GCD eligibility remains unmodeled. Restricted recovery numbers use authentic secret wrappers; isOnGCD remains NeverSecret.

| source_id | test (patch_12_0_0_cooldown_transitions module) |
|---|---|
| structures-CooldownViewerCooldown-079 | cooldown_viewer_parent_identity_category_and_snapshots |
| structures-CooldownViewerCooldown-080 | cooldown_viewer_parent_identity_category_and_snapshots |
| structures-CooldownViewerCooldown-081 | cooldown_viewer_parent_identity_category_and_snapshots |
| structures-SpellCooldownInfo-119 | spell_cooldown_recovery_and_gcd_follow_cast_producer |
| structures-SpellCooldownInfo-120 | spell_cooldown_recovery_and_gcd_follow_cast_producer |
| structures-SpellCooldownInfo-121 | spell_cooldown_recovery_and_gcd_follow_cast_producer |

B06 post-commit GREEN: both new tests and all 27 cooldown/book-output controls passed. Native category selection remains unavailable (optional activeCategory nil); this does not substitute for positive recovery/GCD proof.

## Final verification — 2026-10-06

Last behavioral producer change: 7822aecfe (cooldown metadata/events); LFG publisher decomposition fd7640d1d is behavior-preserving and its positive nested DTO test passed afterward. Final test fixture revisions: 8ccd7eab4 (interaction replacement) and 034315289 (negative cooldown event/expiry). Formatting commit 28a2bbc83 and unused-import removal 990ac2e00 do not invalidate runtime/publication/startup proof. Documentation-only commits do not invalidate any proof.

Commands run from this worktree with absolute script path `/home/osso/.worktrees/wow-ui-sim-p1200-extract-b2/scripts/build-host.py`, always `--build-host local`, default debug target. Integration command suffix: `--test --test integration FILTER -- --nocapture`; lib suffix: `--test --lib -- FILTER --nocapture`. Sweeps use `--test --test integration FILTER -- --test-threads=1`, each alone.

| command/filter | scope revision | result |
|---|---|---|
| lfg | 0608c9571 | 96 passed; later publisher-only decomposition covered by positive LFG probe below |
| major_faction | 0608c9571 | 35 passed; later changes are fixture expansion/comment only |
| event_scheduler | 0608c9571 | 10 passed |
| transmog_set | e1ccc0588 | 16 passed; stale master fabricated-set-zero assertion migrated |
| item_interaction | 0608c9571 | 7 passed; later fixture replacement proof below |
| private_aura | 0608c9571 | 52 passed |
| struct_shapes | 0608c9571 | 5 passed; current B02 probe below |
| spell_casting | 61fcc3c7f | 19 passed |
| cooldown_viewer | 28a2bbc83 | 6 passed |
| spell_book_cooldown_outputs | 28a2bbc83 | 27 passed, including restricted numeric and public boolean controls |
| patch_12_0_0_struct_shapes | 28a2bbc83 | 2 passed |
| patch_12_0_0_lfg_playstyle | 28a2bbc83 | 1 passed |
| patch_12_0_0_state_dtos | 8ccd7eab4 | 4 passed; two ordered highlights, two interaction flag masks, host upcoming-list transition |
| patch_12_0_0_cooldown_transitions | 034315289 | 2 passed; both positive recovery and negative expiry callbacks observed |
| lib event_scheduler | 61fcc3c7f | 1 passed (migrated state test) |
| lib transmog_sets_defaults | 61fcc3c7f | 2 passed |
| patch_12_0_0_publication_sweep | 28a2bbc83 | 1 passed, isolated |
| patch_12_0_5_publication_sweep | 28a2bbc83 | 1 passed, isolated |
| patch_12_0_7_publication_sweep | 28a2bbc83 | 1 passed, isolated |
| patch_12_1_0_publication_sweep | 28a2bbc83 | 1 passed, isolated |
| cargo fmt --manifest-path ABS/Cargo.toml --check | 990ac2e00 | passed; dynamic integration sources also formatted separately with rustfmt --edition 2024 |
| --build-host local --check | 990ac2e00 | passed; no warnings from changed Rust code |
| --build-host local --run -- --no-addons --no-saved-vars lua-errors | 28a2bbc83 | exit 0, stdout JSON [] |

Master comparison: initial existing subsystem controls all passed at 6a8e37342 (premade 18, reward 9, private anchors 18, appearance 3, viewer 1, spell/book cooldown 27, major factions 14, transmog set 15, item interaction 6). Initial scheduled_event integration filter matched zero tests; scheduler lib baseline matched one and passed. New RED probes executed before changing the relevant producers; B05 producer source diff 6a8e37342..8ff519d84 and B06 producer diff 6a8e37342..e1ccc0588 are empty. These are same-production-code comparisons, not a separately modified master checkout. B02 RED: saved filter unchanged; B04 RED: fractional enum accepted; B05 RED: missing/nil/fabricated DTOs and aliased nested snapshot; B06 RED: cast missed cooldown event. Branch-introduced visibility compile error fixed in 04de7efb4. Branch unused import warning fixed in 990ac2e00. Six pre-existing iced-wgpu-patched manifest lint-name deprecations remain; vendor files are excluded and no warning was suppressed.

Readability review: inspected changed Rust functions for nesting, mutable state scope, naming and duplicate field publication. Split dungeon/PvP publication into bounded helpers. No suppressions added. Scope: complete cached DTO parents/deltas and host/cast transitions, not native eligibility, server services, grant behavior, rendering or secret-value VM operation parity. All 34 assigned IDs have bounded proven-by-test outcomes above; no assigned ID is still pending at this proof level. Page-coverage JSON unchanged.
