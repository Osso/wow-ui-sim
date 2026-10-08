# 6.2.4 proof ledger
All commands ran from the p624-page checkout. Cargo uses the dedicated external target; historical receipts contain the invocation revision and log hash. Runtime/test source is unchanged after a690a6959. No full suite or startup parity claim.
| Scope | Revision | Exit | Command | Log SHA-256 |
|---|---|---|---|---|
| p624-all-sweeps | a690a6959a26c345730407f5c92716e9cca4bded | 0 | `cargo test --test prefork_full_ui -- publication_sweep` | `a1a4f47ad60a5ea163f59f028fd7c41ecd549752b13404d25855ec851bb0c8cf` |
| p624-bare-identity | a690a6959a26c345730407f5c92716e9cca4bded | 0 | `cargo test --test integration patch_6_2_4` | `faaf2204411d28e8189ab49c5157e7be801bb6c4ac18a6461409d9dddd88d6fe` |
| p624-behavior-format-dev | aa6f8163f5f53bfff834bc01e09fb4a972746858 | 0 | `cargo fmt` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| p624-bnet-model | a690a6959a26c345730407f5c92716e9cca4bded | 0 | `cargo test --test integration c_battle_net_probes::` | `9da774abbd8a60371c3b607da0c1f85d4ceeefeac280475b57df121dd1da9cd9` |
| p624-cached-identity | a690a6959a26c345730407f5c92716e9cca4bded | 0 | `cargo test --test prefork_full_ui -- patch_6_2_4_cached` | `63fa87fca55560315e66a21f6733bcdeae9565b6d934bff519441816a993e094` |
| p624-deprecated-bnet | a690a6959a26c345730407f5c92716e9cca4bded | 0 | `cargo test --test prefork_full_ui -- blizzard_deprecated_battle_net` | `671cf314227df0553dc235f9cf4ecb778d39286f1881cc3a0a87e8ceb624c077` |
| p624-discovery | aa6f8163f5f53bfff834bc01e09fb4a972746858 | 0 | `cargo test --test prefork_full_ui -- patch_6_2_4` | `e914c2bd611c4ca0474e481a2e69124b9d36faead19c1c9201fb164be10c6c34` |
| p624-extractor-fixtures-final | a690a6959a26c345730407f5c92716e9cca4bded | 0 | `python3 -B tools/test_extract_patch_non_inventory.py` | `68a6a4cc70869aba044e172a4c8b166706d7bd826ddd99a580193e7092813d9c` |
| p624-extractor-fixtures | aa6f8163f5f53bfff834bc01e09fb4a972746858 | 1 | `python3 -B tools/test_extract_patch_non_inventory.py` | `a780e2e5ced0c5be97b24e5a910c51354993f1e7c8a9564a6c23b1d08b831695` |
| p624-final-format | a690a6959a26c345730407f5c92716e9cca4bded | 0 | `cargo fmt --check` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| p624-format-dev | 25fbde058514034692df481bd568e952a2b9851e | 0 | `cargo fmt` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| p624-generator-fixtures | aa6f8163f5f53bfff834bc01e09fb4a972746858 | 0 | `python3 -B tools/test_gen_patch_wikitext_register.py` | `aa9de8e6336c91276985e8b7edfd74ee4053e151c065e21365799c14cea15e97` |
| p624-mists-check | a690a6959a26c345730407f5c92716e9cca4bded | 0 | `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` | `009ea71fea3cffc444c11f45b9fc49bb441d161df67ab9a16620d48e8992f979` |
| p624-negative | a690a6959a26c345730407f5c92716e9cca4bded | 1 | `cargo test --test prefork_full_ui -- patch_6_2_4_publication_sweep` | `a6a1bab9c7b06215b7bae7331141ae574a0ebab4dd230bd106cb889c4c0affb2` |
| p624-other-validators | a690a6959a26c345730407f5c92716e9cca4bded | 0 | `python3 -B /home/osso/.worktrees/wow-ui-sim-p624-page/data/patch-api/evidence/6.2.4-session-2026-10-08/check_other_validators.py` | `bb9d72e1ecf2ce6563ab4a9b1451213f9e3cfc196ff22c1298c98e333c168918` |
| p624-source-reproduction | aa6f8163f5f53bfff834bc01e09fb4a972746858 | 0 | `python3 -B /home/osso/.worktrees/wow-ui-sim-p624-page/data/patch-api/evidence/6.2.4-session-2026-10-08/reproduce_sources.py` | `e55183daaf12cad7bd02034a3bce5bdfd54083b6deaff5f6e13f53375d406193` |
| p624-targeted-driver | a690a6959a26c345730407f5c92716e9cca4bded | 0 | `python3 -B /home/osso/.worktrees/wow-ui-sim-p624-page/data/patch-api/evidence/6.2.4-session-2026-10-08/verify_targeted.py` | `d124bc54de18fe4716ccd9770a3d9e98287d67ac3e2e2575eb230a995f33f7f5` |
| p624-validator-fixtures | aa6f8163f5f53bfff834bc01e09fb4a972746858 | 0 | `python3 -B tools/test_patch_audit_validation.py` | `87f7147fd5cb582ac56150ae191a8503a48bf6f56c729357de454e3ecf0bd31c` |

Initial extractor fixture failure is superseded by p624-extractor-fixtures-final after correcting expected heading spacing. Original failed proof/log retained. RED parser/extractor fixtures remain separate failure artifacts; final positive receipts do not relabel them.
