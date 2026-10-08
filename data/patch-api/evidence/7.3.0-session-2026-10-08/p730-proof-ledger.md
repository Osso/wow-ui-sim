# Patch 7.3.0 proof ledger

All commands ran with explicit owned cwd and target. Historical failures/empty selections carry no acceptance credit. No full integration suite or synchronous poll/wait.

| Proof | Revision | Command | Result | Validity |
|---|---|---|---|---|
| p730-all-sweeps | `c1533a3e6b` | `cargo test --test prefork_full_ui -- publication_sweep` | exit 0; test result: ok. 43 passed; 0 failed; 43 total | accepted |
| p730-bare-model | `c1533a3e6b` | `cargo test --test integration patch_7_3_0_sound_model:: -- --nocapture` | exit 0; test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10665 filtered out; finished in 0.09s | accepted |
| p730-console | `c1533a3e6b` | `cargo test --test prefork_full_ui -- blizzard_console_` | exit 0; test result: ok. 6 passed; 0 failed; 6 total | accepted |
| p730-debug-tools | `c1533a3e6b` | `cargo test --test prefork_full_ui -- blizzard_debug_tools_` | exit 0; test result: ok. 11 passed; 0 failed; 11 total | accepted |
| p730-deprecated-sound | `c1533a3e6b` | `cargo test --test prefork_full_ui -- blizzard_deprecated_sound_` | exit 0; test result: ok. 4 passed; 0 failed; 4 total | accepted |
| p730-discovery-fixed | `d950a4559f` | `cargo test --test prefork_full_ui -- patch_7_3_0` | exit 1; test result: FAILED. 2 passed; 1 failed; 3 total | Actual full-UI sound alias used namespace no-op; request remained None. Fixed by modeled C_Sound.PlaySound. |
| p730-discovery | `01d089fbea` | `cargo test --test prefork_full_ui -- patch_7_3_0` | exit 101;  | Compile error: unsupported u32 Lua eval conversion; no runtime result. |
| p730-extractor-fixtures | `01d089fbea` | `python3 -B tools/test_extract_patch_non_inventory.py` | exit 0;  | accepted |
| p730-format-final | `2148ac3b15` | `cargo fmt --check` | exit 0;  | accepted |
| p730-format | `d950a4559f` | `cargo fmt --check` | exit 0;  | Passing baseline predates runtime source edits; superseded by p730-format-final. |
| p730-generator-fixtures | `01d089fbea` | `python3 -B tools/test_gen_patch_wikitext_register.py` | exit 0;  | accepted |
| p730-mists-check | `d950a4559f` | `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` | exit 0;  | Passing baseline predates runtime fix; invalidated by C_Sound source changes; superseded by p730-mists-final. |
| p730-mists-final | `c1533a3e6b` | `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` | exit 0;  | accepted |
| p730-model-green | `c1533a3e6b` | `cargo test --test prefork_full_ui -- patch_7_3_0` | exit 0; test result: ok. 3 passed; 0 failed; 3 total | accepted |
| p730-negative | `c1533a3e6b` | `cargo test --test prefork_full_ui -- patch_7_3_0_publication_sweep` | exit 1; test result: FAILED. 0 passed; 1 failed; 1 total | negative control expected exit 1 |
| p730-options-fixed | `c1533a3e6b` | `cargo test --test integration patch_12_1_0_struct_shapes::patch_12_1_0_play_sound -- --nocapture` | exit 0; test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10665 filtered out; finished in 0.12s | accepted |
| p730-options | `c1533a3e6b` | `cargo test --test integration patch_12_1_0_struct_shapes::play_sound -- --nocapture` | exit 0; test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 10666 filtered out; finished in 0.00s | Wrong filter selected zero tests; no acceptance credit. Correct nonempty filter passes in p730-options-fixed. |
| p730-prior-validators | `6a7b903b83` | `python3 -B /home/osso/.worktrees/wow-ui-sim-p730-page/data/patch-api/evidence/7.3.0-session-2026-10-08/check_prior_validators.py` | exit 0;  | accepted |
| p730-retail-build | `c1533a3e6b` | `cargo build --bin wow-sim` | exit 0;  | accepted |
| p730-sound-defaults | `c1533a3e6b` | `cargo test --lib sound_driver_defaults::tests:: -- --nocapture` | exit 0; test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1977 filtered out; finished in 0.12s | accepted |
| p730-soundkit | `c1533a3e6b` | `cargo test --test integration soundkit_ig_inventory_rotate_character:: -- --nocapture` | exit 0; test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 10664 filtered out; finished in 0.10s | accepted |
| p730-source-reproduction | `01d089fbea` | `python3 -B /home/osso/.worktrees/wow-ui-sim-p730-page/data/patch-api/evidence/7.3.0-session-2026-10-08/reproduce_sources.py` | exit 0;  | accepted |
| p730-startup | `c1533a3e6b` | `timeout 90 /home/osso/.cache/wow-ui-sim-targets/p730-page/debug/wow-sim --no-addons --no-saved-vars lua-errors` | exit 0;  | accepted |
| p730-targeted-queue | `c1533a3e6b` | `python3 -B /home/osso/.worktrees/wow-ui-sim-p730-page/data/patch-api/evidence/7.3.0-session-2026-10-08/run_targeted_checks.py` | exit 0;  | orchestration receipt only |
| p730-utility-sound | `c1533a3e6b` | `cargo test --test integration utility_api::test_sound_ -- --nocapture` | exit 0; test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 10664 filtered out; finished in 0.10s | accepted |
| p730-validator-fixtures | `01d089fbea` | `python3 -B tools/test_patch_audit_validation.py` | exit 0;  | accepted |
