# Independent P801 historical preservation proof — GREEN

Verified 2026-10-10 in `<repo>`, HEAD `63f3a8a7373c8e52ffcc005bb443a9cf64b59fbf`.

## Historical validation — PASS

Read `<verify-skill>` first and the complete 343-line `data/patch-api/evidence/8.0.1-session-2026-10-08/validate.py`. Inspected imported helper behavior: validation reads local files/Git blobs; imported generator/extractor entrypoints are guarded. No network or repository mutation in this validation path.

Ran native Python entrypoint exactly once through Pyrun with absolute repository cwd:

`python3 <repo>/data/patch-api/evidence/8.0.1-session-2026-10-08/validate.py`

Exit 0; empty stderr; JSON status PASS. Saved full output and status beside this report: `historical-validation.stdout`, `historical-validation.stderr`, `historical-validation-status.json`.

Validator reports 158 sealed artifacts, 269 historical inventory rows / 252 OK / 17 gaps, 199 historical preserved inputs, 41 historical sweeps, 18 proof receipts, 12 integrated required receipts. These are retained-proof validations, not newly executed Rust suites. Historical inherited extract failures remain 12.0.5, 12.0.7, 12.1.0; no new credit assigned.

## Exact frozen/live distinction — PASS

`tests/data/patch_8_0_1_sweep_known_gaps.json` at Git revision `d142860059e816cb15ebefcbd12e75bc8cdc8372` retains exactly 17 entries and SHA-256 `bad5e7e4e77494f4e506684fb63b97d5b0d8ff9dd8279f714fbf00ad32bac55a`.

Live fixture has exactly 16 entries and SHA-256 `618f9a1c663f16ce1df4613ff953e69b322e2cd04fe2868ad099189ab669e1b1`. Sole removed ID: `wt-global-api-C_Map.GetMapDisplayInfo-30`; no additions.

Inspected `preserved_input_matches`: it accepts original recorded digest or exact `(recorded_digest, current_digest)` tuple in `LATER_AUDIT_REPLACEMENTS`. P801 replacement is precisely the two hashes above; no count-only, row-only, or whitespace-tolerant acceptance. The existing 11-test log covers unrelated byte/second-ID rejection and historical 17/live 16 preservation. Read `<private-log>`: 11 tests, OK; did not rerun its command.

`git diff 63f3a8a73^ HEAD -- data/patch-api/evidence data/patch-api/sources` is empty. Frozen register/source/evidence files unchanged by this closure. Commit 63f3a8a73 changes only the live gap fixture and exact digest allowlist. Saved `preservation-proof.json`.

## Existing current native P801 receipt — GREEN, not pending

Observed ONLY `<private-verification>/map-display-publication-green-current` once for availability; no polling. Saved proof present at `20261010T195723Z`.

Submission revision matches HEAD. Existing `execution-results.json`: exact P801 selector, exit 0, reached 1 / expected 1 test, artifact unchanged. Existing stdout: 1 passed, 0 failed. Existing outcome: compile exit 0, source equality true. No Cargo/build/test command executed by this verifier.

Actual saved result has 269 rows, 253 OK, 16 gaps. Actual `wt-global-api-C_Map.GetMapDisplayInfo-30` row is `ok: true`, observed `raw=function; lookup=function`, expected publication `published`. Failed-row IDs exactly equal the live 16-entry known fixture (`native-failed-set-comparison.json`). Before/after source snapshots equal; every captured current source hash matches saved snapshot (zero mismatches), including fixture SHA-256 618f above. Saved `native-saved-proof-observation.json` and `native-current-snapshot-comparison.json`.

## Credit boundary

GREEN proves historical preservation, exact live-input reconciliation, and current simulator API publication under the saved exact native P801 test. It does NOT establish native WoW semantics, C++ backing-model correctness, unknown-input behavior, source-to-artifact provenance of external dependencies, or complete patch implementation. No source C++ model or unknown native semantics credited. Saved submission exclusions remain applicable.

## Execution scope

No delegation, network, Cargo, build, broad suite, deployment, operations, or repository edits. Only read-only investigation plus one local historical validator invocation; verification artifacts written under the requested output directory. Existing unrelated checkout changes left untouched.
