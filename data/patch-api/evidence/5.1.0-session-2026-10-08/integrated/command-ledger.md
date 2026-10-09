# Integrated 5.1.0 command ledger

Pinned master: `7c3bbbfd311dffc2f2c069ce676f0a8ff8afb87a`. Every invocation uses explicit cwd `/home/osso/.worktrees/wow-ui-sim-p510-page` and that worktree's `target/`. Master was tested by temporarily detaching this same worktree, never changing canonical or sibling files. All source/test proof scopes remain valid after documentation/evidence-only commits.

| Receipt | Command | Revision | Exit | Result |
|---|---|---|---:|---|
| all-sweeps | `cargo test --test prefork_full_ui -- publication_sweep` | `0801851248847212771a23816f3ce11a04c5ec2d` | 0 | 63 passed/0 failed (ok) |
| checks | `python3 -B /home/osso/.worktrees/wow-ui-sim-p510-page/data/patch-api/evidence/5.1.0-session-2026-10-08/integrated/run_checks.py` | `0801851248847212771a23816f3ce11a04c5ec2d` | 0 | see sealed log |
| diagnostic-own-sweep | `cargo test --test prefork_full_ui -- patch_5_1_0_publication_sweep` | `ee9bd455a373b799bad971bb4e32b8177519ebe3` | 0 | 1 passed/0 failed (ok) |
| format | `cargo fmt --check` | `0801851248847212771a23816f3ce11a04c5ec2d` | 0 | see sealed log |
| historical-replay | `python3 -B /home/osso/.worktrees/wow-ui-sim-p510-page/data/patch-api/evidence/5.1.0-session-2026-10-08/validate.py` | `341799ec6a113ad742d6e82da7cd4e70b67b6dc6` | 0 | see sealed log |
| integration-namespace | `cargo test --test integration namespace_stubs_patched -- --test-threads=4` | `0801851248847212771a23816f3ce11a04c5ec2d` | 0 | 2 passed/0 failed (ok) |
| integration-patch_5_1_0 | `cargo test --test integration -- patch_5_1_0` | `0801851248847212771a23816f3ce11a04c5ec2d` | 0 | 1 passed/0 failed (ok) |
| integration-pet-info | `cargo test --test integration pet_info -- --test-threads=4` | `0801851248847212771a23816f3ce11a04c5ec2d` | 0 | 19 passed/0 failed (ok) |
| integration-pet-stats | `cargo test --test integration pet_stats -- --test-threads=4` | `0801851248847212771a23816f3ce11a04c5ec2d` | 0 | 13 passed/0 failed (ok) |
| lib-namespace | `cargo test --lib namespace -- --test-threads=4` | `0801851248847212771a23816f3ce11a04c5ec2d` | 0 | 24 passed/0 failed (ok) |
| master-all-sweeps | `cargo test --test prefork_full_ui -- publication_sweep` | `7c3bbbfd311dffc2f2c069ce676f0a8ff8afb87a` | 0 | 62 passed/0 failed (ok) |
| master-checks | `python3 -B /home/osso/.worktrees/wow-ui-sim-p510-page/target/p510-refresh/master-runner/master_checks.py` | `7c3bbbfd311dffc2f2c069ce676f0a8ff8afb87a` | 0 | see sealed log |
| master-integration-namespace | `cargo test --test integration namespace_stubs_patched -- --test-threads=4` | `7c3bbbfd311dffc2f2c069ce676f0a8ff8afb87a` | 0 | 2 passed/0 failed (ok) |
| master-integration-pet-info | `cargo test --test integration pet_info -- --test-threads=4` | `7c3bbbfd311dffc2f2c069ce676f0a8ff8afb87a` | 0 | 19 passed/0 failed (ok) |
| master-integration-pet-stats | `cargo test --test integration pet_stats -- --test-threads=4` | `7c3bbbfd311dffc2f2c069ce676f0a8ff8afb87a` | 0 | 13 passed/0 failed (ok) |
| master-lib-namespace | `cargo test --lib namespace -- --test-threads=4` | `7c3bbbfd311dffc2f2c069ce676f0a8ff8afb87a` | 0 | 24 passed/0 failed (ok) |
| master-mists-all-sweeps | `cargo test --no-default-features --features sound,gui,casc,client-mists --test integration -- publication_sweep` | `7c3bbbfd311dffc2f2c069ce676f0a8ff8afb87a` | 0 | 6 passed/0 failed (ok) |
| master-prefork-pet | `cargo test --test prefork_full_ui -- pet` | `7c3bbbfd311dffc2f2c069ce676f0a8ff8afb87a` | 0 | 21 passed/0 failed (ok) |
| mists-all-sweeps | `cargo test --no-default-features --features sound,gui,casc,client-mists --test integration -- publication_sweep` | `0801851248847212771a23816f3ce11a04c5ec2d` | 0 | 6 passed/0 failed (ok) |
| mists-check | `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` | `0801851248847212771a23816f3ce11a04c5ec2d` | 0 | see sealed log |
| negative | `cargo test --test prefork_full_ui -- patch_5_1_0_publication_sweep` | `0801851248847212771a23816f3ce11a04c5ec2d` | 1 | 0 passed/1 failed (FAILED) |
| original-pin-independence | `python3 -B /home/osso/.worktrees/wow-ui-sim-p510-page/data/patch-api/evidence/5.1.0-session-2026-10-08/integrated/check_original_pin_independence.py` | `1b2b601cedc20fd96894fd907533174ec7ebba7d` | 0 | see sealed log |
| own-sweep | `cargo test --test prefork_full_ui -- patch_5_1_0_publication_sweep` | `ee9bd455a373b799bad971bb4e32b8177519ebe3` | 0 | 1 passed/0 failed (ok) |
| prefork-patch_5_1_0 | `cargo test --test prefork_full_ui -- patch_5_1_0` | `0801851248847212771a23816f3ce11a04c5ec2d` | 0 | 3 passed/0 failed (ok) |
| prefork-pet | `cargo test --test prefork_full_ui -- pet` | `0801851248847212771a23816f3ce11a04c5ec2d` | 0 | 22 passed/0 failed (ok) |
| reproduction | `python3 -B /home/osso/.worktrees/wow-ui-sim-p510-page/data/patch-api/evidence/5.1.0-session-2026-10-08/integrated/reproduce_sources.py` | `ee9bd455a373b799bad971bb4e32b8177519ebe3` | 0 | see sealed log |
| scan-tool-build | `cargo install ripgrep --locked --root /home/osso/.worktrees/wow-ui-sim-p510-page/target/ripgrep --target-dir /home/osso/.worktrees/wow-ui-sim-p510-page/target/ripgrep-build` | `ee9bd455a373b799bad971bb4e32b8177519ebe3` | 0 | see sealed log |
| test_check_patch_validators | `python3 -B /home/osso/.worktrees/wow-ui-sim-p510-page/tools/test_check_patch_validators.py` | `0801851248847212771a23816f3ce11a04c5ec2d` | 0 | 4 Python fixtures |
| test_extract_patch_non_inventory | `python3 -B /home/osso/.worktrees/wow-ui-sim-p510-page/tools/test_extract_patch_non_inventory.py` | `0801851248847212771a23816f3ce11a04c5ec2d` | 0 | 37 Python fixtures |
| test_gen_patch_wikitext_register | `python3 -B /home/osso/.worktrees/wow-ui-sim-p510-page/tools/test_gen_patch_wikitext_register.py` | `0801851248847212771a23816f3ce11a04c5ec2d` | 0 | 35 Python fixtures |
| test_patch_5_1_register | `python3 -B /home/osso/.worktrees/wow-ui-sim-p510-page/tools/test_patch_5_1_register.py` | `0801851248847212771a23816f3ce11a04c5ec2d` | 0 | 3 Python fixtures |
| test_patch_audit_validation | `python3 -B /home/osso/.worktrees/wow-ui-sim-p510-page/tools/test_patch_audit_validation.py` | `0801851248847212771a23816f3ce11a04c5ec2d` | 0 | 8 Python fixtures |
| test_patch_mists_520_register | `python3 -B /home/osso/.worktrees/wow-ui-sim-p510-page/tools/test_patch_mists_520_register.py` | `0801851248847212771a23816f3ce11a04c5ec2d` | 0 | 2 Python fixtures |
| test_patch_mists_extract | `python3 -B /home/osso/.worktrees/wow-ui-sim-p510-page/tools/test_patch_mists_extract.py` | `0801851248847212771a23816f3ce11a04c5ec2d` | 0 | 1 Python fixtures |
| test_patch_mists_register | `python3 -B /home/osso/.worktrees/wow-ui-sim-p510-page/tools/test_patch_mists_register.py` | `0801851248847212771a23816f3ce11a04c5ec2d` | 0 | 6 Python fixtures |
| test_patch_mists_transclusion | `python3 -B /home/osso/.worktrees/wow-ui-sim-p510-page/tools/test_patch_mists_transclusion.py` | `0801851248847212771a23816f3ce11a04c5ec2d` | 0 | 2 Python fixtures |
| test_patch_warlords_register | `python3 -B /home/osso/.worktrees/wow-ui-sim-p510-page/tools/test_patch_warlords_register.py` | `0801851248847212771a23816f3ce11a04c5ec2d` | 0 | 3 Python fixtures |

## Coverage and limits

- Branch/master retail sweeps 63/62; Mists 6/6 each. All 66 other pages and 10,153 observations equal pinned master.
- Own prefork/integration 3/1; journal/namespace integration 19/13/2, prefork pet 22 branch/21 master, library namespace 24/24.
- All 101 Python fixtures, formatting and Mists tests check pass; zero non-vendor warnings. Inherited iced_wgpu manifest warnings remain untouched.
- All 67 registers/64 main extracts and the supplemental 5.4.0 diff reproduce. Only inherited 12.0.5/12.0.7/12.1.0 extracts fail unchanged.
- All 63 own observations equal historical proof; 17 gaps unchanged, negative control 17 → 18. No fixture or source-ledger reclassification justified.

Retirement scans are retained in `retirement-scans.json`, with eight complete whole-word qualified/bare rg logs. No cache consumers; src/tests hits are only the marker and absence assertions. Fifteen original Git pins can be denied without weakening any historical invariant.

Known host limit: no WoW installation. CASC texture tests are outside this requested bounded run; no asset/host remediation or native-2012/full-integration claim. Historical addons-enabled startup failures remain documented, not erased.
