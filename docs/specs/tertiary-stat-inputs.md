# Tertiary stat inputs

Next55 covers exactly three pending 12.0.5 PlayerScript rows from the [retained GlobalAPI source](../../data/patch-api/sources/12.0.5-api-changes.txt) and [register](../../data/patch-api/sources/12.0.5-register.json). Existing typed character rating fields must supply meaningful current-snapshot outputs; [unit-stat output restriction](unit-stat-output-restriction.md) owns the shared secrecy contract. As of 2026-10-02: saved parent RED and GREEN reconciled below; minimal upstream producer fix observed in the compiled snapshot. Fresh independent acceptance, profile proof and accounting remain pending.

| Exact retained ID | Literal added annotation | Existing input → getter expectation |
|---|---|---|
| `global api-PlayerScript GetAvoidance-420` | `PlayerScript GetAvoidance` / `+ SecretWhenUnitStatsRestricted` | `avoidance_rating = 2250` → `12.5` |
| `global api-PlayerScript GetLifesteal-442` | `PlayerScript GetLifesteal` / `+ SecretWhenUnitStatsRestricted` | `leech_rating = 585` → `3.25` |
| `global api-PlayerScript GetSpeed-480` | `PlayerScript GetSpeed` / `+ SecretWhenUnitStatsRestricted` | `speed_rating = 1395` → `7.75` |

These concrete host inputs are chosen fixtures, not native catalog/acquisition evidence. **GUESS / INFERRED assistant conversion:** 180 rating per percentage point, clamped nonnegative. This coefficient reuses the existing common conversion, but is neither native-verified nor a user-selected native coefficient. The requested scope permits this guess; absent native conversion evidence does not block bounded simulator development.

## What it must do

### Current snapshot and shared rating lookup

- [ ] Under `retail-12-0-5`, the three getters read existing `CharacterStats` fields at indices21 avoidance,17 lifesteal,13 speed. No duplicate percentage fields or input struct.
- [ ] **INFERRED:** percentage results equal `max(current_rating / 180, 0)`. These three cases are explicit in the ratio helper rather than classified as unknown indices.
- [ ] `GetCombatRating(21/17/13)` exposes raw ratings, including negatives; `GetCombatRatingBonus` exposes matching clamped percentages from the same snapshot. These supporting queries earn no additional row424/426 credit.
- [ ] Live host replacement of one typed field changes only its corresponding outputs. Default/explicit zero follows a demonstrated nonzero transition; independent environments do not share fields or restriction state.
- [ ] Queries leave the rating snapshot and restriction input unchanged. Existing crit9 and unknown999 controls retain current behavior, including the existing explicit-value helper's unknown-index180 conversion.

### Callback output and security

- [ ] Public callbacks succeed with exactly one result (`pcall` yields exactly `true, value`), preserving exact values through plain → restricted → plain. Each result position is checked for secrecy/access.
- [ ] Public stamped callers retain their stack taint and receive plain values when unrestricted. Restricted stamped callers receive authentic host getter results, cannot unwrap or perform arithmetic, and retain caller taint.
- [ ] Actual getter-produced secret NUMBER results remain rooted with secrecy and raw identity across forced GC and tainted calls; trusted recovery reveals exact values without changing typed inputs. Tests do not fabricate markers, replace APIs, or assume secret Boolean raw equality.

### Recompute and profile boundary

- [ ] Getters read the current computed snapshot, not sticky host overrides. Existing equipment recomputation may reset these unpopulated ratings to zero; subsequent host replacement is immediately visible.
- [ ] New field lookup/bonus cases are gated to `retail-12-0-5`; profiles without that epoch retain previous zero behavior. No other stat formula, state, gear computation, or restriction producer changes.

## How it works

- [Lua API architecture](../lua-api.md)
- [Shared restriction contract and evidence](unit-stat-output-restriction.md)

## Implementation inventory

- `src/lua_api/state_types/character_world.rs` — existing `CharacterStats.{avoidance_rating,leech_rating,speed_rating}` and snapshot computation; unchanged.
- `src/lua_api/globals/real/combat_stats.rs` — indices13/17/21 map to existing speed/leech/avoidance fields; raw bonus queries reuse that mapping and the explicit guessed180 ratio. One `cfg!(feature = "retail-12-0-5")` constant gates the new lookups/bonus branch. Existing getters and `push_stat_number`/taint behavior are unchanged; older profiles retain zero even with nonzero host fields.
- `src/lua_api/globals/admin_equipment.rs` — existing equipment callbacks replace stats with `CharacterStats::compute`; unchanged.
- `src/c_api/c_secrets.rs` — existing trusted number output wrapper; unchanged.

Root cause before this fix: `combat_rating_for` omitted indices13/17/21 despite those fields already existing; its unknown arm returned zero. `GetCombatRatingBonus` separately omitted them. The three getters already route through the shared rating lookup and ratio helper. Adding downstream percentage state would duplicate existing inputs and conceal the upstream omission; that earlier proposal in `/tmp/patch-12.0.5-zero-stat-input-boundary.md` is rejected.

`CharacterStats::compute` starts from defaults and does not populate the three ratings. No public `PlayerState::recompute_stats()` method exists in the inspected Lua state implementation. The test instead uses the actual existing `A_Admin.UnequipItem(1)` callback, which replaces the snapshot through `CharacterStats::compute`; it asserts item removal, zero reset and later host replacement. No new admin API or persistence contract is introduced.

## Tests asserting this spec

`tests/tertiary_stat_inputs.rs`: 14 focused tests, cfg `retail-12-0-5`, discovered by the existing grouped integration harness; no new Cargo target.

| Coverage | Tests | Proof |
|---|---:|---|
| Exact getters with nonzero raw/bonus lookup | 3 | Saved parent RED → GREEN; independent acceptance pending |
| Live replacement, default/reset, negative clamp, environment isolation, query immutability | 5 | Saved parent RED → GREEN; independent acceptance pending |
| Toggle arity/values, public/restricted stamped taint, authentic rooted GC/recovery | 4 | Saved parent RED → GREEN; independent acceptance pending |
| Actual equipment snapshot replacement; crit/unknown/explicit-ratio controls | 2 | Saved parent RED → GREEN; independent acceptance pending |

Saved RED proof: `/tmp/patch-12.0.5-batch55-red-build-result.json` and `/tmp/patch-12.0.5-batch55-red-run.json`, with parent full outputs. Artifact SHA256 `caf93dbd864b986aceec5e72cb94b6fcf45c62acda4488e3c1fd12fbad2e11ee`; dirty-source diff SHA256 `6967f0b47312d926c2359bd29bc1c26d4d1d523522abf8d87e067104da170a1a`. Dirty combined evidence includes preserved unowned `src/c_api/aura_duration.rs`, not clean-revision proof; this producer change invalidates RED as current-code proof. Formatting is not GREEN.

All14 batch55 tests and existing four restriction tests/zero fixtures remain unchanged. Source420/442/480 remain `audit-pending` and uncredited: **217 pending/131 bounded/14 partial =362**. No capability/page accounting, data, PLAN or unrelated annotation promotion; supporting raw/bonus observations do not broaden row424/426 credit.

## Reconciled batch55 saved parent GREEN — 2026-10-02

Inputs/tests revision `4610f9a42bd5ae34045266536c76fe9988274e6e`; actual producer `bd71ffddbe0521823c345e68ca4971f57e229b36` changes only the shared rating lookup, explicit divisor cases and bonus mapper. Three field branches share one `cfg!(feature = "retail-12-0-5")` decision; no new PercentState. The explicit180 coefficient and nonnegative clamp remain **GUESS / INFERRED**, not native conversion evidence. Existing callbacks, secret output wrapper, state and tests are unchanged by that producer commit.

Both saved snapshots carry dirty diff SHA256 `6967f0b47312d926c2359bd29bc1c26d4d1d523522abf8d87e067104da170a1a`; preserved unowned source is not credited or inspected here. These are snapshot-plus-dirty proofs, not clean revision or current-tree acceptance.

| Saved observation | Exit | Count/cost |
|---|---:|---|
| RED compilation | 0 | 158.60273947101086s |
| RED tertiary run | 101 | 0 PASS/14 FAIL; 4.041617421084084s |
| GREEN compilation | 0 | 368.3648609179072s (368.364861s rounded), separate from runtime |
| GREEN tertiary | 0 | 14 new PASS; 16.710450819926336s |
| GREEN character | 0 | 35 controls PASS, including four restriction tests; 33.573984101065435s |
| GREEN unit | 0 | 25 controls PASS; 17.225789727992378s |
| GREEN BreakUpLargeNumbers | 0 | 20 controls PASS; 7.298436012933962s |
| GREEN action | 0 | 17 controls PASS; 5.7219406380318105s |
| GREEN space | 0 | 8 controls PASS; 2.8313578619854525s |
| GREEN startup | 0 | `[]`; 6.245067259063944s |

Actual six-run runtime SUM **83.3619591619353740s**; with startup **89.6070264209993180s**. All six run exits0; exact names below establish **119 unique PASS =14 new +105 controls**. Four restriction tests are already inside35, never added again. Compiler JSONL outputs each contain738 records, successful build-finished and no compiler-message diagnostics; no fresh check/readability/profile gate was run.

Observed getter/raw/bonus values, negative clamp, live host replacement, isolation and read-only queries match the inferred matrix above. Callback `pcall` arity is exactly `true,value`; restriction toggles preserve values/secrecy positions. Stamped public and restricted callers preserve taint; authentic restricted host numbers reject tainted unwrap/arithmetic. Rooted getter NUMBER wrappers survive forced GC with raw identity/secrecy and trusted exact recovery, including after restriction clears. These are current VM observations, not native permission/secrecy proof.

Existing `A_Admin.UnequipItem(1)` removes the item and replaces the computed snapshot: ratings reset to zero, then host replacement900/1800/450 is visible. This demonstrates replacement, not sticky overrides, model persistence or native acquisition. Earlier-profile preservation is only the producer's static cfg intent; no executed profile proof. Fresh independent verifier, requirements/acceptance and page accounting remain pending; broader goal/native parity not claimed.

### Artifact ledger

All paths below are under `/tmp/`; saved build command is `cargo test --test integration --no-run --message-format=json`. Run commands use `timeout 90 <integration-artifact> <filter> --test-threads=1`; exact argv is retained in JSON. Startup uses `timeout 90 <wow-sim-artifact> --no-addons --no-saved-vars lua-errors`.

Integration artifact `target/debug/deps/integration-a11e89d240f9bd0c`: RED SHA256 `caf93dbd864b986aceec5e72cb94b6fcf45c62acda4488e3c1fd12fbad2e11ee`; GREEN `8e7a87b74f806dcbbdb57d27f8c04204cab96e11567dc7f57502dc13857859fa`. GREEN startup `target/debug/wow-sim` SHA256 `e4b236cc2a43387eba38aaa6b4d9b36e916c782021561e05b2a81a0438c6abed`. Build-result JSON retains all additional compiler artifact hashes.

| Artifact | SHA256 |
|---|---|
| `patch-12.0.5-batch55-green-build-result.json` | `c32b6868ff1b1e3fbaf701979535d5d2e5a573876c67e6f8d874162881eb083f` |
| `patch-12.0.5-batch55-green-build.jsonl` | `d21cf105281a110332122b858284e96b47defe6f812956a3c52504df1543e2fa` |
| `patch-12.0.5-batch55-green-build.stderr` | `b01a5804143dcda91b9183a66621391247fcad4abdd27eece36efeebd6e12145` |
| `patch-12.0.5-batch55-green-run-0.stderr` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `patch-12.0.5-batch55-green-run-0.stdout` | `2ac2978c171834f84fbd4b2f64e16eb480ea2a744cf3af346a46453ed3228d3b` |
| `patch-12.0.5-batch55-green-run-1.stderr` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `patch-12.0.5-batch55-green-run-1.stdout` | `433829e04920323d79d8de3cafb4c34d8bc0623044ea45f428436bfb790c3cf4` |
| `patch-12.0.5-batch55-green-run-2.stderr` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `patch-12.0.5-batch55-green-run-2.stdout` | `479dfb8d0443beea53e79df975788e065f497612b73f4878f34f13d87b0e84c9` |
| `patch-12.0.5-batch55-green-run-3.stderr` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `patch-12.0.5-batch55-green-run-3.stdout` | `645aa7f6247d5bb293f22eed5c678f3ee5bd70fc91faf24242be39790f760952` |
| `patch-12.0.5-batch55-green-run-4.stderr` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `patch-12.0.5-batch55-green-run-4.stdout` | `a4c48416becebfd136b3b7e95abc15776bcd8e17ed8cda51240d540a588ffecf` |
| `patch-12.0.5-batch55-green-run-5.stderr` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `patch-12.0.5-batch55-green-run-5.stdout` | `c940da10adfe83c9c1e477595035249dc438823aafd2ebbf654a7346bf6fcd9b` |
| `patch-12.0.5-batch55-green-runs.json` | `de602a6bd6c5ad63eaac04caaff05b96bddbad789ada97028f9c6c605802a098` |
| `patch-12.0.5-batch55-green-startup-run.json` | `9fac5d931469d696506f3cda02aa96f41ddbb9562292abf145dc66b25276523b` |
| `patch-12.0.5-batch55-green-startup.stderr` | `d14d604206f6cad51c8911cd3c785ad506a5562bc8b3b1fd46c38f4470489835` |
| `patch-12.0.5-batch55-green-startup.stdout` | `37517e5f3dc66819f61f5a7bb8ace1921282415f10551d2defa5c3eb0985b570` |
| `patch-12.0.5-batch55-independent-artifact-manifest.json` | `6fde547f410481c139f269efed80f6310dc90fcc040d4cef560dac377230cb34` |
| `patch-12.0.5-batch55-independent-cargo-check.stderr` | `974f319b6f3179f05a71fb1bbbfb5596522202a753577bada0897c15bf6c77ab` |
| `patch-12.0.5-batch55-independent-cargo-check.stdout` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `patch-12.0.5-batch55-independent-gates.json` | `f7b4cfe9b6e8294aa6502b2fff8828add84197c96a057f3fc3858f5365cbbd4d` |
| `patch-12.0.5-batch55-independent-global-fmt.stderr` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `patch-12.0.5-batch55-independent-global-fmt.stdout` | `6ffbeac5e67d2dc2d7a6d37c4cbe48c6e478491a62f1ac5860fc549598f437b5` |
| `patch-12.0.5-batch55-independent-scoped-rustfmt.stderr` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `patch-12.0.5-batch55-independent-scoped-rustfmt.stdout` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `patch-12.0.5-batch55-readability-combat_stats.stderr` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `patch-12.0.5-batch55-readability-combat_stats.stdout` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `patch-12.0.5-batch55-readability-tertiary_stat_inputs.stderr` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `patch-12.0.5-batch55-readability-tertiary_stat_inputs.stdout` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `patch-12.0.5-batch55-red-build-result.json` | `de2af4f256e0765e954e8cc7a5be4a55c942e056d8693c0e78da52de15956d18` |
| `patch-12.0.5-batch55-red-build.jsonl` | `46f1e2527cac4f81a2638f6836d28ba934e76b0ba7cf0e0752c415a163ffb05b` |
| `patch-12.0.5-batch55-red-build.stderr` | `99bcaad64981f1c4240c2133ab0beb3bd1308cc67cee35865be524dd2cc4db10` |
| `patch-12.0.5-batch55-red-run.json` | `cf848e4a01d86cf2c9a727ba668b07d71bc2a4a223a13a29deb3f233f1627e0b` |
| `patch-12.0.5-batch55-red-run.stderr` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `patch-12.0.5-batch55-red-run.stdout` | `a45b95332a879c94db2fe81bd25b480ba42ce45ca2cf0351c53064d55096a679` |

### Exact observed119 test names

#### `tertiary_stat_inputs::` — 14 PASS

```text
tertiary_stat_inputs::actual_equipment_recompute_replaces_snapshot_and_resets_unpopulated_inputs
tertiary_stat_inputs::authentic_getter_secrets_keep_rooted_identity_after_gc_and_trusted_recovery
tertiary_stat_inputs::avoidance_reads_existing_rating_21_and_shared_bonus
tertiary_stat_inputs::default_zero_is_distinguished_from_nonzero_input_and_explicit_reset
tertiary_stat_inputs::environments_keep_rating_snapshots_and_restriction_inputs_isolated
tertiary_stat_inputs::explicit_ratio_queries_preserve_existing_crit_and_unknown_index_controls
tertiary_stat_inputs::lifesteal_reads_existing_leech_rating_17_and_shared_bonus
tertiary_stat_inputs::live_host_replacement_changes_only_the_selected_field_outputs
tertiary_stat_inputs::negative_ratings_remain_raw_but_percentage_outputs_clamp_to_zero
tertiary_stat_inputs::public_callbacks_preserve_single_result_values_across_restriction_toggle
tertiary_stat_inputs::repeated_queries_do_not_mutate_the_current_rating_snapshot
tertiary_stat_inputs::speed_reads_existing_rating_13_and_shared_bonus
tertiary_stat_inputs::stamped_public_callers_keep_taint_and_receive_plain_nonzero_outputs
tertiary_stat_inputs::stamped_restricted_callers_receive_opaque_real_getter_outputs
```

#### `character_stats::` — 35 PASS

```text
character_stats::combat_rating_value_converts_with_crit_divisor
character_stats::missing_apis::active_power_regen_selects_pool_and_tracks_updates
character_stats::missing_apis::effective_attack_power_preserves_five_distinct_fields_and_unavailable_shape
character_stats::missing_apis::mastery_tracks_existing_rating_and_effect_component
character_stats::missing_apis::missing_stats_tainted_callers_receive_opaque_host_outputs
character_stats::missing_apis::missing_stats_toggle_preserves_all_values_and_arities
character_stats::missing_apis::override_ap_by_spell_power_tracks_conversion_percentage
character_stats::missing_apis::override_spell_power_by_ap_tracks_independent_conversion_percentage
character_stats::missing_apis::pet_melee_haste_is_independent_and_missing_pet_is_zero_guess
character_stats::missing_apis::requested_power_regen_selects_input_not_active_pool
character_stats::missing_apis::secret_selection_inputs_are_allowed_when_untainted
character_stats::missing_apis::spell_penetration_tracks_explicit_amount
character_stats::missing_apis::sturdiness_tracks_explicit_percentage_not_avoidance
character_stats::missing_apis::weapon_attack_power_uses_existing_guid_and_never_creates_entities
character_stats::stat_restriction::stat_restriction_absent_unit_shapes_and_constant_outputs
character_stats::stat_restriction::stat_restriction_all_supported_outputs_preserve_values_arity_and_toggle
character_stats::stat_restriction::stat_restriction_predicate_defaults_plain
character_stats::stat_restriction::stat_restriction_tainted_callers_receive_opaque_host_results
character_stats::test_combat_rating_bonus_is_percentage
character_stats::test_combat_rating_crit_positive
character_stats::test_combat_rating_haste_positive
character_stats::test_combat_rating_mastery_positive
character_stats::test_combat_rating_versatility_positive
character_stats::test_equip_increases_stats
character_stats::test_get_crit_chance_includes_base
character_stats::test_get_haste_positive
character_stats::test_get_mastery_effect_two_values
character_stats::test_get_versatility_bonus_positive
character_stats::test_paper_doll_combat_stat_helpers_return_safe_numbers
character_stats::test_stats_drop_to_base_with_no_gear
character_stats::test_unequip_reduces_stats
character_stats::test_unit_stat_returns_four_values
character_stats::test_unit_stat_stamina_high_with_gear
character_stats::test_unit_stat_strength_is_primary_for_paladin
character_stats::test_unit_stat_strength_positive_with_default_gear
```

#### `unit_stats::` — 25 PASS

```text
unit_stats::attack_power_for_stat_is_defined_for_strength_and_agility
unit_stats::paper_doll_defensive_chance_globals_return_numbers
unit_stats::paperdoll_attribute_helpers_return_numbers
unit_stats::paperdoll_health_and_ap_sp_helpers_are_available
unit_stats::paperdoll_helpers_disable_ap_to_sp_on_intellect_spec
unit_stats::paperdoll_helpers_track_primary_stat_by_spec
unit_stats::stats_for_unknown_unit_fall_back_to_zero
unit_stats::unit_armor_returns_five_values_for_player
unit_stats::unit_attack_power_returns_three_values
unit_stats::unit_critical_strike_reads_player_crit_rating
unit_stats::unit_damage_returns_seven_values
unit_stats::unit_defense_scales_with_level
unit_stats::unit_dodge_and_parry_return_base_percents
unit_stats::unit_health_max_reads_player_health_max
unit_stats::unit_power_max_returns_only_maximum
unit_stats::unit_power_max_returns_requested_secondary_pool_only
unit_stats::unit_ranged_attack_power_matches_melee_in_sim
unit_stats::unit_ranged_critical_strike_matches_melee_in_sim
unit_stats::unit_reaction_nonexistent_unit_returns_neutral
unit_stats::unit_reaction_player_to_self_is_friendly
unit_stats::unit_resistance_returns_four_zero_values
unit_stats::unit_spell_haste_derives_from_haste_rating
unit_stats::unit_stat_indexes_map_to_strength_agility_stamina_intellect
unit_stats::unit_xp_reads_player_state
unit_stats::unit_xp_zero_for_non_player_units
```

#### `break_up_large_numbers::` — 20 PASS

```text
break_up_large_numbers::authentic_secret_false_natural_rejects_even_in_secure_context
break_up_large_numbers::authentic_secret_locale_result_rejects_under_local_public_input_policy
break_up_large_numbers::authentic_secret_true_natural_rejects_even_in_secure_context
break_up_large_numbers::current_wow_locale_provider_mutations_apply_immediately
break_up_large_numbers::formatter_keeps_provider_identity_and_caller_inputs_read_only
break_up_large_numbers::gc_keeps_global_list_and_stack_secret_roots_identical_and_secret
break_up_large_numbers::inferred_natural_truncates_both_signs_toward_zero_not_floor
break_up_large_numbers::inferred_zero_and_group_boundaries_have_decimal_not_abbreviated_output
break_up_large_numbers::locale_and_input_fixtures_are_isolated_between_environments
break_up_large_numbers::locale_provider_collects_before_returning_public_locale_with_live_roots
break_up_large_numbers::locale_provider_lua_error_preserves_callers_and_multiple_query_recovery
break_up_large_numbers::locale_provider_wrong_missing_and_non_utf8_results_reject_then_recover
break_up_large_numbers::nonfinite_numbers_reject_in_both_natural_modes_then_recover
break_up_large_numbers::optional_public_natural_rejects_nonbool_values_then_recovers
break_up_large_numbers::required_public_number_rejects_missing_nil_and_wrong_types_then_recovers
break_up_large_numbers::secret_arg1_rejection_is_conservative_local_policy_not_row411_permission
break_up_large_numbers::secret_number_and_string_natural_reject_before_formatting
break_up_large_numbers::secure_and_stamped_tainted_calls_keep_taint_across_rejection_and_recovery
break_up_large_numbers::startup_formatter_returns_one_public_grouped_string_with_default_agreement
break_up_large_numbers::wrapped_actual_frame_and_table_natural_reject_without_representation_assumptions
```

#### `action_spell_slot_identifiers::` — 17 PASS

```text
action_spell_slot_identifiers::actual_registered_ui_frame_does_not_create_a_spell_assignment
action_spell_slot_identifiers::actual_vm_secrets_reject_after_gc_without_mutation_and_public_calls_recover
action_spell_slot_identifiers::alias_replace_and_remove_change_both_queries_immediately
action_spell_slot_identifiers::environments_isolate_existing_inputs_and_queries_are_read_only
action_spell_slot_identifiers::explicit_uppercase_name_alias_resolves_shared_key
action_spell_slot_identifiers::full_colored_link_alias_overrides_embedded_spell_id
action_spell_slot_identifiers::identifier_u32_endpoints_are_valid_but_slot_zero_is_not_a_result
action_spell_slot_identifiers::invalid_required_values_reject_before_alias_coercion
action_spell_slot_identifiers::macro_and_outfit_priority_hide_same_spell_assignments
action_spell_slot_identifiers::numeric_alias_precedes_identity_and_numeric_string_requires_seed
action_spell_slot_identifiers::ordinary_public_calls_preserve_secure_and_tainted_contexts
action_spell_slot_identifiers::public_move_then_host_clear_and_replace_are_live
action_spell_slot_identifiers::replaced_default_bar_returns_one_empty_table_and_false
action_spell_slot_identifiers::returned_tables_are_fresh_and_caller_mutation_never_changes_inputs
action_spell_slot_identifiers::two_direct_assignments_return_exact_real_slots_not_spell_ids
action_spell_slot_identifiers::unknown_and_known_unslotted_spells_and_unseeded_strings_miss
action_spell_slot_identifiers::unrelated_spell_is_excluded_from_singleton_query
```

#### `string_util_space_limit_security::` — 8 PASS

```text
string_util_space_limit_security::empty_text_cannot_short_circuit_secret_number_limit_rejection
string_util_space_limit_security::forced_gc_preserves_rooted_secret_identity_state_and_public_recovery
string_util_space_limit_security::public_three_limit_matrix_preserves_non_space_bytes_and_single_public_string
string_util_space_limit_security::retained_public_strict_limit_policy_rejects_and_recovers_in_both_contexts
string_util_space_limit_security::secret_number_limits_reject_on_ordinary_valid_text_in_both_contexts
string_util_space_limit_security::secret_numeric_string_limit_rejects_without_coercion_in_both_contexts
string_util_space_limit_security::space_free_text_cannot_short_circuit_secret_number_limit_rejection
string_util_space_limit_security::wrapped_actual_frame_and_table_limits_reject_in_both_contexts
```

## Known gaps (current cycle)

- [x] Parent compiled and observed genuine RED before producer implementation: revision `4610f9a42bd5ae34045266536c76fe9988274e6e`, compile exit0/158.60273947101086s; run exit101/4.041617421084084s, 0PASS/14FAIL. Failures occurred after startup/setup at value assertions; the authentic rooted-secret test reports an assertion failure, not the named exact-value error.
- [ ] Fresh independent verifier acceptance, current-source check and profile-preservation execution proof remain pending. Saved parent GREEN is bounded observation only; requirements and page accounting stay unchecked. No verifier name/ID is established in these artifacts.
- [ ] Native conversion, acquisition/catalog, rating-by-level/profile, activation rules, permissions and secrecy parity are unknown. These tests establish only the inferred simulator contract.

## Out of scope

- Duplicate percentage inputs, new structs, admin APIs, catalog/acquisition, gear synthesis or preservation of host inputs across recomputation: neither required nor grounded.
- Changes to other formulas, state, automatic restriction producers or prior-profile ordinary values.
- Local build/test/check/readability/coverage/gates, delegation, model CLI, push and accounting changes: parent owns verification and acceptance.
