# Downloaded Forever addon runtime coverage

Overall compatibility remains **unverified**. The [per-project matrix](../data/forever-addon-audit/runtime-coverage.json) covers all 269 projects in the [cached archive inventory](../data/forever-addon-audit/cached-comparisons.json), not just previously selected candidates. The earlier [comparison audit](forever-addon-comparison.md) records static evidence; it is not runtime acceptance.

## Inventory — 2026-09-22

All 268 successfully downloaded Forever packages exist and match their indexed sizes and SHA-256 hashes. ConsumableTracker's selected Forever file `8924598` is unavailable; its older comparison archive is not silently substituted. Missing cached dependencies remain separate blockers. No further downloads are authorized by this audit.

The matrix separates current startup and interaction results from `priorEvidence`. A `not-run` current result does not erase an earlier scoped proof: it means that proof has not yet been reconciled or rerun for this inventory-wide pass. Current bounded batches add Abattis PugBoard open/close, Abgesattelt DB/event initialization, three clean startup observations, one intentional LoadOnDemand-not-requested module, one profile-excluded package, and one concrete ActionBarAuras startup failure. Ellesmere's existing six-group proof is retained. EpicDamageMeter also has historical bounded interaction proof; CooldownMaster and DragonGuildMaster have startup-only evidence. Carbonite and Baganator have recorded failures or dependency blockers.

## Acceptance discipline

- Verify that intended addon roots actually load; empty error JSON with nothing loaded is not a pass.
- Separate clean startup, LoadOnDemand/excluded roots, missing dependencies, runtime failures, and incomplete observations.
- Define addon-specific major actions and assert resulting state. Merely finding a slash command, frame, or library is not interaction proof.
- Preserve exact package hashes, simulator revision/binary identity, commands, complete outputs, and proof scope. Reuse prior evidence only when relevant behavior is unchanged.
- Keep goal open while required projects or major workflows fail or remain untested. No full native-conformance claim follows from bounded simulator tests.

Initial archive proof: `/tmp/forever-addon-audit/selected-archive-hash-proof.json`. Prior runtime evidence reconciliation: `/tmp/forever-addon-audit/runtime-proof-inventory.json`. Detailed per-run artifacts remain under `/tmp/forever-addon-runtime/`; the tracked matrix records their paths and dispositions.
