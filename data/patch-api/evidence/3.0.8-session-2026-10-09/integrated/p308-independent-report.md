# Bounded independent 3.0.8 integrated proof — 2026-10-09

## Verdict and scope

PASS for assigned 3.0.8 publication/existing empty-HookScript subset at stable `dd8ca0f19e9f0172851116a7bfae43792a1b78b4`, recorded-option parser replay at `98f2ef2286de7523a627fa1060a8f548b12a3b1b`, default-parser preservation at `efbe97f20fa49dd12c284f8e3505e16decd3d711`, and separately requested lightweight 3.0.3 / 3.0.2 proofs below. Not whole-current-canonical/native/security/full-suite/CI acceptance.

Read/followed verify skill as independent verifier; read wiki index before 3.0.8 audit/spec and rust-readability skill. No repository edits, commits, network, delegation, Bash, broad Cargo tests/check/build, or full-suite commands. Temp reports/receipts and normal test/build artifacts only. No extra Cargo test after the one assigned focused command. Parent owns profile checks, asynchronous full suite, native and final gates.

## Exact command receipts

Every command below ran through `/tmp/p434-acceptance-runner.py` using Pyrun argv helpers. Exact argv, cwd, explicit environment overrides, full stdout/stderr, timestamps, start/end HEAD/status/tracked-input-diff hash retained in `/tmp/<name>-result.json`; combined untruncated logs `/tmp/<name>.log`; start scope `/tmp/<name>-start.json`. All 177 retained runner receipts have unchanged start/end scope. Full output read; not inferred from exit alone.

Raw aggregate: `/tmp/p308-independent-raw-receipts.json` (177 complete receipts, 437073 bytes, including the initially failed verification harness and inherited extractor failure). No logs discarded or commands repeated just to recover output.

### Stable 3.0.8

Cwd `/home/osso/.worktrees/wow-ui-sim-p308-source`; HEAD `dd8ca0f19e9f0172851116a7bfae43792a1b78b4`; clean tracked scope and final worktree. Env overrides `{}` except specified retail output.

| Receipt prefix | Exact argv | Observed result |
|---|---|---|
| p308-independent-retail | `/usr/bin/cargo test --offline --test prefork_full_ui -- patch_3_0_8 --nocapture` | exit 0; 2 passed / 0 failed / 2 total |
| p308-independent-source | `python3 tools/test_patch_3_0_8_source.py` | exit 0; 1/1, OK |
| p308-independent-validator | `python3 tools/test_patch_3_0_8_validator.py` | exit 0; 1/1, OK |
| p308-independent-historical | `python3 data/patch-api/evidence/3.0.8-session-2026-10-09/validate.py` | exit 0; 25 seals |
| p308-independent-fmt | `/usr/bin/cargo fmt --check` | exit 0; empty output |

Retail override exactly `P308_SWEEP_OUT=/tmp/p308-integrated-results.json`. Actual passing cases are `patch_3_0_8_hook_without_script` and `patch_3_0_8_publication_sweep`, not 56 behavioral tests. Native log contains six inherited iced manifest lint-key deprecation warnings and one aggregate warning line; no other warning. Fresh build finished in 2m03s; full startup output retained. No startup-error parity gate inferred.

Historical stdout retains 56 inventory / 226 ledger IDs, 57 raw / 57 extract nonblank rows, 13 headers, 30 explicit / 26 unspecified signature rows, 71 original actual retail successors, statuses 22 bounded / 97 pending / 107 metadata, 22 matches / 34 gaps, negative 35. This is immutable original evidence, not a newly measured historical/native client. Portable fixture passes relocation/tamper/restoration on disposable evidence; no original seals changed.

## Exact current observation comparison and successors

`/tmp/p308-integrated-results.json`: 56 observations, **22 matches / 34 gaps**. Entire parsed JSON equals sealed `own-sweep-green-results.json`: **zero changed observations**, not merely equal counts. `/tmp/p308-independent-observation-delta.json` is `{}`. As audit states, secure ShowHide template factory failure is not secure-template removal proof; only 21 classifier matches receive publication credit.

`tests/patch_3_0_8_publication_sweep.rs:20-93` now includes 74 actual retail successor registers in verified numeric oldest-first order, beginning 3.1.0, 3.2.0, 3.3.0, 3.3.3, 3.3.5, 4.0.1. No Wrath Classic successor included.

| Literal identity | Actual integrated register/observation | Proof limit |
|---|---|---|
| GetGlyphLink | 3.1.0 exact `changed` row: `* UPDATED - link = GetGlyphLink(index [,talentGroup])`; current `raw=nil; lookup=nil`, gap, `superseded_by=null` | Changed is not removal or closure. No signature/link/unknown-ID proof. |
| SetUpAnimation | 3.1.0 raw source line 182 describes removal; **zero registered successor identity**. Own 3.0.8 added secure-control row remains `factory=false; type=string`, gap, no supersession | Ownerless prose not expanded into registered contract. Not an ordinary frame method or proven restricted-environment removal. |
| RestoreVideoStereoDefaults | 4.0.1 exact removal `wt-global-api-RestoreVideoStereoDefaults-74`; expected absent, same superseded ID already in original historical observation; `raw=nil; lookup=nil`, match | Old removal already applied; no new closure/model/native stereo-default credit. |
| UninviteUnit | 3.3.3 exact changed row; current absence expectation is already later-superseded `wt-global-api-UninviteUnit-185`; published deprecated fallback recognized by existing classifier | Entire row identical to original; 3.3.3 changed row does not invent new closure. |
| 3.2.0 / 3.3.0 / 3.3.5 overlaps | Zero exact own-symbol overlaps | No guessed supersession. |

Hook test `tests/patch_3_0_8_publication_sweep.rs:99-113` verifies an ordinary Frame starts with nil OnShow handler, HookScript then receives the same frame exactly once on each of two hide/show cycles. No new implementation/model. Not universal ScriptObject, restricted event/security, coercion, or native proof.

## Whole merged parser: recorded options and defaults

No 3.1 fixture rerun: existing `/tmp/p310-independent-integrated-source-result.json` stays scoped to `6a84c2b09e38d37e4822bb0b27c414d9219b6efb`, 3/3. Parent accepted exact current register replay instead of duplication.

At canonical `98f2ef2286de7523a627fa1060a8f548b12a3b1b`, **81/81 current registers** reproduced byte-for-byte, all exit 0; **78/81 extracts** reproduced byte-for-byte. Exactly three inherited nonreproductions remain:

- 12.0.5: saved `4da3872aa566695f46e2dacd4e79992f5b06be9541f0d19cf0e8dba45cea8329`; generated `f446822f72eeb475e5b261657104cc87984c98079487b233c68988dc2cde202a`; exit 0, unequal bytes.
- 12.0.7: saved `014f7d51eca1b2fc5d76071978e09c537efd66d14069e21d163e58ccd04a561e`; generated `ad333a5b736549f66f6756386baad2039d1980315fdf6b6f650e9a9f157b1855`; exit 0, unequal bytes.
- 12.1.0: exit 1, exact `unhandled template: {{#description2:Midnight 12.1.0 (Curse of Ula’tek)}}`; saved `c051a68b442b2e69d602650d1c5ec07c1d0262dd1aea01fe73d3f60cd5f29c3b`.

Flags derived from each saved provenance, or `8.3.0-session-2026-10-08/p830-register-reproduction.json` / `p830-saved-extract-reproduction.json` when provenance lacks flags. No guessed option trials. Earlier inherited failure identities/error/saved hashes preserved; extractor source unchanged since `6a84c2b09` (actual diff retained). Generated register files retained under `/tmp/p308-independent-replay/`.

Plan `/tmp/p308-independent-replay-plan.json`; results/options/provenance `/tmp/p308-independent-reproduction.json`; exact per-command receipts `/tmp/p308-replay-{register,extract}-<patch>-result.json`.

3.1.0 register specifically reproduced with `--legacy-function-labels --client-line retail`, exit 0, byte-identical at `98f2ef228`; its extract also byte-identical with flags `[]`. Thus older fixture evidence is not falsely relabeled as current, and merged earlier parser behavior has current concrete replay.

Default parser comparison against archived `6a84c2b09` at canonical `efbe97f20fa49dd12c284f8e3505e16decd3d711`: **82/82 default outcomes unchanged**; 79 emitted outputs byte-identical and three identical pre-existing default errors: 5.4.2 `no symbol reference in: ': StartUnratedArena'`; 9.0.1 and 9.1.0 `no symbol reference in: ': Scripts'`. Physical old/new outputs `/tmp/p308-independent-default-replay/`; comparisons `/tmp/p308-independent-default-comparisons.json`; successful runner receipt `p308-independent-default-replay-v2`.

Initial default comparison harness at `98f2ef228` incorrectly assumed every opt-in page must also parse under defaults; aborted at old generator's 5.4.2 error, exit 1. Retained `p308-independent-default-replay-result.json`/log. Corrected only the temporary verification harness to compare successful bytes **and failure outcomes**; no repository fix/fallback. This is separate from the three inherited extract failures. No repeated global recorded-option register/extract gate after later integration; main owns next batched replay.

No 3.3 fixture rerun: fresh recorded-option replay covered its actual parser at `98f2ef228`; subsequent inspected generator diff adds independent launch parser and retains prior function/routing. 3.2 fresh focused conflict fixture below is warranted by explicit colliding routing.

## Separately requested canonical 3.0.3 source-only proof

Cwd `/home/osso/Projects/wow/wow-ui-sim`, revision `1ba5b66735ab09e6fcd7208f8e720662278eb3d6`; all start/end scopes unchanged; overrides `{}`.

| Receipt prefix | Exact argv | Result |
|---|---|---|
| p303-independent-source | `python3 tools/test_patch_3_0_3_source.py` | exit 0; 3/3, OK |
| p303-independent-validator | `python3 tools/test_patch_3_0_3_validator.py` | exit 0; 2/2, OK |
| p303-independent-historical | `python3 data/patch-api/evidence/3.0.3-session-2026-10-09/validate.py` | exit 0; 16 seals |

History retains three inventory / eight ledger IDs, statuses two metadata / three publication-unproven / three semantic-unproven, six contract-gap records, no signatures/meaningful closures. Four integrated successor-closure register hashes match actual current files and all exact CVar overlaps are empty; `/tmp/p303-independent-successor-comparison.json`. Both CVar-definition and labeled-summary flags retained in inspected merged generator and all-register replay at `98f2ef228`. No native/model/synchronization semantics credit; no Cargo runs for 3.0.3.

## Separately requested canonical 3.0.2 source-only proof

Latest bounded commands ran at `323fc333efdf9951922ff5d487af6cc0ee29bdcc` (later than requested `efbe97f20`; intervening HEAD is 3.1 documentation-only). All start/end scopes unchanged; overrides `{}`.

| Receipt prefix | Exact argv | Result |
|---|---|---|
| p302-independent-source | `python3 -B tools/test_patch_3_0_2_source.py` | exit 0; 4/4, OK |
| p302-independent-accounting | `python3 -B data/patch-api/evidence/3.0.2-session-2026-10-09/test_accounting.py` | exit 0; 4/4, OK |
| p302-independent-historical | `python3 -B data/patch-api/evidence/3.0.2-session-2026-10-09/validate.py` | exit 0; 36 original seals / one closure seal |
| p302-independent-p320-parser | `python3 -B tools/test_patch_3_2_register.py` | exit 0; 1/1, OK |
| p302-independent-seals | `python3 -B -c <exact retained SHA256 comparison program>` | exit 0; 36 original / one closure / two receipt seals, zero hash failures |

Source/count proof: 373 inventory occurrences (279 added /25 changed /69 removed), 431 source rows (389 UNPROVEN /42 metadata), 41 headers, 294 explicit signature texts, 367 signature limits, 80 prose limits. Native observations 0, runtime observations 0, meaningful closures 0. Historical actual successors remain 3.3.0 /3.3.3 /3.3.5 /4.0.1; queued records remain frozen separately, not rewritten by current integration. Factory publication measurement remains separate-worker owned.

Inspected generator diff `98f2ef228` → `efbe97f20` retains `wrath_retail_change_bullets` and adds `wrath_launch_inventory` without replacing prior route. Fresh 3.2 fixture exercises retained updated/removed uncertainty, event/table identities, literal signatures, and default-byte restoration. `/tmp/p308-independent-p302-parser-diff.log` retains full diff. No repeated 81-register/78-extract gate; no additional Cargo gate.

## Rust readability and limitations

Manually audited complete 114-line new `tests/patch_3_0_8_publication_sweep.rs` against rust-readability: two concrete test bodies below 200-line test threshold; declarative successor list; no deep nested flow, excessive primitive parameters, hidden model mutation, duplicate helper bodies, or warning suppression. File I/O through existing sweep/output env is explicit in specification. No in-scope readability violations identified. Marker counts TODO/FIXME/HACK/XXX/allow each zero. Actual successful named prefork cases prove wiring; no source-substring assertion used as behavior proof.

No edits requested or made. Native historical security/taint, stereo defaults, glyph-link semantics, animation/control restricted environment, universal ScriptObject, name/token, Calendar sentinel/state transitions and other six pending prose contracts remain UNPROVEN. Zero new 3.0.8/3.0.3/3.0.2 models or semantic closures. Worktree stable native proof is not promoted to later canonical runtime proof. Parent-provided CI success at `1ba5b6673` was not independently fetched; no CI conclusion here.

## Later assigned retained full-suite inspection — no rerun

Inspected artifacts present at canonical `44df100f1b553c5d717af9d8c14d3405823ecfa7`: `data/patch-api/evidence/3.1.0-session-2026-10-09/integrated/full-suite-result.json` and `full-suite-comparison.json`. Native run scope is **only** `614402d56f726ec34cedea2f28a9609cdd653462`; later 3.0.8/3.0.3/3.0.2 additions are not covered by it.

Read complete existing log (2115736 bytes /16881 lines), parsed all three sections and failure IDs, not merely parent summary. Exact log path `/home/osso/Projects/wow/full-suite-results/614402d56f726ec34cedea2f28a9609cdd653462.log`; SHA256 `a3efd6c27231a0b93c2b24064db588278f02a0e7aa5bf31a289969a0c2871bb3` matches retained comparison. Current result SHA256 also matches `ce013458b822e883a0442e495186cfa37d7ef0f78ba6d8cce95774af334248b8`.

| Section | Exact existing log summary | Exit / verdict |
|---|---|---|
| Integration | 10673 run: 10650 passed (8 slow), **23 failed**, 19 skipped | 100 / FAIL |
| Prefork | **2318 passed /1 failed /2319 total** | 1 / FAIL |
| Lib | 1979 run: 1973 passed, **6 failed**, 0 skipped | 100 / FAIL |

All 30 unique failure IDs reconstructed from full log equal retained current result. Independently read `/home/osso/Projects/wow/full-suite-results/c17f1b4bb1c66e1b5012b3f301147805153b7f87.json`: all three failure lists exactly equal, **zero added /zero removed** in every section. Garrison remains baseline-new relative to runner master reference, present in both old/current `new_failures.prefork`; do not relabel it passing or resolved. Actual prefork error at log line12623: `Blizzard_AdventuresCombatLog.lua:90`, bad argument #1 to ipairs (table expected, got nil).

Full log contains **74 exact `::patch_<version>_publication_sweep ... ok` cases**, plus two separate passing animation-factory and 3.1 radians cases. All 76 lines equal retained comparison list; no failed publication case. This is not 76 publication sweeps. The 3.0.8 native evidence remains its separate fresh 2/2 receipt.

Detailed independent full-log/failure-delta/hash evidence: `/tmp/p308-independent-full-suite-log-inspection.json`. No test or broad command rerun. **Broad suite FAIL with unchanged baseline—not acceptance.**
