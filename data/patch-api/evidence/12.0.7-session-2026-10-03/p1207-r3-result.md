Rebase step git stash -u: exit 0.
Rebase step git rebase master: exit 0.
Rebase step git stash pop: exit 1.
Explicit worktree: git stash -u exit 0.
Explicit worktree: git rebase master exit 0.
Explicit worktree: git stash pop exit 0.
Initial Pyrun commands used canonical cwd (clean master); no changes there. All further commands specify worktree cwd. Active goal: six-CVar slice; RED/GREEN, focused regressions, fmt/check, one commit. Exclusions as original task.

Step 1: recovered contract. Existing src/cvars.rs already has all six defaults, epoch insertion and explicit per-storage overrides; global/namespace APIs share it. No missing producer established. RED will temporarily withdraw only these six default rows (saved in persistent audit dir), then restore them for GREEN. Default/coercion policies remain INFERRED. No alternate feature sets authorized; strict publication gate remains unproved.
Step 2 preparation: baseline producer backed up as r3-cvars-before-red.rs; temporarily removing six built-in default tuples only, no API replacement. RED hypothesis: all ten tests fail at initial/default assertions when producers absent.
Proof invocation RED patch_12_0_7_cvars: rebased base 9ce93bb12 + surviving tests + six-default withdrawal, cvars SHA256 35b94e61ba9de9fe14459148f026a9f68f2608cca335922200b7d205e9bc2e0d; no earlier test proof exists. Command: CARGO_BUILD_JOBS=4 BUILD_HOST_SCRIPTS=/home/osso-test/Projects/world-of-osso/game-engine/scripts python3 scripts/build-host.py --build-host local --test --target-dir /home/osso-test/.cache/wow-ui-sim-target-b100 --test integration patch_12_0_7_cvars -- --test-threads=1.
RED invocation finished exit 101; output captured once to /home/osso-test/.cache/wow-ui-sim-audit/r3-RED-patch_12_0_7_cvars.log.
Step 2 RED: 0 passed / 0 behavioral failures / 10 not executed; compile blocked by three E0433 errors in unchanged master src/c_api/c_spell_maw_powers.rs:48,50,59. Its Val import is cfg(not(retail-12-0-7)), while enabled-epoch mark_border_atlas_removed uses Val. No RED behavioral proof obtained.
Step 3: restored six pre-existing default tuples unchanged; added INFERRED comments only. Existing explicit storage/API producers need no behavior change. Out-of-scope master compile blocker retained.
Proof invocation GREEN patch_12_0_7_cvars: master 9ce93bb12 + survivor tests + restored default values / INFERRED comments; cvars SHA256 8d630ffc5a1da181b9c9f30cd63dafc40320f7ea4368fec7b88d5699dcce427a. RED proof unavailable; compiler blocker expected. Required command uses one filter, jobs=4, local host and exclusive b100 target.
GREEN patch_12_0_7_cvars invocation exit 101; captured once to /home/osso-test/.cache/wow-ui-sim-audit/r3-GREEN-patch_12_0_7_cvars.log.
Proof invocation GREEN set_cvar_global: master 9ce93bb12 + survivor tests + restored default values / INFERRED comments; cvars SHA256 8d630ffc5a1da181b9c9f30cd63dafc40320f7ea4368fec7b88d5699dcce427a. RED proof unavailable; compiler blocker expected. Required command uses one filter, jobs=4, local host and exclusive b100 target.
GREEN set_cvar_global invocation exit 101; captured once to /home/osso-test/.cache/wow-ui-sim-audit/r3-GREEN-set_cvar_global.log.
Proof invocation GREEN cvar_bitfields: master 9ce93bb12 + survivor tests + restored default values / INFERRED comments; cvars SHA256 8d630ffc5a1da181b9c9f30cd63dafc40320f7ea4368fec7b88d5699dcce427a. RED proof unavailable; compiler blocker expected. Required command uses one filter, jobs=4, local host and exclusive b100 target.
GREEN cvar_bitfields invocation exit 101; captured once to /home/osso-test/.cache/wow-ui-sim-audit/r3-GREEN-cvar_bitfields.log.
Proof invocation GREEN test_cvar_display_settings: master 9ce93bb12 + survivor tests + restored default values / INFERRED comments; cvars SHA256 8d630ffc5a1da181b9c9f30cd63dafc40320f7ea4368fec7b88d5699dcce427a. RED proof unavailable; compiler blocker expected. Required command uses one filter, jobs=4, local host and exclusive b100 target.
GREEN test_cvar_display_settings invocation exit 101; captured once to /home/osso-test/.cache/wow-ui-sim-audit/r3-GREEN-test_cvar_display_settings.log.
Step 3 log review: each GREEN filter fails before executing tests with the same three E0433 errors. Each reports 0 passed / 0 behavioral failures; no GREEN proof. Strict enabled/disabled epoch runs not attempted (alternate feature sets forbidden).
Step 4 cargo fmt: exit 0, jobs=4; captured once to /home/osso-test/.cache/wow-ui-sim-audit/r3-fmt.log. Scope: restored six defaults, survivor tests and INFERRED comments on rebased master.
Step 4 cargo fmt --check: exit 0, jobs=4; captured once to /home/osso-test/.cache/wow-ui-sim-audit/r3-fmt-check.log. Scope: restored six defaults, survivor tests and INFERRED comments on rebased master.
Step 4 cargo check --target-dir /home/osso-test/.cache/wow-ui-sim-target-b100 --jobs 4: exit 101, jobs=4; captured once to /home/osso-test/.cache/wow-ui-sim-audit/r3-check.log. Scope: restored six defaults, survivor tests and INFERRED comments on rebased master.

Step 4 review: fmt and fmt --check passed; cargo check failed with unchanged master E0433 import gate. No warnings suppressed. Manual Rust-readability review of changed Rust scope: straight-line Rust wrappers, shared Lua assertions, bounded fixture loops, no new warning suppressions or producer complexity. Coverage JSON byte comparison to HEAD unchanged. Net producer behavior identical to master; only two INFERRED-default comments added.
## Verification counts (not acceptance proof)

| Phase | Filter | Passed | Behavioral failures | Result |
|---|---|---:|---:|---|
| RED | patch_12_0_7_cvars | 0 | 0 | compile blocked; all 10 unexecuted |
| GREEN | patch_12_0_7_cvars | 0 | 0 | compile blocked; all 10 unexecuted |
| GREEN | set_cvar_global | 0 | 0 | compile blocked; negative epoch control also disabled in default build |
| GREEN | cvar_bitfields | 0 | 0 | compile blocked |
| GREEN | test_cvar_display_settings | 0 | 0 | compile blocked |

No tests ran. Counts are not behavioral RED/GREEN; all invocations exit 101. Default-feature runs only; strict epochs not authorized. Known pre-existing runtime failures were not reached or tested.
## Per-source-row scope

| Source row | Proven scope | Unproved scope | Recommended status |
|---|---|---|---|
| `cvars-assistedCombatReduceHighlights-166` | Retained source names addition; static existing epoch-gated default and shared storage/API routing inspected. No runtime proof. | Default/mutation/reset, getter arity/bool, coercion/case behavior, sibling/environment isolation, strict epoch publication; native defaults/flags/security/persistence and downstream effects. | audit-pending |
| `cvars-developerLogFilterDebug-167` | Retained source names addition; static existing epoch-gated default and shared storage/API routing inspected. No runtime proof. | Default/mutation/reset, getter arity/bool, coercion/case behavior, sibling/environment isolation, strict epoch publication; native defaults/flags/security/persistence and downstream effects. | audit-pending |
| `cvars-developerLogFilterError-168` | Retained source names addition; static existing epoch-gated default and shared storage/API routing inspected. No runtime proof. | Default/mutation/reset, getter arity/bool, coercion/case behavior, sibling/environment isolation, strict epoch publication; native defaults/flags/security/persistence and downstream effects. | audit-pending |
| `cvars-developerLogFilterFatal-169` | Retained source names addition; static existing epoch-gated default and shared storage/API routing inspected. No runtime proof. | Default/mutation/reset, getter arity/bool, coercion/case behavior, sibling/environment isolation, strict epoch publication; native defaults/flags/security/persistence and downstream effects. | audit-pending |
| `cvars-developerLogFilterNormal-170` | Retained source names addition; static existing epoch-gated default and shared storage/API routing inspected. No runtime proof. | Default/mutation/reset, getter arity/bool, coercion/case behavior, sibling/environment isolation, strict epoch publication; native defaults/flags/security/persistence and downstream effects. | audit-pending |
| `cvars-developerLogFilterSpam-171` | Retained source names addition; static existing epoch-gated default and shared storage/API routing inspected. No runtime proof. | Default/mutation/reset, getter arity/bool, coercion/case behavior, sibling/environment isolation, strict epoch publication; native defaults/flags/security/persistence and downstream effects. | audit-pending |
| `source-context-165` | Retained heading identifies examples, not complete inventory. | No runtime capability claim. | audit-pending |
| `source-context-172` | Excerpt expressly states “20 added, 5 removed” and incomplete list; only six additions named. | Fourteen addition names and all five removal names absent; full historical page or authenticated 67602 → 68182 diff needed. Current simulator constants are not historical evidence. | audit-pending |

## Merge risk

Not ready: default build fails on master Val import gating, so CVar behavioral verification and integration-binary compilation remain unavailable. No native/default/security conformance or full inventory coverage claimed. Changes add public-API tests/spec and inferred-policy annotations only; no new CVar behavior, fallbacks, shims, vendor/cache edits or coverage JSON edits. Potential fixture/integration failures remain undiscovered until compiler blocker is resolved. No push, merge, deployment, PR or model/agent invocation.
Proof ledger correction: exact rebased base was 7702befe8a567c225d9e8680594186e9689734f4, not abbreviated 9ce93bb12 (master advanced during initial work). Delta from 9ce93bb12 inspected; earlier base references mean its unchanged producer code plus the additional base commit. All RED/GREEN/fmt/check logs belong to this exact base + changes described above.
Step 5 commit exit 0; only four CVar paths staged.

## Committed result

Commit: `ab0b4db6c52d4c7e2cc0b77cc815e0db123681d2`.

Changed files:
- `src/cvars.rs`
- `tests/set_cvar_global.rs`
- `tests/patch_12_0_7_cvars.rs`
- `docs/specs/patch-12-0-7-cvars.md`

One commit made; worktree status checked. Existing proof logs retain scope because no behavior changed after GREEN; fmt ran before commit and only formatted the same Rust scope. Integration remains BLOCKED / audit-pending, not passing or merge-ready.
