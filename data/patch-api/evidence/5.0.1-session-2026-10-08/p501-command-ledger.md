# Patch 5.0.1 proof ledger

Scope: literal historical retail redirect, not destination APIs. Base/source receipt commit `83e5d3c3b2643a360bf32e73f343faa987eeb39b`; implementation `29a642ca8`; runner target isolation `f86fe61eb`. Each `.proof.json` records exact argv, revision, committed tree/blob IDs, exit and complete-log seal. No live unrelated file hashes or tree listings.

## Completed at f86fe61eb

- `python3 -B …/run_proof.py p501-reproduction python3 -B …/reproduce_sources.py`: PASS, 68 registers/65 extracts; exact inherited failures 12.0.5/12.0.7 byte mismatch and 12.1.0 unsupported template. Supplemental 5.4.0 diff extract passes. Other source bytes are preserved against the base by the validator.
- `cargo fmt --check`: PASS. Changed Rust sweep manually checked for readability; one literal spec, no nested control flow or warning suppressions.
- Eleven root `tools/test_*.py` programs: PASS, 112 fixtures. Extra `tools/tests/test_gen_patch_12_0_0_register.py`: initial system Python failure (PyYAML absent) retained; `uv run --no-project --with PyYAML python3 -B …` passes 7/7, no code changes. Total 119 passing fixtures.
- `cargo check --no-default-features --features sound,gui,casc,client-mists --tests`: async completed; retained log/receipt. No vendor edits or warning suppression.

## Pending

Own prefork, fabricated-entry control, all retail publication sweeps, seal tamper control and clean/later validator gate. Full suite remains coordinator-only.

## Invalidation

Later validator/evidence/docs-only edits do not intersect committed source/test/tool/Cargo tree identities, so completed proof stays valid. No repeated broad checks merely for a new milestone. The fabricated control changes only a dedicated override input, never committed runtime/source inventory.
