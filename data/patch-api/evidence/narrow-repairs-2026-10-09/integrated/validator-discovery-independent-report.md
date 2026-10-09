# Independent verification — validator discovery

PASS for narrow discovery repair and invalidated global validator gate only. Not overall acceptance.

## Revision and invocation

- Canonical cwd: `/home/osso/Projects/wow/wow-ui-sim`.
- Exact HEAD: `894680abe9e7282453152f6372c34680d61e2cf0` before and after; tracked input diff empty both times. Pre-existing untracked `.code-index.db` unchanged.
- Required command executed once: `python3 -B tools/check_patch_validators.py 894680abe`.
- Executed through `python3 -B /tmp/p434-acceptance-runner.py validator-discovery-independent-global /home/osso/Projects/wow/wow-ui-sim '["python3", "-B", "tools/check_patch_validators.py", "894680abe"]' '{"PYTHONDONTWRITEBYTECODE": "1"}'` (argv execution, no shell).
- Runner SHA256: `71ba3178c7ced1ec04af5a8414e99fb939d77b33a1db9708e743e5aeda01ffde`.
- Environment: inherited process environment plus `PYTHONDONTWRITEBYTECODE=1`; exact inherited+override snapshot `/tmp/validator-discovery-independent-env.json`.
- UTC start/end: `2026-10-09T14:14:23.149218+00:00` / `2026-10-09T14:18:00.552045+00:00`.
- Exit 0; gate status PASS; outer stderr empty; full stdout 160510 bytes.

## Exact execution coverage

Inventory derived from actual filesystem discovery and compared by exact validator paths with complete gate receipts: 90 discovered, precisely three template-inputs snapshots excluded, 87 genuine validators retained.

| Phase | Executed/pass | Failed | Missing genuine paths | Unexpected paths | Archived snapshots executed |
|---|---:|---:|---:|---:|---:|
| clean pinned revision | 87/87 | 0 | 0 | 0 | 0 |
| synthetic unrelated later audit | 88/88 | 0 | 0 | 0 | 0 |

175 total executions; later phase adds exactly `data/patch-api/evidence/9.9.9-synthetic-later-audit/validate.py`. Synthetic detached revision: `233b12ecf3d407cf34a78e9a9196a55e499211b1`. No oversized tracked evidence in either phase. Gate-owned temporary worktree cleaned up; existing development worktrees untouched.

Excluded exact paths:

- `data/patch-api/evidence/2.5.2-session-2026-10-09/template-inputs/2.5.5/validate.py`
- `data/patch-api/evidence/2.5.2-session-2026-10-09/template-inputs/2.5.6/validate.py`
- `data/patch-api/evidence/2.5.2-session-2026-10-09/template-inputs/3.4.3/validate.py`

These are absent in both phase execution receipts; all 87 genuine paths present exactly once per phase. The genuine 2.5.2 validator runs successfully and reports 61 sealed inputs, 198 source rows, 151 unproven contracts, zero native/runtime/model observations. Validator success does not turn these unproven contracts into implemented coverage.

## Source/test inspection

Read complete `tools/check_patch_validators.py`, `tools/test_check_patch_validators.py`, repair diff at 894680abe, and preceding test diff at 7d91acfd5.

- Helper lines 30–39: only exact relative path component `template-inputs` is excluded. Root validators and arbitrary other nested validators remain in recursive discovery; every retained subprocess exit/stdout/stderr is recorded. No missing-seal/input exception skip added.
- Docstring explicitly describes archived snapshots versus other nested/current proofs.
- Actual subprocess fixtures create ephemeral repositories, exercise clean+later phases, and assert original repository state/worktree cleanup. New tests prove nested genuine current execution and genuine missing-seal failure while skipping archived missing-seal snapshots. Existing root failing-dependency fixtures prove root failures propagate.
- Reused, not rerun: `/tmp/validator-discovery-red-result.json` records 2/2 expected failures at `7d91acfd5bec7eafa927b90dc7d9bf3a34b20c07`; `/tmp/validator-discovery-green-result.json` records 8/8 tests OK, exit 0 at exact repair HEAD. Logs: `/tmp/validator-discovery-red.log`, `/tmp/validator-discovery-green.log`.
- Limit: no dedicated root missing-seal fixture was added or rerun; root failure propagation has existing GREEN evidence plus direct inspection of the identical retained execution path. The new missing-seal fixture is nested current, not root.

## Preservation and receipts

65 canonical seal/template files hashed before/after, byte-identical, with identical path sets. No source changes, fabricated seals/sidecars, network, delegation, Bash, native checks/builds/formatters, source50 rerun, default reproduction, or full-suite rerun. The sole authorized gate internally creates its existing temporary detached checkout and synthetic audit commit; verifier did not create development branches or commits.

Full receipts:

- `/tmp/validator-discovery-independent-global-start.json`
- `/tmp/validator-discovery-independent-global-result.json` — exact command/cwd/overrides/start/end/code scope/full separate stdout+stderr
- `/tmp/validator-discovery-independent-global.log` — full combined output
- `/tmp/validator-discovery-independent-global-stdout.json`
- `/tmp/validator-discovery-independent-global-stderr.txt`
- `/tmp/validator-discovery-independent-discovery.json` — exact path-set comparison and derived counts
- `/tmp/validator-discovery-independent-streams/manifest.json` — all 175 executions, per-execution full stdout/stderr files
- `/tmp/validator-discovery-independent-output-review.json` — complete output parsing/review evidence, including full non-single-JSON streams and test stderr
- `/tmp/validator-discovery-independent-diff.txt`
- `/tmp/validator-discovery-independent-env.json`
- `/tmp/validator-discovery-independent-protected-before.json`
- `/tmp/validator-discovery-independent-protected-after.json`

## Acceptance limits

User-supplied parent fullsuite614 remains FAIL with unchanged 23+1+6 failures and 74 publication checks OK; not rerun or independently re-established here. Prior owned proofs 50/50 + six history + six copied remain prior evidence, not fresh executions claimed here. Unrelated native profile compile blocker not investigated. No overall acceptance claim. Documentation audit, if requested, remains separate.
