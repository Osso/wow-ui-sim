# Integrated 6.1.0 proof ledger

Verified: 2026-10-08. Runtime/source scope: `59ccbbc6a480f9e85d16e1007df760a8902c9fe0`; comparison master: `787b47591`. All commands execute from the owned p610-page worktree. Cargo target: `/home/osso/.cache/wow-ui-sim-targets/p610-page`. Subsequent evidence/wiki/validator commits do not alter `src`, `tests`, or `tools`; pinned directory trees gate receipt validity.

| Command | Result | Receipt |
|---|---|---|
| `python3 -B integrated/reproduce_sources.py` | 52 registers byte-identical; 49 extracts byte-identical; inherited 12.0.5/12.0.7/12.1.0 failures unchanged | `p610-register-reproduction.json`, `p610-saved-extract-reproduction.json` |
| `cargo test --test prefork_full_ui -- patch_6_1_0_publication_sweep` | 1 passed | `own-sweep.proof.json` |
| `cargo test --test prefork_full_ui -- publication_sweep` | 53 passed | `all-sweeps.proof.json` |
| `cargo test --test prefork_full_ui -- patch_6_1_0` | 1 passed | `prefork-patch.proof.json` |
| `cargo test --test prefork_full_ui -- death_recap` | 8 passed | `prefork-recap.proof.json` |
| `cargo test --test integration c_death_recap_probes` | 6 passed | `integration-recap.proof.json` |
| `cargo test --test integration p1200_removed_plain_globals` | 1 passed | `integration-absence.proof.json` |
| `cargo fmt --check` | exit 0 | `format.proof.json` |
| `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` | exit 0; zero non-vendor warnings, six inherited iced_wgpu manifest warnings | `mists-check.proof.json` |
| `python3 -B tools/test_check_patch_validators.py` | 4 passed | `test_check_patch_validators.proof.json` |
| `python3 -B tools/test_extract_patch_non_inventory.py` | 36 passed | `test_extract_patch_non_inventory.proof.json` |
| `python3 -B tools/test_gen_patch_wikitext_register.py` | 33 passed | `test_gen_patch_wikitext_register.proof.json` |
| `python3 -B tools/test_patch_audit_validation.py` | 8 passed | `test_patch_audit_validation.proof.json` |
| Own sweep with `P610_SWEEP_REGISTER=integrated/negative-register.json` | expected exit 1; gaps 1 → 2; three retained observations identical | `negative.proof.json`, `negative-results.json` |
| `python3 -B integrated/check_prior.py` | 35 pinned-master validators passed; inventory selected by recorded `git ls-tree` | `prior-validator-matrix.json` |
| `python3 -B integrated/check_tamper.py` | altered own historical log rejected; original bytes restored | `tamper-proof.json` |

Commands using `integrated/` resolve under this session directory. Logs are retained in full beside receipts. No runtime source changed; no full integration suite or startup comparison required. Historical receipts and their source scopes are preserved separately, not overwritten by this ledger.
