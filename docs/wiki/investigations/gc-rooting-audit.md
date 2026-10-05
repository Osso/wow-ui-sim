# Host-held Lua values across collection

Rust `Val`/`GcRef` locals are not VM roots. Callback execution can collect DTOs that have not been pushed onto the VM stack or attached to a reachable table. Audit starts at `16682b415` (item tooltip rooting).

## Confirmed sites

| Site | Vulnerable window | Fix | Forced-GC regression |
|---|---|---|---|
| `c_allied_races::build_allied_race_info_table` | Strings and achievement sequence created before `CreateColor` | Run callback first, then assemble DTO without Lua execution | `race_info_survives_collection_in_color_callback` |
| `c_artifact_ui::helpers::build_artifact_art_info_table` | Earlier colors and title strings survive later `CreateColor` calls only in locals | Stack-root DTO, attach each color before next callback | `artifact_art_survives_collection_in_each_color_callback` |
| `c_major_factions::build_major_faction_data_table` | DTO and descriptive strings survive color callback only in locals | Stack-root DTO, attach strings before callback | `faction_data_survives_collection_in_color_callback` |

All three regressions failed at baseline with `table has been collected` / `invalid table reference` when the color callback performed `collectgarbage('collect')`.

## Tooltip follow-up

Eleven additional construction sites need roots: `color_segment_table`, spell, aura, toy, mount, unit, currency, companion-pet DTO builders, action binding append, achievement and shapeshift probes. Rooting `lines` alone does not keep its containing tooltip alive: reachability is parent-to-child, not child-to-parent. Root each tooltip for the entire fill; root the segment array and attach entries before color callbacks.

`tests/tooltip_gc_rooting.rs` forces collection in ten API calls, checking IDs, text, width hints, GUIDs, line counts and segment RGB/text. Mutable pet/mount/achievement/form names carry markup to reach the color callback even when their normal text is plain. All ten initially fail with collected-table errors; the action test limits collection to the final green binding line.

C API subsystem filters at `7f6d28454`: allied races 13/13, artifact panel 42/42, major factions 14/14.

## Event error-handler window

`WowLuaEnv::fire_event_with_args` and `fire_named_event_state` retained payloads only in Rust between listeners. A failed listener removes its call arguments from the VM stack, then invokes the configured Lua error handler. Forced collection there destroys table payloads before the next listener. Keep payloads on the VM stack across the entire dispatch; restore stack height on success and failure.

`host_event_payload_survives_collection_in_error_handler` reproduces the host path with a nested payload. `native_aura_event_payload_survives_collection_in_error_handler` reproduces the native path through `A_Admin.AddBuff`. Both failed with `table has been collected` in the second listener before the fix.

`LoaderEnv::fire_event_with_args` has the same listener/error-handler window; `loader_event_payload_survives_collection_in_error_handler` reproduced it after the other two dispatchers were fixed. The loader now roots payloads for its entire dispatch.

Cast completion's dynamic GUID also dies in STOP error reporting, then gets reused by SUCCEEDED. `completion_arguments_survive_collection_in_stop_error_handler` fails against baseline dispatch with `string.sub` rejecting a collected string. Rooting the host dispatcher fixes this caller too. Crucial negative-control detail: a literal full expected GUID in the test closure accidentally roots the interned string, hiding the bug. The regression checks prefix and numeric suffix instead.

This proves a cast-completion rooting vulnerability, not the cause of the original duration-test failure: that test has no failing handler/error callback and does not read the GUID. The subsequent [cast-duration investigation](cast-duration-rounding.md) captures the failing total and establishes floating-point cancellation, independent of GC. The live immediate completion check remains unchanged.

## Dynamic unit token construction

Final pass found `fire_unit_aura_full_update` interning the unit token before evaluating a Lua table constructor. Arbitrary non-static tokens can be collected in that evaluation, before the event dispatcher has a chance to root them. Allocate the token after payload evaluation/secret wrapping instead.

`unit_aura_token_survives_collection_during_payload_construction` forces a complete incremental cycle at the VM allocation safe point (`gc_threshold=0`, `gc_stepmul=1_000_000`) and checks a dynamic token's prefix/numeric suffix, without accidentally rooting its full spelling in Lua constants. Before the fix, delivery fails in `string.sub` on the collected token. This late source change invalidated the earlier full-lib/check/startup proof. Final verification reran those scopes after this fix: 1,972 lib passes/six baseline failures, check/fmt pass, startup `[]`.

## Recorded candidate matrix

41 recorded construction/dispatch candidates: **18 real bugs fixed, 23 false positives left unchanged**. Allocation/execution search intersected 103 Rust files (including tests/helpers); this matrix records the deep classifications, not a claim that every GC lifetime in the simulator is proven safe. Transmog outfit/collection/sets files are excluded.

Raw `state.gc` arena allocation/interning and raw table setters accrue allocation debt; the VM performs collection at GC safe points. In pinned rilua `a76ffa83`, `wrap_secret` authenticates and allocates `Userdata::secret(value)` without executing Lua or collecting. Thus raw-only DTO construction is not equivalent to construction across a Lua callback.

### Fixed sites

Paths are relative to `src/`; line numbers at final Rust revision `0d2e4bfca`. Tooltip test names below are in `tests/tooltip_gc_rooting.rs`; event payload tests are in `tests/event_gc_rooting.rs`.

| Site | Rooting fix | Regression test |
|---|---|---|
| `c_api/c_allied_races.rs:69` | Callback before DTO allocations | `race_info_survives_collection_in_color_callback` |
| `c_api/c_artifact_ui/helpers.rs:102` | Root DTO; attach each color before next callback | `artifact_art_survives_collection_in_each_color_callback` |
| `c_api/c_major_factions.rs:111` | Root DTO; attach strings before callback | `faction_data_survives_collection_in_color_callback` |
| `lua_api/globals/missing_surface/tooltip_info/builders.rs:218` | Root segment array; attach entry before callback | `markup_segments_and_shapeshift_parent_survive_color_collection` |
| `lua_api/globals/missing_surface/tooltip_info/spell.rs:204` | Root spell parent | `spell_parent_survives_color_collection` |
| `lua_api/globals/missing_surface/tooltip_info/spell.rs:218` | Root aura parent | `aura_parent_survives_color_collection` |
| `lua_api/globals/missing_surface/tooltip_info/spell.rs:236` | Root toy parent | `toy_parent_survives_color_collection` |
| `lua_api/globals/missing_surface/tooltip_info/spell.rs:289` | Root mount parent | `mount_parent_survives_markup_color_collection` |
| `lua_api/globals/missing_surface/tooltip_info/spell.rs:97` | Root action parent during binding append | `action_parent_survives_binding_color_collection` |
| `lua_api/globals/missing_surface/tooltip_info/unit.rs:175` | Root unit parent | `unit_parent_survives_color_collection` |
| `lua_api/globals/missing_surface/tooltip_info/sources.rs:89` | Root currency parent | `currency_parent_survives_color_collection` |
| `lua_api/globals/missing_surface/tooltip_info/probes.rs:224` | Root pet parent | `pet_parent_survives_markup_color_collection` |
| `lua_api/globals/missing_surface/tooltip_info/probes.rs:509` | Push achievement result before line fill | `achievement_parent_survives_markup_color_collection` |
| `lua_api/globals/missing_surface/tooltip_info/probes.rs:603` | Push shapeshift result before line fill | `markup_segments_and_shapeshift_parent_survive_color_collection` |
| `lua_api/env_events.rs:233` | Allocate unit token after collecting Lua payload construction | lib `unit_aura_token_survives_collection_during_payload_construction` |
| `lua_api/env_events.rs:245` | Root payload across all listeners/error handlers | `host_event_payload_survives_collection_in_error_handler`; lib `completion_arguments_survive_collection_in_stop_error_handler` |
| `lua_api/script_helpers/event_dispatch.rs:50` | Root native payload across dispatch/error handlers | `native_aura_event_payload_survives_collection_in_error_handler` |
| `lua_api/loader_env.rs:151` | Root loader payload across dispatch/error handlers | `loader_event_payload_survives_collection_in_error_handler` |

### False positives

No changes made at these sites.

| Site (relative to `src/`) | Why not this bug |
|---|---|
| `lua_api/globals/missing_surface/tooltip_info/builders.rs:32` `push_tooltip_line` | Already roots lines and attaches the line before callback in `16682b415`. |
| `lua_api/globals/missing_surface/tooltip_info/builders.rs:576` `populate_item_tooltip_lines` | Already roots the item parent in `16682b415`. |
| `lua_api/globals/missing_surface/tooltip_info/builders.rs:250` `empty_tooltip` | Raw-only allocations and writes, no VM callback/safe point. |
| `lua_api/globals/missing_surface/tooltip_info/probes.rs:295` `c_tooltip_get_minimap_mouseover` | Fixed plain strings without markup or explicit color; no color callback. |
| `c_api/c_artifact_ui/helpers.rs:135` `build_power_info_table` | Vector helper builds native tables, not Lua mixins. |
| `c_api/c_allied_races.rs:119` `build_sequence` | Current converters only perform raw allocation/writes. |
| `c_api/c_major_factions.rs:90` `get_renown_levels` | Entry conversion only performs raw table writes. |
| `c_api/c_encounter_warnings.rs:59` `preview` | Color callback precedes DTO allocation; no later Lua callback. |
| `c_api/c_encounter_timeline/visuals.rs:13` `color` | Color result immediately pushed; no earlier host-created DTO. |
| `c_api/c_aura_container_util.rs:453` `build_rooted_table` | Output already stack-rooted across fill callbacks and errors. |
| `lua_api/globals/auras.rs:855` `build_aura_table` | Identity/flags/points use raw-only construction. |
| `c_api/unit_aura_access.rs:78` `finish_aura_data` | Pinned secret wrapper allocation does not execute Lua/collect; wrapper traces payload. |
| `c_api/c_unit_aura_dispel_color.rs:23` `get_aura_dispel_type_color` | Inputs and evaluated color already explicitly rooted. |
| `c_api/c_scenario_info.rs:44` `get_display_info` | Color callback first; result rooted before native info construction. |
| `c_api/abbreviated_number_formatter.rs:164` `format_number` | New string pushed before wrapping; format calculation is native. |
| `c_api/seconds_formatter.rs:188` `register` | Compiled function pushed before callback allocation; call arguments rooted before execution. |
| `c_api/private_aura_anchors.rs:109` `dispatch` | Payload explicitly pushed throughout callback. |
| `lua_api/globals/real/event_callbacks.rs:174` `dispatch_event_callbacks` | Payload and both immutable snapshots already rooted. |
| `lua_api/string_format.rs:93` `wow_string_format` | Original inputs on incoming stack; new format passed directly as rooted call argument. |
| `loader/addon_modules.rs:97` `finish_file` | Return value explicitly rooted before registry publication. |
| `lua_api/globals/create_frame/template_chain/assignment_builders.rs:60` `build_assignment_handler` | Builder runs after function/literal arguments are on VM stack; no intermediate Lua call. |
| `lua_api/frame/methods/widgets/message_frame/transform.rs:45` `transform_messages` | Native snapshots; fresh args rooted by call helper; results decoded before next callback. |
| `lua_api/frame/methods/widgets/texture/rotation_mask.rs:125` `set_visuals` | Override receives incoming stack args; native DTO branch invokes no Lua callback. |

## Proof ledger

Final integrated Rust revision `0d2e4bfca`; per-subsystem revisions recorded below. Documentation-only follow-ups do not invalidate proof. All tests/check/builds used local host, default retail debug, existing `target/`.

Command prefix for tests: `python3 /home/osso/.worktrees/wow-ui-sim-gc-rooting-audit/scripts/build-host.py --build-host local --test`.

| Scope/tail | Result |
|---|---|
| `--test integration c_allied_races_globals:: -- --nocapture` | 13/13; unchanged C API source since `7f6d28454`. |
| `--test integration c_artifact_ui_panel:: -- --nocapture` | 42/42; unchanged C API source since `7f6d28454`. |
| `--test integration c_major_factions_globals:: -- --nocapture` | 14/14; unchanged C API source since `7f6d28454`. |
| `--test integration tooltip_gc_rooting:: -- --nocapture` | 10/10 at `42d257700`. |
| `--test integration event_gc_rooting:: -- --nocapture` | 3/3 at `0d2e4bfca`. |
| `--test integration tooltip_item_context:: -- --nocapture` | 25/25. |
| `--test integration tooltip_spell_mount_identifiers:: -- --nocapture` | 22/22. |
| `--test integration admin_event_api:: -- --nocapture` | 19/19. |
| `--test integration patch_12_1_0_aura_secret_context:: -- --nocapture` | 4/4 at `0d2e4bfca`. |
| `--test integration aura_table_shape:: -- --nocapture` | 7/7. |
| `--lib -- lua_api::cast_completion:: --nocapture` | 13/13, including original duration test and corrected GUID forced-GC regression. |
| `--lib -- unit_aura_token_survives --nocapture` | 1/1 at `0d2e4bfca`; failed on pre-fix code under forced incremental collection. |
| `--lib` (final integration) | 1,972 pass / six fail, 29.42s, at `0d2e4bfca`. Earlier run at `42d257700` was superseded by the late unit-token source fix; no redundant rerun on unchanged code. |
| Six failing lib tests on baseline | All six fail identically with `src/` byte-identical to master base `16682b415`; original sources restored afterward. |
| `cargo fmt --manifest-path <worktree>/Cargo.toml -- --check` | Exit 0 at `0d2e4bfca`. |
| `python3 <worktree>/scripts/build-host.py --build-host local --check` | Exit 0 at `0d2e4bfca`; six pre-existing iced manifest warnings, no new code warnings. |
| Local debug wow-sim build, then `timeout 90 python3 <worktree>/scripts/build-host.py --build-host local --no-build --run -- --no-addons --no-saved-vars lua-errors` | Exit 0 at `0d2e4bfca`, startup `[]`. |

Full-lib failures: `test_patch_12_0_0_transmog_situation_enum_values`, `installs_debug_environment_defaults`, `installs_seeded_housing_catalog_surface`, `apply_system_anchors_replays_player_frame_size_without_cast_bar_side_effect`, `apply_system_anchors_falls_back_to_minus_one_for_nil_singletons`, `apply_system_anchors_batches_compact_unit_frame_startup_refreshes`. Out of scope; unchanged.

Baseline negatives: three C DTO tests fail with collected/invalid tables; ten tooltip calls fail before roots; host/native/loader event tests lose payloads in collecting error handlers; corrected dynamic-GUID cast test fails against baseline host dispatch. Action test explicitly seeds slot 5 as spell 19750 and uses exactly representable green channels, collecting only in binding append. Initial unseeded action/incorrect aura fixtures and a GUID-pinning literal were corrected, not counted as valid RED proof.

Baseline comparison is master **base `16682b415`**, not subsequently advanced live master (`6899aa881` observed during work). No other checkout, excluded source, or coverage ledger was modified; no push/merge/agents.

Readability review: changed root scopes have explicit balanced restoration and shallow control flow; no warning suppression or generalized rooting abstraction added. Existing broad helpers retain their prior structure.

## Sources

- Commit `16682b415` — established stack/parent rooting idiom.
- `Cargo.toml` pins rilua `a76ffa83f548da94b72777c96d51de38b7f9ef3c`; inspected pinned `src/table_security.rs::wrap_secret` and `src/vm/gc/collector.rs` memory-accounting contract to distinguish arena allocations from collecting VM safe points.
- [Allied races](../../../src/c_api/c_allied_races.rs)
- [Artifact art](../../../src/c_api/c_artifact_ui/helpers.rs)
- [Major factions](../../../src/c_api/c_major_factions.rs)

## See Also

- [Lua API](../../lua-api.md)
- [Event system](../../event-system.md)
