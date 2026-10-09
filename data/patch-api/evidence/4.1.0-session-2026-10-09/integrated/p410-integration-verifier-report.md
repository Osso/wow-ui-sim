# Independent bounded 4.1.0 integration verification

Repository: `/home/osso/.worktrees/wow-ui-sim-p410-source`.
Requested HEAD: `287c1fc455c78df7ff4bed12c302c08083e19b99`, rebased on `34c303c10`.
Final observed HEAD: `61689f258ca48c585e7a864e08ea412454b3e1bb`.
The ten verified runtime/parser/test/validator files have no diff between those revisions. Exact SHA-256 scope: `/tmp/p410-integration-verifier-scope.json`. Later changes add documentation and retained main-owned acceptance artifacts; they do not invalidate these bounded checks.

## Fresh commands

Exact argv, revision, exits and complete output files: `/tmp/p410-integration-verifier-commands.json`.

| Command | Result | Full output |
|---|---|---|
| `cargo fmt --check` | PASS, exit 0, empty output | `/tmp/p410-integration-verifier-fmt.log` |
| `python3 -B /home/osso/.worktrees/wow-ui-sim-p410-source/tools/test_gen_patch_wikitext_register.py` | PASS, 35 tests, exit 0 | `/tmp/p410-integration-verifier-parser.log` |
| `python3 -B /home/osso/.worktrees/wow-ui-sim-p410-source/tools/test_extract_patch_non_inventory.py` | PASS, 37 tests, exit 0 | `/tmp/p410-integration-verifier-extractor.log` |
| `python3 -B /home/osso/.worktrees/wow-ui-sim-p410-source/tools/test_patch_4_1_register.py` | PASS, 1 test, exit 0 | `/tmp/p410-integration-verifier-p410-parser.log` |
| `python3 -B /home/osso/.worktrees/wow-ui-sim-p410-source/data/patch-api/evidence/4.1.0-session-2026-10-09/validate.py` | PASS, exit 0 | `/tmp/p410-integration-verifier-historical.log` |

Fixture outputs, verbatim:

```text
...................................
----------------------------------------------------------------------
Ran 35 tests in 0.165s

OK
.....................................
----------------------------------------------------------------------
Ran 37 tests in 0.039s

OK
.
----------------------------------------------------------------------
Ran 1 test in 0.110s

OK
```

## Coverage and preservation

| Contract | Observed evidence | Missing proof |
|---|---|---|
| Current retail publication | Main's saved receipt: exit 0; `test result: ok. 69 passed; 0 failed; 69 total`. 4.1 results contain 81 rows, 51 matches, exact 30-gap fixture. | Publication does not establish native signatures or behavior. |
| Actual successor | `/home/osso/.worktrees/wow-ui-sim-p410-source/data/patch-api/sources/4.3.0-wikitext-register.json` removes `IsIPv6Available` at `wt-global-api-IsIPv6Available-94`. Integrated raw/lookup absence matches. Sole ledger-row change is `wt-global-api-IsIPv6Available-73`; 50/31 becomes 51/30. | No IPv6 backing-model credit. |
| Source accounting | 171 unique ledger IDs = 81 inventory + 86 extraction + 4 signatures. Current statuses: 51 bounded, 78 metadata, 42 pending. Historical statuses remain 50 bounded, 78 metadata, 43 pending. | Eight prose rows and four signatures remain unproven. |
| Historical isolation | Validator verifies 23 original seals, 78 archived blobs, three selected trees and 64 historical later-retail registers. Two explicit sidecar mappings preserve original ledger/31-gap fixture; no fallback to current files. Negative remains 32 gaps. | Historical selected trees are not complete native/runtime replay. |
| Pet behavior | Original retail 1/1 and supported Mists 2/2 receipt scopes apply to byte-identical current pet implementation and tests. | No fresh Mists compilation or native lifecycle proof from this verifier. |

Main sweep receipt was not rerun. Full saved stdout/stderr was read and copied to `/tmp/p410-integration-verifier-main-full-output.log`; six iced manifest warnings and CVar-default diagnostics remain visible. Its start revision is `fa89afcaa5cf8b86a4a2faee204bf778c2f63f1f`; subsequent changes are evidence isolation, docs and receipt retention, not runtime/publication-input changes.

Pet applicability hashes at original GREEN revision `4f806105badd5453d5f0d5f561bf54b11c2e1380`:

- `/home/osso/.worktrees/wow-ui-sim-p410-source/src/lua_api/globals/real/pet_stats.rs`: archived blob `b198f8abb254dc1a76fc7f33d310f8862bf90908`; historical/current SHA-256 `5a94b3fad13708a8c473cbf64bdbf85416b3f530c4947515fcbd4a9e8aa3b308`.
- `/home/osso/.worktrees/wow-ui-sim-p410-source/tests/pet_stats.rs`: archived blob `f211afd1a2c14b3787520a5d6ebaf2b278e78f3e`; historical/current SHA-256 `a0c1583369b22fbe57101b20810f288678af5f75c88fe5ca2c9f0ceb690a05ad`.

Exact comparisons, including pre-fix snapshots: `/tmp/p410-integration-verifier-pet-applicability.json`.

The current validator fixture explicitly exercises mutated live ledger/gap inputs, sealed sidecar tampering, missing sidecars, source tampering and log tampering. It is unchanged across the supplied rebase/current scope. The caller's already-GREEN targeted fixture was not redundantly rerun; only the own historical validator ran fresh.

## Saved register/extraction replay

Exact data results, flags, hashes and errors: `/tmp/p410-integration-verifier-replay.json`; helper output `/tmp/p410-integration-verifier-replay.log`; generated artifacts `/tmp/p410-integration-verifier-replay/`.

- 73/73 saved registers reproduce byte-identically.
- 73/76 saved extracts reproduce byte-identically. All three inherited failures are retained, not accepted or waived:
  - `12.0.5`: output mismatch; expected SHA-256 `4da3872aa566695f46e2dacd4e79992f5b06be9541f0d19cf0e8dba45cea8329`, actual `f446822f72eeb475e5b261657104cc87984c98079487b233c68988dc2cde202a`.
  - `12.0.7`: output mismatch; expected `014f7d51eca1b2fc5d76071978e09c537efd66d14069e21d163e58ccd04a561e`, actual `ad333a5b736549f66f6756386baad2039d1980315fdf6b6f650e9a9f157b1855`.
  - `12.1.0`: replay error at `/home/osso/.worktrees/wow-ui-sim-p410-source/tools/extract_patch_non_inventory.py:243`: `ValueError: unhandled template: {{#description2:Midnight 12.1.0 (Curse of Ula’tek)}}`.

Recorded command-string/receipt flags matter: 10.0.0 requires `--expand-shared-changes`; 12.1.0 requires `--inventory-only`. Six extraction rows require recorded `--preserve-examples`. Explicit basis is the committed `/home/osso/.worktrees/wow-ui-sim-p410-source/data/patch-api/evidence/5.2.0-session-2026-10-08/reproduction.json`, which also records exactly these three inherited extraction failures. 4.4.x uses its own source-only audit spec's documented `--canonical-patch-navigation`, not an invented source-directory provenance file. Initial wrong/default-flag observations remain separately retained at `/tmp/p410-integration-verifier-replay-initial.json`.

Receipt limitation: initial command-loop capture was interrupted by an incorrect 4.4.0 provenance lookup. Completed register artifacts were preserved; 71 individual subprocess exits were not persisted before that error, so their exit telemetry is explicitly null, not fabricated. Two corrected-flag register commands have captured exit 0. No broad command rerun recovered logs. Byte identity is proven independently by saved generated bytes; this is not a complete 73-command exit receipt.

## Rust readability and artifact gate

Read `/home/osso/AgentConfig/skills/verify/SKILL.md` and the full rust-readability skill. Manual changed-line audit of pet stats, retirement tests and 90-line sweep include found no changed-code readability violations. The sweep's long block is a declarative successor inventory, not branching business logic. No TODO/FIXME/HACK/XXX or warning suppressions in these three files.

[EXIST] PASS: pet implementation 78 lines, tests 131 lines, sweep 90 lines.
[SUBSTANTIVE] PASS: retail handler/registration gating; raw and ordinary absence assertion; preserved non-retail default/seeded state assertions; real successor includes.
[WIRED] PASS: `/home/osso/.worktrees/wow-ui-sim-p410-source/src/lua_api/globals/register.rs:301` calls pet registration; real module declares it at `real/mod.rs:55`. `/home/osso/.worktrees/wow-ui-sim-p410-source/build.rs:54-90,463-497` discovers top-level test modules; integration/prefork harnesses include generated modules. Saved main receipt executes the actual 4.1 sweep.
[ANTI-PATTERN] PASS: zero named markers/suppressions in bounded Rust files.

## Boundaries and verifier incidents

No Bash, cwd switch, delegation, source edits, commits, check/build, native/CI/full-suite or shared-portability run. No main sweep rerun. Main owns stable Mists retry, runtime acceptance and final shared portability; no claim about those pending gates here.

Initial concurrency attribution was incorrect and retracted: absent `/home/osso/.worktrees/wow-ui-sim-p410-source/data/patch-api/sources/4.4.0-api-changes.provenance.json` is an audit-layout error, not evidence of mutation. There are no before/after hashes or writer evidence for that absent file. Direct Python module loading briefly generated one verifier-owned extractor `.pyc`; that exact file and its otherwise-empty directory were removed. Final git status is clean. Incidents are recorded at `/tmp/p410-integration-verifier-incidents.json`.

Overall: bounded code/artifact checks PASS; three inherited extraction failures remain FAIL; register-command exit capture is incomplete. No all-green/current-project acceptance claim.
