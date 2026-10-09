# Bounded 3.1.0 startup + integrated negative verification

2026-10-09. Followed `/home/osso/AgentConfig/skills/verify/SKILL.md` as verifier; no delegation. Read 3.1.0 audit/spec and getter implementation.

## Verdict

PASS for these two assigned gates only, at clean stable `/home/osso/.worktrees/wow-ui-sim-p310-source`, HEAD `614402d56f726ec34cedea2f28a9609cdd653462`. All final command receipts have identical start/end HEAD, empty tracked diff/status and `scope_unchanged=true`.

| Gate | Exact argv | Observed result |
|---|---|---|
| Separate build | `/usr/bin/cargo build --offline --bin wow-sim` | exit 0; six inherited iced manifest lint-key warnings; no build timeout |
| Built startup | `/usr/bin/timeout 90 target/debug/wow-sim --no-addons --no-saved-vars lua-errors` | exit 0; full stdout exactly `[]\n`; parsed empty JSON array; stderr reports `Status: CLEAN`, `Lua errors: 0 unique, 0 occurrence(s)`, unattributed 0 |
| Exact negative only | `/usr/bin/cargo test --offline --test prefork_full_ui -- patch_3_1_0_publication_sweep::patch_3_1_0_publication_sweep --exact --nocapture` | expected exit 1; `running 1 tests`, `0 passed; 1 failed; 1 total`; assertion at `tests/common/publication_sweep.rs:560:5` detects exactly one new gap, no stale/resolved gaps |

All commands use `/tmp/p434-acceptance-runner.py`, with exact cwd/revision/argv/env overrides/timestamps and complete separate stdout/stderr retained. Build/startup overrides `{}` (inherited environment). Negative overrides exactly:

- `P310_SWEEP_REGISTER=/tmp/p310-integrated-negative-register.json`
- `P310_SWEEP_OUT=/tmp/p310-integrated-negative-results.json`

Copied own 110-entry committed register to /tmp, changing only the symbol on `wt-global-api-GetNumArenaOpponents-99` from matched `GetNumArenaOpponents` to unique `P310_Nonexistent_IntegratedNegative_614402d56_20261009`. Mutation receipt: `/tmp/p310-integrated-negative-mutation.json`. Observed fabricated global `raw=nil; lookup=nil`, `ok=false`. Baseline retained fresh observations: 110 rows, 52 matches/58 gaps. Negative: same 110 row IDs, 51 matches/59 gaps. Exactly that row differs; all other observations identical. Historical implementer negative 60 and original negative 62 were not rewritten.

## Output and execution caveats

Startup stderr contains eight ALSA diagnostics (`cannot find card '0'`, `Unknown PCM default`) and `Sound: no audio device available`; audio is not proven. No startup Lua gaps observed, no patches applied. Log shows 291 Blizzard addons loaded, read-only EditMode cache source, bytecode cache 1569/1569 hits with stored=0/store_fail=0, and headless font system `casc=false`. This is this exact cached runtime invocation, not full asset/audio/native parity.

Initial async startup/negative runners left start receipts but no result/log output and no surviving processes. Preserved `/tmp/p310-*-incomplete-attempt1-*`; changed launch to detached `posix_spawn(..., setsid=True)`, not a log-recovery rerun. Incident record `/tmp/p310-async-attempts.json`.

First completed negative used substring `patch_3_1_0_publication_sweep`; module-name matching also selected the radians case (1 passed/1 failed). This accidentally re-executed that existing case despite requested narrow scope. Preserved all `/tmp/p310-integrated-negative-overselected-*` receipts; corrected selection with fully-qualified `--exact`. Final negative is one case only. No broad Cargo/check/fmt/full-suite reruns, no default/Mists reruns. Main's running full suite was neither polled nor changed.

## Proof boundaries

| Scope | Status |
|---|---|
| Built cached retail startup Lua-error gate | PASS, zero observed Lua errors |
| Integrated negative sensitivity | PASS, expected rejection 58 -> 59, exact 110 rows |
| Native 2009/PTR behavior, nil/default/coercion/security parity | UNPROVEN |
| Other profile runtime behavior and security enforcement | UNPROVEN; prior compilation proof is separate and not repeated |
| Broader acceptance, full suite, CI | Main-owned; no completion claim |

No tracked edits/commits, vendor edits, explicit cache writes, network, operations or delegation. Final worktree remains clean.

## Receipts

Aggregate full-output receipt: `/tmp/p310-startup-negative-receipts.json`.
Individual `/tmp/p310-startup-build-result.json`, `/tmp/p310-startup-result.json`, `/tmp/p310-integrated-negative-result.json` include all stdout/stderr, scope and exits; corresponding `.log` files retain combined output. Startup/negative `-stdout.txt` and `-stderr.txt` retain exact separate streams. Mutation register and negative observation JSON remain in /tmp; aggregate receipt includes their SHA256s.
