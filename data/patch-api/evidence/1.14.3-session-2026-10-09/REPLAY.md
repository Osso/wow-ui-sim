# Historical SOURCE replay

`replay.tar.gz` contains55 files:54 immutable sealed inputs plus their original `seals.json`. Extract into a fresh disposable directory with no Git/target/current tools. `audit.py` resolves every input relative to its own copied file; no repository or network dependency. Run absolute copied paths via Python `-B` with explicit owned worktree cwd. Empty PATH proves Git/current command independence.

`test_portable.py` already exercised copied `audit.py`, copied SOURCE8 and copied default generator byte reproduction, then serialized ledger/log omission/fabrication rejection and exact restoration. Exact subprocess commands/outputs/restoration hashes live in `portable-controls.json`; no need to repeat unchanged proof.

Original54 seals remain immutable. Later factory observations, Rust target snapshot, reviewed mismatches and receipt logs are not retroactive SOURCE credit; see `receipt-seals.json` and `HANDOFF.md`. SOURCE original ledger retains all substantive contracts UNPROVEN and zero runtime/model/native measurements.
