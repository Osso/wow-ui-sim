# Saved execution excerpts

## map-probes.stdout
test c_map_probes::get_map_display_info_absence_and_removal_return_zero_values_sim_input_policy ... ok
test c_map_probes::get_map_display_info_returns_one_supplied_bool ... ok
test c_map_probes::get_map_display_info_tainted_caller_accepts_ordinary_and_rejects_secret ... ok
test c_map_probes::get_map_display_info_tracks_updates_with_map_and_environment_isolation ... ok
test c_map_probes::get_map_display_info_untainted_secret_returns_one_supplied_bool ... ok
test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 10673 filtered out; finished in 2.79s

## map-api-controls.stdout
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 10657 filtered out; finished in 3.40s

## lua-errors.stdout
[]

## lua-errors.stderr
Lua errors: 0 unique, 0 occurrence(s)
Unattributed Lua errors: 0 occurrence(s)

## p801-sweep.stdout
test patch_8_0_1_publication_sweep::patch_8_0_1_publication_sweep ... FAILED (panic: assertion `left == right` failed: new gaps: []; resolved/stale gaps: ["wt-global-api-C_Map.GetMapDisplayInfo-30"]
test result: FAILED. 0 passed; 1 failed; 1 total
test result: ok. 0 passed; 0 failed; 0 total
test result: ok. 0 passed; 0 failed; 0 total
test result: ok. 0 passed; 0 failed; 0 total

## p801-sweep.stderr
assertion `left == right` failed: new gaps: []; resolved/stale gaps: ["wt-global-api-C_Map.GetMapDisplayInfo-30"]
