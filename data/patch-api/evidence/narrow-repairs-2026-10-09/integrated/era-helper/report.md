# Independent Era helper integration verification

Behavioral/artifact result: PASS. Receipt completeness caveat below.

Canonical cwd: `/home/osso/Projects/wow/wow-ui-sim`. No cwd switch, repository edits, delegation, push, deploy, or network. Only offline/locked Cargo commands. All four requested revisions are ancestors of verified HEAD. Started at a3c66ef5e75f41223efc9f7c271d629374244096; report HEAD 5b12dac256abc0ffaf541df2bc445e8381612d2d. Exact source hash scope unchanged throughout (`scope-before.json` == `scope-after.json`); concurrent docs/data commits do not invalidate those proofs.

## Coverage matrix

| Contract | Evidence | Result |
|---|---|---|
| p1143 original/current seals | `seal-successor-checks.json` | 54/54 + 21/21 unchanged |
| p1142 original/current seals | same | 72/72 + 17/17 unchanged |
| Copied SOURCE fixtures | `1.14.3-source.stderr`, `1.14.2-source.stderr` | 8/8 each, exit 0 |
| Independent portable replay | `*-portable-fixed.*`, `*-portable-controls-fixed.json` | 3/3 each, exit 0; copied SOURCE 8/8 within each; empty subprocess PATH, no Git/target/tools tree; regenerated default bytes exact; serialized ledger omission/log fabrication rejected then exact seals restored |
| Same Era successors | `seal-successor-checks.json`, copied SOURCE tests | p1143: 1.14.4 + 1.15.0–1.15.9; p1142 additionally 1.14.3. No TBC supersession. Later registers empty; no imported behavioral credit |
| Combined Era execution | `era.stdout`, `era.stderr`, `receipts.json` | p1143 2/2 + p1142 1/1, exit 0; exactly manifest targets, client-era only, no default features; required absolute output path |
| Exact factory inventory | `factory-observations.json`, `comparison.json` | Entire JSON equals retained factory-green-observations.json; 412/412 IDs/order/details/values/defaults/classifications unchanged; 286 direction matches / 126 reviewed mismatches |
| p1142 current/default values | `era.stderr` | faction 1/1; vibration 1/1; telemetry nil/nil; unknown control nil/nil |
| Narrow helper warnings | old retained factory-first/green logs versus fresh `era.stderr` | Original 18 target warnings absent. Baseline remains 6 simulator library + 1 simulator binary warnings, separately 6 vendor manifest warnings. No helper warnings, suppressions, or visibility expansion |
| Extraction compatibility | `comparison.json` + source inspection | CLASSIFIER, Entry fields/serde default alias, ProbeResult tuple, quote_lua, probe_entry byte-identical to d512bdd4e parent. Legacy publication_sweep reexport retained, same pub(crate) boundary |
| Required Retail full-publication subset | `retail-publication.stdout/.stderr`, receipts | One default-feature offline/locked prefork invocation; 78/78 selected publication_sweep cases, exit 0. Original committed registers/known gaps and cache sources used; no scratch register/env overrides. Passed case results retained; no per-row sweep output maps requested by env |
| Default factory consumers | copied publication-probe logs + `proof-summary.json` | Reused p303 3/3 and vertex-color 1/1 GREEN; no rerun. Their src/tests/helper scope unchanged since d512bdd4e. Original revision supplied by caller; logs lack their own revision metadata |
| Fresh formatting | `fmt.stdout/.stderr`, receipts | cargo fmt --check, exit 0 |
| Changed Rust readability | manual audit below | No newly introduced violations |

## Changed Rust readability / wiring

Read every changed line in tests/common/publication_probe.rs, tests/common/publication_sweep.rs and patch-tests/patch_1_14_3_factory.rs; also read the integrated patch-tests/patch_1_14_2_cvars.rs. No available rust-code-analysis-cli/readability-audit executable; manual audit used. Rust helper functions remain short, shallow, named, and unchanged; extraction introduces no new branches, hidden effects, duplicate classifier, warning suppressions, or broad public visibility. Entry serde fields and alias unchanged. Existing large Lua CLASSIFIER is moved verbatim, not rewritten. Factory probes invoke the narrow helper; sweep classify_entry invokes the legacy reexport. Target files present and wired through explicit manifest declarations. TODO/FIXME/HACK/XXX and #[allow counts zero in these files.

No model/native parity claimed: all 412 model_credit/native_credit fields false, original source measurements remain zero. RegisterEvent accepts a nonsense name; event registration gives no producer/native credit.

## Harness and receipt limitations

Initial portable launch wrapper used runpy globals incorrectly: OPTIONS was absent in method globals, producing 3 setup errors per patch before assertions ran. Those failed streams are preserved. Corrected launch executes original test text in one namespace and changes only its hard-coded subprocess CWD in memory to the canonical cwd. Corrected portable checks passed; no source/test behavior edited. CLI argv captures this adjustment.

First combined Era CommandResult was printed before the persistent receipt helper existed. Its full stdout/stderr were transcribed verbatim from the tool result into era.stdout/era.stderr; hashes identify those retained transcripts, not independently captured raw streams. Exact wall-clock start/end timestamps unavailable; observation file mtime retained, reported build duration 1m15s. No rerun solely for log recovery. Later commands have directly captured separate streams and start/end times. This is a receipt metadata limitation, not a behavioral failure; do not represent the first invocation as fully automatically captured.

All remaining command argv/cwd/env overrides/revisions/times and stream hashes: `receipts.json`. Relevant source hash scope: `scope-before.json`, `scope-after.json`. Retained GREEN logs copied unchanged and hashed in `proof-summary.json`. No full suite or duplicate cargo checks run.
