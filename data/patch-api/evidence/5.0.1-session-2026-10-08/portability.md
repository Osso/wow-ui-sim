# 5.0.1 historical proof portability

`rebase-mapping.json` pins the four Git labels read by this session's validator:

| Historical label | Reachable revision |
| --- | --- |
| `83e5d3c3b2643a360bf32e73f343faa987eeb39b` (base) | `60c203907` |
| `f86fe61ebef0c9cbcff753ae83ef607656e736da` (source) | `535723388` |
| `59140d35dc3a63ea2e9cb771029c5f49641e4e5b` (proof) | `5fcf45981` |
| `e1653e5c79bcbdb1844a406ea5e332fef04afbe4` (evidence/documentation) | `582777f0` |

Full hashes live in the mapping. These are checked root-tree reconstructions,
not claims that rebased runtime or evidence equals the historical tree.
`ProofGit` checks reachable ancestry and reconstructs each original tree hash
from the reachable tree, compact exclusions and exact overrides. Unreachable
override blobs are stored as gzip archives and checked by original Git blob ID.
No historical commit/tree objects are imported and no alternate lookup path is
added. The validator installs `ProofGit` before any historical Git reads.

`p501-context.json`, source responses, logs and receipts retain their original
bytes and identities. `historical-validator.py.txt` preserves the original
validator under its existing context seal. `pins.json` separately seals the
adapted validator, historical validator, original context, mapping, this note
and archives. Historical invariant checks remain unchanged apart from the
validator seal now checking the preserved historical validator.

Run only the own validator:

```text
python3 -B data/patch-api/evidence/5.0.1-session-2026-10-08/validate.py
```

This validates recorded historical evidence; it does not rerun Rust, source
reproduction commands, other validators, or shared acceptance gates. The page
is redirect-only with zero inventory and no positive API proof. Three inherited
extract failures remain historical limitations. Current merged 5.0.4 destination
coverage and current runtime behavior belong to coordinator verification.
