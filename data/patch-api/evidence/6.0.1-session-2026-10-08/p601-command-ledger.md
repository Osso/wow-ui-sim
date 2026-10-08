# Proof ledger

Every command ran in the owned worktree with its own CARGO_TARGET_DIR. Complete logs and SHA-256 receipts are retained. No shell, rg, fd or sd used.

## Initial failure and invalidation

Initial discovery and all-sweep receipts are under `initial-failure/`. Both failed because the own expected-gap fixture was a JSON map rather than the required sequence. Commit `4b6c582c7` corrects only that fixture. This invalidates initial sweep proof. Scoped Python fixture/reproduction proof and Rust formatting are unaffected; their inputs did not change. Mists and both sweep filters are rechecked after correction.

## Completed independent checks

- `["cargo", "fmt", "--check"]` — revision `e78b6376a14e616bca0e5cd0565aee1f7406995e`, exit 0; log `p601-format-check.log`, SHA-256 `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`. Scope remains valid after JSON-fixture correction.
- `["python3", "-B", "/home/osso/.worktrees/wow-ui-sim-p601-page/data/patch-api/evidence/6.0.1-session-2026-10-08/reproduce_sources.py"]` — revision `e78b6376a14e616bca0e5cd0565aee1f7406995e`, exit 0; log `p601-reproduction.log`, SHA-256 `5ed07f963829f5128412b7976412c321aa2c9bbedcf4e78eae83b192c711c9da`. Scope remains valid after JSON-fixture correction.
- `["python3", "-B", "/home/osso/.worktrees/wow-ui-sim-p601-page/tools/test_check_patch_validators.py"]` — revision `e78b6376a14e616bca0e5cd0565aee1f7406995e`, exit 0; log `p601-test_check_patch_validators.log`, SHA-256 `eca694caa756f4acdd16c1a74180847ae5ccd4e86277ec65f7b078daaea2da3f`. Scope remains valid after JSON-fixture correction.
- `["python3", "-B", "/home/osso/.worktrees/wow-ui-sim-p601-page/tools/test_extract_patch_non_inventory.py"]` — revision `e78b6376a14e616bca0e5cd0565aee1f7406995e`, exit 0; log `p601-test_extract_patch_non_inventory.log`, SHA-256 `89ca58b296d9868916ccda25250eba5f1d80fca5f72e6396ab35144ae1333e05`. Scope remains valid after JSON-fixture correction.
- `["python3", "-B", "/home/osso/.worktrees/wow-ui-sim-p601-page/tools/test_gen_patch_wikitext_register.py"]` — revision `e78b6376a14e616bca0e5cd0565aee1f7406995e`, exit 0; log `p601-test_gen_patch_wikitext_register.log`, SHA-256 `c8915683650f2fa4dee526b38a5b0820e4a36b801288e67dfe93e7d4f42ce707`. Scope remains valid after JSON-fixture correction.
- `["python3", "-B", "/home/osso/.worktrees/wow-ui-sim-p601-page/tools/test_patch_audit_validation.py"]` — revision `e78b6376a14e616bca0e5cd0565aee1f7406995e`, exit 0; log `p601-test_patch_audit_validation.log`, SHA-256 `48362aa7a7a003acb743d7bb61c1c6002287913e3656efdd36bcc2241a579362`. Scope remains valid after JSON-fixture correction.
