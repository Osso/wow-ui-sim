# Independent bounded 3.4.1 factory artifact verification

Date: 2026-10-09. Canonical snapshot: `7bc19c2b06380a7ce7cfe53a52acb4422354cb21`. Diff base: `0893f7d91`. Source worktree: `/home/osso/.worktrees/wow-ui-sim-p341-factory`; retained GREEN revision: `da31b9ed1`.

**OVERALL: PASS — artifact/accounting/retained-proof scope only. No integrated-runtime, CI or native acceptance.**

[EXIST] PASS — all 20 changed tracked paths exist; exact file sizes and paths in `/tmp/p341-factory-independent-receipts.json`. Only Cargo, one test, docs and evidence changed relative to integration base; zero production/model changes in this change.

[SUBSTANTIVE] PASS — 158-line Rust target contains six concrete cases: exact publication gap set, historical occurrence preservation, exact CVar default differences, invented/native event acceptance control, Command-vs-CVar catalog control, foreign-profile rejection. JSON parsed and all 333 observations individually compared with measurement ledger and inventory; inventory fields individually compared with all 333 historical occurrences. All four complete retained streams read, not just test summaries; exact stream hashes independently match. Empty preservation streams are intentional exit-0 Git receipts, not implementation placeholders.

[WIRED] PASS — Cargo.toml:259–262 registers `patch_3_4_1_factory`, exact test path, `required-features = ["client-wrath"]`; test line 2 has whole-file Wrath cfg. Shared `run_factory_publication_sweep` is called at test line 31 and supplies empty alias map; existing `run_sweep` rejects foreign profile before alias/cache read. Tests exercise values/observable API results, not source-shape assertions. Gating inspection is wiring evidence only, not behavioral proof. Spec/wiki/index/log link target and retained evidence.

[ANTI-PATTERN] PASS — changed Rust: TODO/FIXME/HACK/XXX each 0; no warning suppressions, empty bodies or commented-out implementation. Historical gaps and UNPROVEN docs are limitations, not stub implementations.

## Exact accounting

| Surface | Observations | Matches | Classifier gaps |
|---|---:|---:|---:|
| Globals | 211 | 133 | 78 |
| Events | 44 | 35 | 9 |
| CVars | 77 | 71 | 6 |
| Command | 1 | 0 | 1 |
| Total | 333 | 239 | 94 |

- All 333 unique observation IDs exactly equal inventory IDs; discovery RED and GREEN objects equal individually. Ledger measurement/source rows exactly equal observations/inventory. Known gaps exactly equal all 94 non-ok IDs.
- 15 published CVar string-default differences exactly equal fixture values; Decimal comparison confirms 8 format-only and 7 numeric differences. Missing defaults are not counted as published-default differences.
- Historical seals: 19/19 files hash-match; seal dictionary exactly equals retained preservation dictionary. Original seal-file SHA256 `fae3411a2445ed8176cc42511d89317ca06994d27a12d58a3f4ffe550cc63040` matches. No historical validator rerun or rewrite.

## Proof applicability

Full GREEN stdout: `test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.43s`. Retained command exits 0 at `da31b9ed1`. RED stdout: `test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s`; retained exit 101, empty-gap assertion lists exactly the 94 measured gaps after observations were written. Expected foreign-profile panic in GREEN is the passing should-panic case, not an unexpected failure. GREEN retains six library warnings, one binary unused import and six vendor manifest deprecations; not warning-free.

All 11 GREEN recorded source/input SHA256 values match integrated files, including Cargo/Cargo.lock, target/shared classifier, profile and successor/source inventories. Target and shared classifier unchanged from GREEN through rebase. Retained proof is applicable to these unchanged bounded inputs; no tests/build/check/broad suites rerun.

**Important qualification:** complete runtime tree is not identical to `da31b9ed1`: separately integrated 3.1.0 player-facing work changes four src files. Getter registration remains retail/wowforever-only, but PlayerState gains unconditional nullable facing field. This does not add a Wrath publication implementation; nevertheless original 6/6 is retained development proof, not a fresh integrated compilation/execution claim. Main owns that integration gate.

Fresh `cargo fmt --check` at canonical snapshot exits 0; stdout/stderr both empty. Receipt: `/tmp/p341-factory-fmt-receipt.json`. Manual rust-readability audit covers every changed Rust line: no reportable checklist violations. No Rust metric/compiler/test invocation used for readability.

## Boundaries and findings

Bare Wrath **38001**, never native Classic **30401**, no loaded Blizzard UI/Game/publisher proof. Event acceptance `(true, true)` for invented and named event demonstrates nondiscrimination; native availability/removal/payload/dispatch remains UNPROVEN. Console control `(true, 0)` and false Command probe establish CVar record does not publish LogFps Command, not native command execution parity. 211 signatures, CVar persistence/effects, two source prose summaries and native parity remain UNPROVEN. Models added: NONE.

Administrative doc discrepancy: spec requirement to retain streams/input hashes/preserve seals remains unchecked even though these artifacts are present and verified. No edits made; this is not missing evidence or native acceptance.

Only `/tmp` report/receipt artifacts written. No repo edits, commits, network, operations, delegation, Bash, cargo tests/check/build or broad suites. Existing untracked `.code-index.db` observed and untouched. Main 3.1.0 verifier not disturbed.

Receipts: `/tmp/p341-factory-independent-receipts.json`, `/tmp/p341-factory-fmt-receipt.json`, `/tmp/p341-factory-fmt.stdout.log`, `/tmp/p341-factory-fmt.stderr.log`. Supporting inspected diffs: `/tmp/p341-diff-docs.txt`, `/tmp/p341-runtime-delta.txt`.
