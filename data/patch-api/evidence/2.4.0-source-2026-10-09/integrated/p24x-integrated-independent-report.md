# Independent 2.4.x integrated verification

## Verdict
PASS — current owned source parsing/accounting and immutable historical replay. NOT native/model acceptance. Recorded saved-output scope is not all-green: listed mismatches/errors remain unchanged; no fixes authorized or applied.

## Revisions and content scope
- 2.4.2 integration: `0b31440904988badd674a11fa39d8ad029690ae8`.
- 2.4.0 integration: `923b0c98680b7af18166217de4a918d7c6f8c98c`.
- Targeted six fixtures: `8877566d26df0d89b8923b04d51c6c1954e9796f`.
- Additional 2.1.0 fixture: `88217d328e88c908a9f5f34f4ac00f5483515e8f`.
- Whole replay and final read-only scope: `dd710c6fc469a53e3b77b97aa6340993ae83e46e`.
- Final receipt time: `2026-10-09T14:33:19.562428+00:00`. Covered content unchanged since fixtures and replay; staged input hashes equal canonical hashes. Later document-only commits do not invalidate this selected-content proof.
- Generator SHA256: `65e9c786c9382e52c8a6d725e16bbe2c37f033f550e3ef51ef4b428f0d1eedc6`.
- Extractor SHA256: `fb53f8c267e388a7e1650ff3fbeec8dbfd663fd3cc70787e283ef1a7697f5532`.

## Exact targeted commands
All commands use explicit cwd `/home/osso/Projects/wow/wow-ui-sim`; Python `-B` prevents bytecode writes. Test fixtures live directly under `tools/`, not `tools/tests/`. Complete stdout/stderr retained separately; no broad Cargo/check/full-suite/global commands run.

| File (`python3 -B ABSOLUTE_FILE -v`) | Tests | Exit | UTC start | Seconds |
|---|---:|---:|---|---:|
| `test_patch_2_4_source.py` | 2 | 0 | 2026-10-09T14:27:02.892770+00:00 | 0.138061 |
| `test_patch_2_4_replay.py` | 3 | 0 | 2026-10-09T14:27:03.030967+00:00 | 1.411070 |
| `test_patch_2_4_2_source.py` | 2 | 0 | 2026-10-09T14:27:04.442344+00:00 | 0.128353 |
| `test_patch_2_4_2_accounting.py` | 2 | 0 | 2026-10-09T14:27:04.570909+00:00 | 0.037308 |
| `test_patch_2_4_2_replay.py` | 1 | 0 | 2026-10-09T14:27:04.608393+00:00 | 39.370230 |
| `test_patch_3_0_2_source.py` | 4 | 0 | 2026-10-09T14:27:43.978908+00:00 | 0.142093 |
| `test_patch_2_1_0_source.py` | 4 | 0 | 2026-10-09T14:28:48.550641+00:00 | 2.406557 |

18/18 tests pass; all seven commands exit 0. No assertions were skipped.

## Capability matrix
| Boundary | Proven now | Limit |
|---|---|---|
| Retail 2.4.2 summary/markup opt-in | 11 exact inventory occurrences and annotations; plural/placeholder/reference/limit text; regenerated register/text match saved bytes | Source/factory accounting only; no modeled historical behavior acceptance |
| Retail 2.4.0 summary opt-in | 50 unique source occurrence IDs; 99 retained nonblank lines; defaults unchanged | 169 pending contracts, zero meaningful closures |
| 3.0.2 launch inventory | 373 occurrences; 346 labeled rows each accounted once; repeated symbols preserved; malformed labeled row rejects | Source parser fixture, not native parity |
| Legacy labeled summaries | Own `return entries` remains at generator line 1134; 3.0.3/3.0.8/3.1.0 recorded register/text outputs pass additional 2.1.0 fixture | No inferred extra API/domain expansion |
| 2.1.0 profiling opt-in | Own inventory/accounting/omission/default isolation + recorded template outputs pass 4 tests | Source-only |
| Default parser scope | 170/170 input pairs equal sealed archived baseline, including error outcomes | Exact captured input set only |
| Historical 2.4.2 evidence | Fresh no-Git/no-target validator exit0; 53 ledger rows; original23 + separate portable5 seals match | 2 bounded publication receipts, 9 publication gaps, native unmeasured |
| Historical 2.4.0 evidence | Fresh no-Git/no-target validator exit0; 99 source/50 inventory/46 signatures; original432 + closure3 seals match | Parser regression receipts retained in archive; current replay recorded separately |

## One whole recorded/default replay
- UTC start `2026-10-09T14:30:08.996700+00:00`, elapsed `19.912967` seconds.
- Existing sealed `data/patch-api/evidence/2.4.0-source-2026-10-09/replay_parsers.py` supplies load/extract behavior; generate wrapper records complete argv/streams and uses explicit canonical cwd. Copies of current tools and captured inputs execute in `/tmp/p24x-integrated-independent-replay`; no canonical extract output is overwritten.
- 427 generator subprocesses: 170 baseline/current default pairs plus 87 recorded-option registers. Extract comparisons use existing runner `extract(module, raw, flags)` in memory, recording successful byte hashes or exact exceptions.
- Recorded result: 83/87 registers and 78/87 extracts match saved bytes after explicit bounded harness correction; 2.4.0,2.4.2,3.0.2,2.1.0 owned opt-in outputs match.
- Initial archived runner interpreted 3.0.2 CLI-only `--text-only` as `text_only=True`; this is a harness TypeError, not a source extractor failure. Raw whole-replay receipt retained unchanged. One isolated exact recorded extractor CLI invocation exits0 and yields SHA256 `5e814a5740e836a848f63859a95bb0ea00d32820c6172756f1dbaa4e673cd86a`, matching saved output. Whole replay was NOT rerun.

### Preserved saved differences
- Register mismatches: 10.0.0,12.1.0,2.3.0,2.5.3. The 2.3.0 saved inventory has no current provenance flag file; replay records default flags, not an invented flag selection.
- Extract byte mismatches despite exit0: 10.0.0,10.0.2,10.1.0,10.1.7,10.2.5,12.0.5,12.0.7,9.2.5. Archived baseline yields same bytes and mismatches for these.
- 12.1.0 extraction exit1 retained verbatim: `ValueError: unhandled template: {{#description2:Midnight 12.1.0 (Curse of Ula’tek)}}`. Archived baseline has the same exception. No saved difference fixed.

## Fresh relocated historical validators
- Evidence trees copied independently to `/tmp/p24x-integrated-independent-relocated-2.4.0` and `...-2.4.2`. Neither contains `.git` or `target`. Validators derive historical inputs from their own copies; 2.4.2 skips archived Rust code during extraction. `PATH=/nonexistent`, `PYTHONDONTWRITEBYTECODE=1`.
- Exact validator argv/cwd/env/time/exit/hash in each `...-relocated-VERSION-command.json`; complete streams in `...-relocated-VERSION.stdout` and `.stderr`. Both exit0. Test fixture tamper cases reject altered serialized source/ledger/log/archive/seals and restore original hashes.
- Additional 2.4.0 development seals3 match. No original seal/input was edited. Original evidence source hashes and current source input hashes stayed stable through final snapshot.

## Acceptance exclusions
- Historical factory GREEN/RED and currency amount/separator development proof are preserved artifacts, not a fresh runtime test. No Rust binary built/executed; no native client probe; no model/12.x/global acceptance claimed.
- No source edits, worktree/directory switch, delegation, push, deploy, or services changed. Zero auto-backgrounded commands. One whole replay only, plus one explicitly recorded scoped harness correction.

## Receipt index
- `-fixtures.json` — six exact commands, revisions, start times, durations, exits, stream hashes and stream paths.
- `-test_patch_2_1_0_source.json` — extra own fixture command and before/after hashes.
- `-before.json`, `-after-fixtures.json`, `-replay-before.json`, `-replay-after.json`, `-final-scope.json` — exact content-scope receipts.
- `-replay.json` — untouched full replay, 170 defaults, 87 recorded outcomes, all 427 generator argv/stream receipts.
- `-replay-command-NNNN.stdout/.stderr` — complete generator streams; `-replay-302-cli-correction.json` plus streams — isolated harness correction.
- `-seal-checks.json`, `-relocated-VERSION-command.json` plus streams — original/portable seal checks and fresh validators.
- `-summary.json` — effective outcomes with explicit initial harness failure and supplemental correction; no silent replacement.

All above suffixes use prefix `/tmp/p24x-integrated-independent`.
