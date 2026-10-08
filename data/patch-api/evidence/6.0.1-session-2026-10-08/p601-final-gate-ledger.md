# Final validator proof ledger

Extends the frozen [command ledger](p601-command-ledger.md). No source inputs changed after the passing targeted proofs.

- `["python3", "-B", "/home/osso/.worktrees/wow-ui-sim-p601-page/data/patch-api/evidence/6.0.1-session-2026-10-08/validate.py"]` — revision `352ebb471c3b1d5a01fe6e3d57f5734dddb8c744`, exit 0; complete log `p601-own-validator.log`, SHA-256 `94070c62f76cc9ef261e7cde374d043f0d69729409238e4fc5b3806396f23247`. No later proof-scope changes.
- `["python3", "-B", "/home/osso/.worktrees/wow-ui-sim-p601-page/tools/check_patch_validators.py", "352ebb471"]` — revision `352ebb471c3b1d5a01fe6e3d57f5734dddb8c744`, exit 0; complete log `p601-validator-gate.log`, SHA-256 `62019919cdda59ff5a736f6a9415458600695f12d77a82affd3c617e4fd7967d`. No later proof-scope changes.

Gate: 30/30 clean validators; 31/31 after synthetic later-audit changes, including its synthetic validator. Zero failures. Detached gate worktree removed by the gate.

Remaining: three inherited extract failures (12.0.5, 12.0.7, 12.1.0) remain outside this redirect audit; queued-page placeholders are integration work. No own-page gaps, retirements or pending checks.
