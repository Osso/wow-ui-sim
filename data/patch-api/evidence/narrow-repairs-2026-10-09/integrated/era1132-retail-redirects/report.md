# Independent bounded integrated proof — Era 1.13.2 / original Retail redirects

**PASS for assigned SOURCE, separate-receipt, portable and current Era storage checks.** No historical/native closure or parent-goal completion. One Cargo manifest wiring caveat and one minor readability observation retained below; no repairs made.

Canonical checkout: `/home/osso/Projects/wow/wow-ui-sim`. All new artifacts: `/tmp/era1132-retail-redirect-independent`. Read/followed verify skill as verifier; no delegation. Read owned HANDOFF/REPLAY/spec/wiki pages and entire wiki index (636,933 bytes at initial read; hash and owned entries in `index-read.json`). No unrelated test/check/full-suite invocation.

## Verified identifiers and execution epochs

| Page/history | Merge | Source tip |
|---|---|---|
| Classic Era 1.13.2 | `9fb8968f88c0d3a3de9c32bea9c0e0445c762070` | `9fb20ec3130e0830112966885eeb9ba34d995b07` |
| Original Retail 1.12.0 | `d8b98ecf53702bd9315a331a39a58a0de5ed7c6c` | `0ec696f6e9d2c5f0f08f29df2c593853e1fc4906` |
| Original Retail 1.11.0 | `31686de7147c77499de64b1da0885ae07261efcb` | `592bf2e743dfe27a089f28a7b914717f936d2a3e` |
| Original Retail 1.10.2 | `f8da74e315acd21a0af85512598b27c25da6fd23` | `d026c9e6a4022db89c5e0205461d13f2725a9107` |

Git parent output confirms each merge's second parent is its source tip. Initial actual HEAD was `f8da74e315acd21a0af85512598b27c25da6fd23`; all canonical Python suites/validators and portable executions recorded this HEAD before/after with zero scoped changes. Retail 1.12 merge was **d8b98ecf5**, not the initial mistyped identifier.

Cargo started at `43d9ce02e91bcfeb9f70993a325df50fe1ec1f08` and ended at `f2eebd359a18c127ac228a4750157414086b13d2`. Final sampled HEAD: `a9d2433b6b113aca7ab3e2505c5548a59b4f0445`. This was a shared mutable checkout, not immutable-SHA execution.

Concurrent changes: `43d9ce02` retained failed Mists-capture documentation; `3d1a4140` merged source-only Retail 1.10.1; `f2eebd359` changed only the historical TransmogSituation library-test feature gate plus docs; `a9d2433b` merged source-only Retail 1.10.0. Metadata is retained in `concurrent-history/` and `final-concurrent-metadata/`; their page tests were not run. Initial/final scoped hash delta contains only `docs/wiki/index.md` and `src/loader/tests/wow_api_globals/transmog_situation.rs`. All four owned evidence trees, frozen inputs, source headers/queue, Cargo files and owned CVar test remain hash-identical.

The changed TransmogSituation file is below `src/loader/mod.rs:860`'s `#[cfg(test)] mod tests`, excluded from this standalone integration-test library build, and additionally requires absent `retail-12-0-0`. No applicable production/test-target/lock change invalidated storage proof. No command rerun merely for concurrent commits. `runtime-epoch-diff/stdout.txt` is empty between original runtime epoch `15d12138f56ac2db90499303a80d9b702ccf4e5b` and initial integrated HEAD for src/Cargo/lock/build/owned test. Original owned-base diff adds Cargo target/test only, no production src changes.

## Exact results

| Page | Canonical SOURCE | Original seals | Separate receipt seals | Fresh copied SOURCE | Portable controls |
|---|---:|---:|---:|---:|---:|
| 1.13.2 | 8/8 | 89/89 | 43/43 | 8/8 (nested portable process) | 3/3 |
| 1.12.0 | 5/5 | 23/23 | 7/7 | 5/5 | 3/3 |
| 1.11.0 | 5/5 | 24/24 | 6/6 | 5/5 | 3/3 |
| 1.10.2 | 5/5 | 26/26 | 6/6 | 5/5 | 3/3 |

Canonical SOURCE **23/23**; copied SOURCE **23/23**; portable **12/12**. Separate 1.13.2 successor suite **3/3** and receipt-validator suite **3/3**. Era runtime **1/1**. Total **65 passing test executions**, including intentionally repeated copied SOURCE executions, not 65 unique cases. No failures/errors/ignored/filtered runtime tests. All top-level proof commands exited 0; expected nested rejection exits are separately retained, not failures.

All four canonical original validators exited 0. Original maps validate **162/162** entries; separate maps validate **62/62** entries (not necessarily distinct content). Redirect separate seals checked directly against every SHA256 entry; Era separate validator additionally derives source/successor/model/restoration counts. Final seal reinspection confirms zero mismatches without rewriting or resealing.

Era source: **2,758 inventory**, **5,011 UNPROVEN contracts**, 2,853 physical/2,838 nonblank rows, 2,764 signature-limit records, 23 prose boundaries, 11 headers/9 matching numeric terms/4 captions, 19,013 omission controls. Zero declared full signatures or historical/native closure. Three Retail sources are independently pinned **45-byte redirects**, each one unexpanded reference/one UNPROVEN contract, zero inventory/signature/default/prose/header/model/runtime/native credit. Empty local inventory does not imply no historical patch changes.

Separate Era successors: **20 actual ledgers**, **581 exact overlaps** affecting **580 source occurrences**, **2,178 without exact overlap**. Frozen queue remains unchanged; no foreign-history supersession or semantic closure. Original Retail references remain separate/unapplied.

## Portable proof and rejection boundaries

Fresh archives were extracted below this report directory; every member was a safe regular relative file, with no Git/target/current-tools/addons. Members/bytes: Era **90 / 779,729**; Retail 1.12 **24 / 47,289**; 1.11 **25 / 48,326**; 1.10.2 **27 / 50,127**. Member hashes and archive hashes are in `archive-proofs.json`.

Era used its documented `test_portable.py --archive ... --scratch ... --receipts ... --cwd /home/osso/Projects/wow/wow-ui-sim`, not deleted worktree cwd. Redirect copies used absolute `/usr/bin/python3 -I -B`, `PATH=/nonexistent`, documented audit/SOURCE/portable script argv and new receipts. Era nested children used empty PATH. Each page's disk ledger omission and fabricated GREEN log was rejected; **8/8 serialized tamper controls** and **8/8 exact restorations** passed, original bytes/maps preserved. Fresh copied validators reproduced historical defaults without Git/target/current tools. No generic full-validator replay was substituted.

Era default register reproduces 2,758 identities exactly; its historical extractor still rejects with retained ValueError and no stdout/text output. Default header omission and original target/display disagreements are preserved, not repaired. Redirect default bytes reproduce exactly; 1.10.2's retained malformed-symbol/null-extract exception types/messages remain expected rejection. Portability is bounded to supplied Python standard-library copies and this Python 3.14.7 host; no cross-OS/Python-version/native compatibility proof. Hash maps do not authenticate evidence against replacement of every map/archive.

## Single current-model invocation

Exact Cargo argv/cwd/env/times/hashes and full streams: `era-runtime/receipt.json`, `stdout.txt`, `stderr.txt`.

`cargo test --offline --locked --target-dir /tmp/era1132-retail-redirect-independent/cargo-target --no-default-features --features client-era --test patch_1_13_2_cvar_state -- --nocapture`

Runner configured through `CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER=/usr/bin/timeout 120s`; build had no timeout. Owned TMPDIR and observation output are under `/tmp/era1132-retail-redirect-independent`. One invocation, no duplicate runtime filters. Command **exit 0**, **280.551s**; build reports **4m40s**, test **0.17s**.

Exact fresh observations equal explicit cases and historical owned observations. `rawget(_G,...)` types: SetCVar/GetCVar/GetCVarBool all **function**, not name-factory credit. Six transitions, each accepted true, GetCVar exact string, GetCVarBool exact bool, Rust store exact string:

| CVar | Input sequence | Boolean sequence |
|---|---|---|
| alwaysShowTargetNameplate | 0 → 1 → 0 | false → true → false |
| instantQuestText | 1 → 0 → 1 | true → false → true |

Configured Era **11507**, source **11302**, intentionally distinct. Existing bare generic storage only; no source-default, historical-signature, UI-effect, native security/server/loaded-UI credit. Source ledger unmodified, runtime implementation changes zero.

## Warnings and audit findings

Retained **13 existing diagnostics**: six iced-wgpu-patched Clippy-key manifest deprecations, six library warnings (PROVENANCE_SCHEMA, CacheProvenance::new, remove_missing_marker, ensure_known_asset_cached, encoding_key_hex, cooldown helpers), one wow-sim unused SavedVariablesManager import. Cargo summary lines are not additional distinct warnings. No iced aggregate compiled, no warnings suppressed/fixed, no zero-warning claim.

Readability: full new 46-line Rust test inspected against original owned base/tip; one loop, no nested conditionals/suppressions or new production Rust. Minor [STATE] checklist match at test line18: mutable Vec plus push could be collected while preserving mutation order. Detailed finding in `rust-readability.md`; no edit authorized.

**Cargo wiring caveat:** original owned diff inserts the new test immediately before an existing `required-features = ["client-era"]`; that line now belongs to the CVar target, leaving the preceding existing NPC target without its former explicit gate. Exact diff retained in `owned-runtime-diff/stdout.txt`. The assigned CVar target remains correctly gated and passed; adjacent-target behavior was not executed/audited and no repair was made. Bounded PASS is not whole-manifest safety approval.

## Retention and limits

Terminal `cli.*.cwd(...).capture().run()` only; no spawn, persisted process handles, detached jobs or lost exit/streams. Every proof command saves full stdout/stderr immediately with exit, UTC start/end, duration, argv, cwd and before/after HEAD/scope hashes. `commands.json` indexes all receipts; each command directory retains independent full receipts/streams. Nested Era process streams and redirect restoration receipts are retained unchanged.

Actual environment key names inspected first. Only key names retained for ambient environment; all ambient values omitted. Explicit retained overrides are owned paths, isolated replay settings and timeout runner, not credential values. No full environment snapshot or exhaustive secret-detection claim.

All four old owned worktree paths were absent; worktree inventory confirms canonical committed evidence/copies were used. Initial/final git status contains only pre-existing untracked `.code-index.db`; this verifier did not create/edit it. No repository code, cache, vendor, frozen evidence, commits, push, deployment, network, agents or model CLI changed by verifier. Compilation artifacts are owned `/tmp` outputs required by assigned runtime invocation. Histories remain separate; **all 5,011 Era historical contracts and each redirect's missing historical contract remain UNPROVEN**. No parent completion/native/final-suite acceptance asserted.
