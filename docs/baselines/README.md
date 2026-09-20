# Profile Baselines

Validation artifacts for supported client profiles.

## Lua Errors

`mists-lua-errors.json` is the committed clean startup baseline for:

```bash
cargo build --no-default-features --features "sound,gui,casc,client-mists" --bin wow-sim
WOW_SIM_NO_SAVED_VARS=1 WOW_SIM_NO_ADDONS=1 timeout 90 ./target/debug/wow-sim lua-errors > docs/baselines/mists-lua-errors.json
```

Do not refresh this file to bless new startup errors. Refresh only after a
verified clean Mists capture.

Use `scripts/diff-lua-errors.sh BASELINE NEW` to compare message sets and
report regressed/fixed errors. The Mists CI guard uses this for addon-induced
error counts and release proof diagnostics.

## Forever Lua Errors

`wowforever-lua-errors.json` records the first `client-wowforever` startup capture after commit `65139f909` synchronized all 4,398 manifest files from local `wow_classic_beta` CASC. It contains 422 distinct error records with 514 occurrences; the process exited 1. Preserve it as the initial failure baseline, not an accepted clean startup.

## Panel And Addon Evidence

- `mists-panels.md` lists the Blizzard panel parity matrix.
- `mists-panel-interactions.md` records interaction audit coverage.
- `mists-panel-visuals.tsv` records visual metric samples.
- `mists-test-coverage.md` maps parity rows to focused Rust tests.
- `mists-release-proof.md` records the full local release proof command.
- `classic-addon-test-targets.md` lists the installed Mists addon matrix.
- `mists-lod-audit.md` records LoD addon coverage.
