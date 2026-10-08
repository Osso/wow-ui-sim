# Final portable verification

Revision `076a89d78aec05e42557d917d931255882089752`.

- `python3 -B tools/check_patch_validators.py`: exit 0, PASS; clean 35/35, unrelated later-audit 36/36. Full output: `portability.txt`; hashed command receipt: `portability.proof.json`.
- `python3 -B data/patch-api/evidence/6.0.2-session-2026-10-08/integrated/check_tamper.py`: exit 0; historical and integrated own-log tampering rejected, bytes restored. Details: `tamper-proof.json`.
- Prior matrix: 37/37 validators from git ls-tree at d0fabed03, including four validate_integrated.py gates not discovered by the two-phase tool.
- Saved-input reproduction: `python3 -B data/patch-api/evidence/6.0.2-session-2026-10-08/integrated/reproduce_sources.py`, exit 0 at 7e670b48c; 53 registers/50 extracts, three unchanged inherited failures.

All Cargo commands use `CARGO_TARGET_DIR=/home/osso/.cache/wow-ui-sim-targets/p602-page`; main commands execute from `/home/osso/.worktrees/wow-ui-sim-p602-page`. Exact runtime/check commands and results are in `command-ledger.md`. Final receipt/wiki changes do not alter tested src/tests/tools or sealed validator inputs.

## Integration commits before this receipt commit

```text
7e670b48c Add integrated 6.0.2 reproduction and file-backed proof drivers
89d359e65 Define integrated sweep parity and scenario regression proof scope
3167253bd Preserve 6.0.2 historical receipts and map rebased input blobs
5a99f401f Map queued 6.1.0 historical pins to their merged revisions
98803cf2c Replay original 6.0.2 invariants from mapped historical blobs
6bb402768 Pin integrated 6.0.2 validator and preserve every historical invariant
3db5e4142 Bound rebase inventories and seal committed integration proof inputs
db1a53068 Seal owned validator wrapper and retain receipt tamper controls
59c37ece2 Document exact 6.0.2 integration coverage and inherited scenario failure
ca736adaa Retain exact master comparison and completed 6.0.2 regression receipts
076a89d78 Seal portable 6.0.2 integrated receipt bytes
```
