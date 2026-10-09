# Bounded 4.2.0 verification — PASS within stated scope

Verified read-only checkout `/home/osso/.worktrees/wow-ui-sim-p420-source`; observed final HEAD `ee8a25efced8b5b7b92d3195ffb9fd3034a6c92d`. No cwd-switch, delegation, Bash, repository edits/commits, cargo test/check/build reruns, broad suites, or shared portability launch. Only verifier-generated `/tmp` reports/receipts written.

## Coverage and proof

| Capability | Observation | Proof / limitation |
|---|---|---|
| Retail BNGetFriendIndex | Current ordered `SimState.bnet_friends` position + 1, nil when absent | Real model implementation; native signature/error/unknown-ID parity unproven |
| Original friend behavior | Retained GREEN 2/2, RED 0/2 on nil global | Same model/test/defaults/C_BattleNet/state/harness blobs across recorded GREEN, integrated acceptance, and current HEAD; not rerun |
| Current publication | 65 observations, 35 matching, 30 exact gaps | Main's 68/68 publication run exit 0; 4.2.0 output matches exact current known-gap set |
| Successor closure | Only `wt-global-api-EJ_SetDifficultyByMask-39` moved from gap to match | 4.3.0 removal `wt-global-api-EJ_SetDifficultyByMask-88`; observed raw=nil/lookup=nil, not historical behavior implementation |
| Historical accounting | 65 observations, 34 matching, 31 gaps; negative 32 gaps | Own validator exit 0, byte-identical source replay; 73 archive members, 64 actual historical successors, 2 pending placeholders, 19 receipt-artifact seals |

## Model reachability and readability

- Implementation `src/lua_api/globals/real/bnet_friend_index.rs:8-20`, 21 lines. Lookup uses `.position()` on account ID, not stale `friend_index` fields. Shared backing list declared at `src/lua_api/state/sim_state.rs:588`; C_BattleNet reads list position at `src/c_api/c_battle_net.rs:507-525`.
- Retail-only module gate at `src/lua_api/globals/real/mod.rs:10-11`; retail registration at `src/lua_api/globals/register.rs:154-155`.
- `tests/patch_4_2_0_behavior.rs`, 40 lines: ordered IDs 100001/100002, swap, C_BattleNet cross-check, removal, unknown and empty list. Included by `tests/integration.rs:1` through `build.rs:54-90,463-508` top-level Rust-module discovery.
- Read/followed Rust readability skill. Manual audit found no substantive readability violations in bounded model/test sources: no deep nesting, long functions, opaque conditions, suppression attributes, or TODO/FIXME/HACK/XXX markers. Metrics binary unavailable; no numeric cognitive-complexity claim.

## Hash-backed proof applicability

`/tmp/p420-verifier-behavior-blob-map.json` maps eight direct model/test/state/default/C_BattleNet/harness inputs at recorded GREEN `6b89034a3`, acceptance `566ce85d7fdc448d8ef32b42d9f8e551078c885a`, and current HEAD: all identical. Model SHA256 `7a8932152c77b0b7699eb9cc4a500853298c632302ba6035fa82c9b2e88b8e0d`; behavior-test SHA256 `46361a0af8fcb305e9bff5d6e944e76f33dd26d553c802df2e646bb3c4df0e21` match original ledger.

Do NOT claim every historical runtime-input file is unchanged: `globals/register.rs` differs by the independent retail GetSessionTime registration only; original BN registration unchanged. Historical sweep source differs by actual 4.3.0/4.3.4 successor additions, so original 31-gap sweep is historical evidence, not the current sweep proof. See `/tmp/p420-verifier-runtime-hashes.json`.

Acceptance tree identity across rebase/evidence commits verified by SHA256 of `git ls-tree -r <revision> -- src tests Cargo.toml Cargo.lock build.rs native Interface`: both acceptance and current HEAD equal `44feb0140623280dbb122ddfd3ac6360e5a20294e96a527b73c51b62b1f4039c`. `/tmp/p420-verifier-acceptance-scope-hashes.json` records this.

## Historical preservation and independent command

Own exact command, once:

`python3 -I /home/osso/.worktrees/wow-ui-sim-p420-source/data/patch-api/evidence/4.2.0-session-2026-10-09/validate.py`

Executed with Pyrun `cli.command(...).cwd(checkout).capture().run()`: exit 0, stderr empty, PASS, source_replay byte-identical, native_runtime_replay false. Full stdout and command saved `/tmp/p420-verifier-validator-result.json`.

All 73 archive-member Git blob hashes independently match original handoff `ebf7d2c9155260c62ef4732ec876d141f1318e88`, including exact original fixture and coverage ledger. All 20 retained manifest evidence files also match that original tree; 19 of these are ledger artifact seals (development-proof itself is separately retained). See `/tmp/p420-verifier-original-archive-hashes.json` and `/tmp/p420-verifier-original-evidence-hashes.json`.

Validator checks pinned page 315583/revision 3045158/time 2021-08-22T03:01:41Z, source-response/raw-source bytes, provenance, replayed register/extract, exact fixture/ledger identities and credit, successor identities, negative extra row, and retained logs/results against seals. Hashes detect alteration, not independently attest native truth. Existing 13/13 validator fixtures and all validator-proof seals inspected and unchanged; NOT rerun.

## Fresh formatting command

`cargo fmt --check`, Pyrun `cli.cargo('fmt','--check').cwd(checkout).capture().run()`: exit 0, stdout/stderr empty. Receipts `/tmp/p420-verifier-fmt-start.json`, `/tmp/p420-verifier-fmt-result.txt`. Scope tree remains identical after receipt commit; no formatting rerun needed.

## Main-owned acceptance receipts inspected, not rerun

Receipts under `data/patch-api/evidence/4.2.0-session-2026-10-09/integrated/`; original `/tmp` copies byte-identical where compared:

- `/tmp/p420-integrated-publication-{start,result}.json`, `.log`, `-results.json`: `cargo test --test prefork_full_ui publication_sweep -- --nocapture`; exit 0, `68 passed; 0 failed; 68 total`.
- `/tmp/p420-mists-check-{start,result}.json`, `.log`: `cargo check --no-default-features --features sound,gui,casc,client-mists --tests`; exit 0.
- `/tmp/p420-runtime-build-{start,result}.json`, `.log`: `cargo build --bin wow-sim`; exit 0.
- Integrated `p420-runtime-lua-errors-result.json`, `.stdout`, `.stderr`: `timeout 90 /home/osso/.worktrees/wow-ui-sim-p420-source/target/debug/wow-sim --no-addons --no-saved-vars lua-errors`; exit 0, stdout `[]`. Build-revision/binary hash recorded in receipt.
- `/tmp/p420-negative-current-{start,result}.json`, `.log`, `/tmp/p420-current-negative-results.json`: intentional control exits 1; 65 rows, 31 gaps, exactly `negative-p420-missing-global` added to current 30 gaps, no resolved/stale gaps. Expected rejection, not positive test failure.

Full publication/check/build outputs inspected; six iced manifest deprecation warnings remain. No warning-free claim. Output review `/tmp/p420-verifier-acceptance-review.json`.

## Remaining limitations

Shared portability start `/tmp/p420-portability-start.json` observed; final result not yet available at inspection. Did not launch or duplicate it. Native WoW parity, broad suites, CI, Classic runtime behavioral parity, signatures/errors, and complete 4.2.0 behavior remain unproven. Thirty publication gaps remain explicit. This PASS is bounded evidence verification, not broader goal completion.
