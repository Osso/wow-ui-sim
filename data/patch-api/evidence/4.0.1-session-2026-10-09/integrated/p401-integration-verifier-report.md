# Independent bounded 4.0.1 integration verification

Repository: `/home/osso/.worktrees/wow-ui-sim-p401-source`.
Verified HEAD: `3e9003b64367d9a7c733401d7e6aba8c64ee4eb3`; base `9ff195d19` (caller shorthand `9ffmain` is not a Git ref). Initial/final status clean. No repository edits, delegation, Bash, session cwd switch, commits, Rust tests/check/build invocation, shared validator gate, native/full-suite/CI-parity claim.

## Fresh command proof

All commands executed with `.capture().run()`; every exit/stdout/stderr retained. Exact SHA-256 source scope: `/tmp/p401-independent/scope.json`; each command receipt also embeds that scope.

| Command | Result | Exact full receipt |
|---|---|---|
| `python3 -B tools/test_gen_patch_wikitext_register.py` | PASS, 35 tests, exit 0 | `/tmp/p401-independent/command-0.json` |
| `python3 -B tools/test_extract_patch_non_inventory.py` | PASS, 37 tests, exit 0 | `/tmp/p401-independent/command-1.json` |
| `python3 -B tools/test_patch_cataclysm_register.py` | PASS, 3 tests, exit 0 | `/tmp/p401-independent/command-2.json` |
| `python3 -B data/patch-api/evidence/4.0.1-session-2026-10-09/validate.py` | PASS, exit 0 | `/tmp/p401-independent/command-3.json` |
| `cargo fmt --check` | PASS, exit 0, stdout/stderr empty | `/tmp/p401-independent/command-4.json` |

Python fixture output, verbatim:

```text
...................................
----------------------------------------------------------------------
Ran 35 tests in 0.440s

OK
.....................................
----------------------------------------------------------------------
Ran 37 tests in 0.148s

OK
...
----------------------------------------------------------------------
Ran 3 tests in 0.127s

OK
```

Historical-validator output, verbatim:

```text
PASS: {"historical_only": true, "inventory": 419, "prose_limits": 4, "publication_gaps": 117, "retired_globals": ["CollapseSkillHeader", "ExpandSkillHeader"], "sealed_evidence": 42, "sealed_inputs": 84, "signature_limits": 1, "source_rows": 428, "statuses": {"audit-pending": 122, "bounded-coverage": 302, "metadata-only": 4}, "successor_registers": 64}
```

Caller reports the five fresh-process v2 validator fixtures GREEN. Those fixtures were read, not rerun; this verifier independently ran the documented own validator once. No fresh five-fixture execution is attributed to this verifier.

## Concrete contract matrix

| Contract | Evidence and result | Missing/blocked scope |
|---|---|---|
| Retail retirement | Only `CollapseSkillHeader` and `ExpandSkillHeader` excluded from `GLOBAL_NIL_STUBS` registration under `retail-12-0-0`; both remain in original list. Current runtime files and retirement test are byte-identical to sealed GREEN inputs. Original retail factory 1/1, cached 2/2 and Mists factory 1/1 receipts apply to unchanged retirement model/test inputs. | Cached historical 2/2 includes the old publication sweep, whose expectations have since changed; old 302/117 accounting is historical only. Mists cached command was blocked by retail-only harness features, not a passing behavioral control. Other Classic execution unproven. |
| Current retail publication | Main receipt at exact verified HEAD: `test result: ok. 70 passed; 0 failed; 70 total`, exit 0. Own result has all 419 IDs, 304 matches, 115 gaps; live gap fixture contains exactly 115 IDs. Ledger has 428 rows: 304 bounded, 120 pending, 4 metadata. | Publication/absence only; four prose contracts plus one numeric-input signature remain unproven. |
| Actual 4.x successors | Sweep includes actual 4.1.0/4.2.0/4.3.0/4.3.4 registers before 5.x. Exact historical/current result delta consists only of `wt-global-api-CanTransform-105` and `wt-global-api-Transform-336`. 4.1 removals at `wt-global-api-CanTransform-8`/`wt-global-api-Transform-8` make expected publication absent; observations remain `raw=nil; lookup=nil`. | No transformation behavior/model credit. Delta: `/tmp/p401-independent/publication-delta.json`. |
| Historical isolation | Own validator passes: 419 inventory, original 302 covered/117 gaps, negative 118 gaps retained in sealed evidence, 84 pinned inputs, 42 evidence seals, 64 historical successors. Preserved v1 validator/context hashes exact. Current context retains all 40 original evidence seals and adds two v1-file seals (42 total). | Historical proof is not current native/runtime replay. |
| v2 portability fix | Exact v1/v2 code diff is one deletion: `dir=ROOT / 'target'` from `TemporaryDirectory` at validate.py:119. No fallback or repo target creation. v2 hash `12bd752b78ff9d47f91cec474dceca6d48b991faab43d6287bc2ec94e2fa6606`; v1 hash `ab28d73b49bef4a6e428fc060258dbe0f5d5cc53e1490db5b985249a8bb91cc6`; v1 context `a7ae9cdfb4506ff99f15e4254ddc464fe11723b9563aa662f7d26b446c7c92c0`. | Global portable gate remains main-owned; not run here. |

Runtime retirement applicability hashes (historical/current identical):

- `src/c_api/mod.rs`: `cc8212c4ef56c59923902c373d2c81107acb3d0e69d648bfaec03f551fb3f70a`.
- `src/c_api/patch_retired_members.rs`: `8e7fdc4d44971730139476b45723b1bd9a3d7777129f0fa4deb18b734f3984d5`.
- `src/lua_api/globals/stubs/global_stubs.rs`: `e58910706b30395dc8a85107b1d29d16b7aa7e1e3132bb79dd967983abcd546f`.
- `tests/patch_4_0_1_retirements.rs`: `bb4d064857e527ad5b27f267077a1b16e12038a726e2c586d94c5c8434827f3e`.
- `tests/common/publication_sweep.rs`: `9806ecce8fad716f3c3c0e3faaec53573b881c8630da8f6b2fe8caebc2247837`.

Registration, widget-type/method lookup and existing fallbacks are unchanged by the bounded runtime diff. No new fallback introduced.

## Saved register/extract replay on NEW parser bytes

Exact argv/provenance path/hash, parser hash, exit, full stdout/stderr, output existence and byte hashes for all 151 final replay rows: `/tmp/p401-independent/replay-receipts.json`. All outputs confined to `/tmp/p401-independent/`.

- Registers: **74/74 byte-identical, all exits 0**.
- Extracts: **74/77 byte-identical**; 76 commands exit 0, one exit 1/no output. Exactly the three inherited failures retained:
  - `12.0.5`: exit 0, output exists, byte mismatch. Expected `4da3872aa566695f46e2dacd4e79992f5b06be9541f0d19cf0e8dba45cea8329`; actual `f446822f72eeb475e5b261657104cc87984c98079487b233c68988dc2cde202a`.
  - `12.0.7`: exit 0, output exists, byte mismatch. Expected `014f7d51eca1b2fc5d76071978e09c537efd66d14069e21d163e58ccd04a561e`; actual `ad333a5b736549f66f6756386baad2039d1980315fdf6b6f650e9a9f157b1855`.
  - `12.1.0`: exit 1, no output, **not a byte mismatch**. `tools/extract_patch_non_inventory.py:247`: `ValueError: unhandled template: {{#description2:Midnight 12.1.0 (Curse of Ula’tek)}}`.

Current generator SHA-256: `f98f56fd0488fd1560e1dc5257d8cb76d0d960aca1df701a01bf868fe93ba2a1`.
Current extractor SHA-256: `0cbacb477244db36fea9a473ee980c5a53dc5d673760e9ed5eba171964642af2`.

Recorded flags explicitly restored for replay: 10.0.0 `--expand-shared-changes`, 12.1.0 `--inventory-only`, six `--preserve-examples` cases, and the 5.0.4 recorded diff input. Basis: committed `data/patch-api/evidence/5.2.0-session-2026-10-08/reproduction.json` and page provenance, not inferred flags. 4.4.x extract flags come from each own `page-coverage.json` non_inventory_source (`--canonical-patch-navigation`) and the recorded evidence commands; no nonexistent source-directory provenance assumed.

Verifier initial replay used incomplete default-flag metadata and omitted the 5.0.4 evidence diff from the temporary snapshot. These initial nonmatching/error results remain in `/tmp/p401-independent/replay-initial.json`; only those affected rows were corrected and rerun with explicit recorded inputs/flags. Correction receipts: `/tmp/p401-independent/replay-corrections.json`. No concurrency attribution, no broad rerun for logs, no lost return codes. The corrected final set retains the exact inherited failures above.

## Main-owned receipts observed, not rerun

All three start receipts pin exact verified HEAD `3e9003b64367d9a7c733401d7e6aba8c64ee4eb3`:

- `/tmp/p401-integrated-publication-{start,result}.json`: publication filter, 70/70 tests, exit 0.
- `/tmp/p401-mists-check-{start,result}.json`: `cargo check --no-default-features --features sound,gui,casc,client-mists --tests`, exit 0; compilation only, not native/Mists behavioral execution.
- `/tmp/p401-runtime-build-{start,result}.json`: `cargo build --bin wow-sim`, exit 0; no runtime smoke proof.

Full stdout/stderr read from saved receipts and separately retained in `/tmp/p401-independent/p401-*-result-{stdout,stderr}.txt`. Receipt/source hashes and warning lines: `/tmp/p401-independent/main-receipt-summary.json`. Six inherited iced manifest deprecation warnings remain; publication output also contains CVar-default mismatch diagnostics. No zero-warning/default-parity claim.

## Rust readability / artifact checks

Read full rust-readability skill and audited actual changed Rust lines plus own retirement/publication tests. No introduced readability violations found. Retirement test is 32 lines, explicitly asserts normal/raw absence for retail and invokes both original no-ops for non-retail. Sweep's long block is a declarative ordered successor list, not branching logic. Existing registration function length is inherited, not redesigned here.

[EXIST] PASS — actual runtime files and tests exist; line counts and exact hashes in scope.json.
[SUBSTANTIVE] PASS — real cfg-gated exclusion and behavioral Lua assertions, not placeholders. Existing no-op functions are the deliberately preserved Classic surface.
[WIRED] PASS — `src/c_api/mod.rs:315` exports list; `global_stubs.rs:309` consumes it; `stubs/mod.rs:144` invokes global registration. `build.rs:26,114-122` generates prefork registry; integration.rs:1 and prefork_full_ui.rs:13,188 include generated tests. Main's actual 4.0.1 sweep receipt passes.
[ANTI-PATTERN] PASS — changed Rust additions contain zero TODO/FIXME/HACK/XXX or warning suppressions.

## In-scope documentation discrepancy

`docs/specs/patch-4-0-1-publication-sweep.md:11` still requires pending 4.x placeholders and says main replaces them later; current code already uses actual registers. Its current-gap paragraph says 117 publication mismatches, while current ledger/fixture/receipt show 115. Historical 117 must stay untouched; current spec wording needs reconciliation by main. Reported to parent, not edited.

Overall: fresh bounded parser/validator/fmt and retirement applicability PASS; all register replay PASS; three inherited extract failures remain FAIL; current spec wording stale. Main owns global portability and runtime/native/full-suite acceptance.
