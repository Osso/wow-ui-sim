# 7.2.0 integrated command ledger

Runtime/register/sweep scope: `0d92f4dbb76f1f85b8bee4b8573135c95316d0de`. Exact archived master: `8b6131f3645bb02d83f4c608cb7181139901cc93`.
Every command uses cwd `/home/osso/.worktrees/wow-ui-sim-p720-page` and CARGO_TARGET_DIR=/home/osso/.cache/wow-ui-sim-targets/p720-page. Source reproduction records its actual driver revision separately. Later driver commits change only evidence/docs, not the hashed runtime/tool scope.

| Receipt | Driver revision | Source revision | Command | Exit | Result |
|---|---|---|---|---|---|
| all-sweeps.proof.json | 1aeeb9f0c | 0d92f4dbb | `cargo test --test prefork_full_ui -- publication_sweep` | 0 |     Finished `test` profile [optimized + debuginfo] target(s) in 2m 40s; test result: ok. 46 passed; 0 failed; 46 total |
| equipment-lib.proof.json | cdc920183 | 0d92f4dbb | `cargo test --lib -- wow_api_equipment_set::` | 0 |     Finished `test` profile [optimized + debuginfo] target(s) in 59.81s; test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 1972 filtered out; finished in 0.38s |
| equipment-regression.proof.json | 1aeeb9f0c | 0d92f4dbb | `cargo test --test integration -- equipment_set` | 0 |     Finished `test` profile [optimized + debuginfo] target(s) in 0.17s; test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 10664 filtered out; finished in 0.15s |
| extend-receipts.proof.json | 1aeeb9f0c | 0d92f4dbb | `python3 -B /home/osso/.worktrees/wow-ui-sim-p720-page/tools/extend_patch_audit_receipts.py /home/osso/.worktrees/wow-ui-sim-p720-page /home/osso/.worktrees/wow-ui-sim-p720-page/data/patch-api/evidence/7.2.0-session-2026-10-08/integrated/extension p720 'Merged 7.2.5 at 8b6131f3645bb02d83f4c608cb7181139901cc93' 7.2.0` | 0 | exit 0 |
| extract_patch_non_inventory-fixtures.proof.json | cdc920183 | 0d92f4dbb | `python3 -B /home/osso/.worktrees/wow-ui-sim-p720-page/tools/test_extract_patch_non_inventory.py` | 0 | Ran 34 tests in 0.081s |
| format.proof.json | cdc920183 | 0d92f4dbb | `cargo fmt --check` | 0 | exit 0 |
| gen_patch_wikitext_register-fixtures.proof.json | cdc920183 | 0d92f4dbb | `python3 -B /home/osso/.worktrees/wow-ui-sim-p720-page/tools/test_gen_patch_wikitext_register.py` | 0 | Ran 29 tests in 0.004s |
| integration-p720.proof.json | 1aeeb9f0c | 0d92f4dbb | `cargo test --test integration -- patch_7_2_0` | 0 |     Finished `test` profile [optimized + debuginfo] target(s) in 35.17s; test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 10670 filtered out; finished in 0.00s; ZERO CASES, no coverage credited |
| mask-regression.proof.json | 1aeeb9f0c | 0d92f4dbb | `cargo test --test integration -- methods_texture::masks_and_misc::` | 0 |     Finished `test` profile [optimized + debuginfo] target(s) in 0.16s; test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 10660 filtered out; finished in 0.18s |
| master-all-sweeps.proof.json | 1aeeb9f0c | 8b6131f36 | `cargo test --manifest-path /home/osso/.cache/wow-ui-sim-targets/p720-page/master-8b6131f36/Cargo.toml --test prefork_full_ui -- publication_sweep` | 0 |     Finished `test` profile [optimized + debuginfo] target(s) in 2m 39s; test result: ok. 45 passed; 0 failed; 45 total |
| mists-check.proof.json | 3a572cbdd | 0d92f4dbb | `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` | 0 |     Finished `dev` profile [optimized + debuginfo] target(s) in 41.72s |
| negative.proof.json | 3a572cbdd | 0d92f4dbb | `env P720_SWEEP_REGISTER=/home/osso/.worktrees/wow-ui-sim-p720-page/data/patch-api/evidence/7.2.0-session-2026-10-08/p720-negative-register.json P720_SWEEP_OUT=/home/osso/.worktrees/wow-ui-sim-p720-page/data/patch-api/evidence/7.2.0-session-2026-10-08/integrated/negative-results.json cargo test --test prefork_full_ui -- patch_7_2_0_publication_sweep` | 1 |     Finished `test` profile [optimized + debuginfo] target(s) in 0.13s; test result: FAILED. 0 passed; 1 failed; 1 total |
| own-sweep.proof.json | 1aeeb9f0c | 0d92f4dbb | `cargo test --test prefork_full_ui -- patch_7_2_0_publication_sweep` | 0 |     Finished `test` profile [optimized + debuginfo] target(s) in 3m 38s; test result: ok. 1 passed; 0 failed; 1 total |
| patch_audit_validation-fixtures.proof.json | cdc920183 | 0d92f4dbb | `python3 -B /home/osso/.worktrees/wow-ui-sim-p720-page/tools/test_patch_audit_validation.py` | 0 | Ran 8 tests in 0.340s |
| prefork-p720.proof.json | 1aeeb9f0c | 0d92f4dbb | `cargo test --test prefork_full_ui -- patch_7_2_0` | 0 |     Finished `test` profile [optimized + debuginfo] target(s) in 0.16s; test result: ok. 4 passed; 0 failed; 4 total |
| texture-regression.proof.json | 1aeeb9f0c | 0d92f4dbb | `cargo test --test integration -- texture_methods_port::` | 0 |     Finished `test` profile [optimized + debuginfo] target(s) in 0.16s; test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 10645 filtered out; finished in 0.37s |

## Sources and gaps

45/45 registers and 42/45 extracts reproduce with recorded/inherited flags. The exact inherited failures remain: 12.0.5 and 12.0.7 output mismatches; 12.1.0 unsupported #description2 template. Per-row flags, hashes and errors are retained in the reproduction files. The shared receipt extender adds the real 7.2.5 register and its three sweep rows (three known gaps). Added receipt rows retain their template revision and stamp the actual extension driver revision.

Exact master: 44 pages plus factory, 45/45 passing cases, 8,869 observations. Branch: 45 pages plus factory, 46/46 passing cases, 8,872 observations. All 44 existing pages have identical row IDs, per-row ok status and known-gap sets versus exact master. Own gap set remains zero. No supersession closures, new gaps or LATER_AUDIT_REPLACEMENTS changes. Full per-page comparison is gap-comparison.json.

Negative control changes exactly wt-global-api-C_EquipmentSet-10: 0 -> 1 gap; expected exit 1, no resolved/stale IDs. Neither known gaps nor probe invariants were relaxed.

## Coverage and warnings

Requested integration patch_7_2_0 filter selects zero cases: this page declares cached prefork cases only, so no bare page-specific coverage is credited. Requested prefork patch_7_2_0 passes 4/4. Texture/mask/equipment integration filters pass 25/10/6; equipment library passes 7/7. Python fixtures pass 29/34/8. Format and requested Mists check pass; zero non-vendor warnings, six inherited iced manifest deprecations remain unsuppressed.

No src, Cargo, build, vendor, Interface or profile-manifest changes versus master. No new runtime deployment or startup claim; original separately archived startup evidence remains sealed. No agents, push, merge or directory switching. No __pycache__.

## Validator scope

Historical 7.2.0 scope remains fixed at 8ae333df454e4f0d5d4bf08cd9e6b9d90f270da7 (44 registers/45 sweep cases); all 155 originally sealed artifacts remain byte-identical. Integrated scope is separately pinned at the runtime revision above (45 registers/46 sweep cases). All 22 required evidence/*/validate.py scripts passed with unchanged covered inputs; prior-validator-matrix.json records their exact commands, revisions, validator hashes, logs and exits. Final integrated gate is recorded separately in validator-matrix.json.
