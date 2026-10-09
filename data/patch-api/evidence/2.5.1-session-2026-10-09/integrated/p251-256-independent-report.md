# Independent bounded SOURCE verification: Classic/TBC 2.5.1–2.5.6

Date: 2026-10-09. **SOURCE checks PASS; required global portable gate FAIL. No native/runtime acceptance.**

## Revision and scope

Canonical `/home/osso/Projects/wow/wow-ui-sim` and command worktree `/home/osso/.worktrees/wow-ui-sim-p251-source` both observed at `fcdf1d151371e894aa19d800eeaaf17aa0a3674a`. Stable worktree clean before/after; canonical had pre-existing untracked `.code-index.db`, left untouched. Cumulative diff from `44df100f1` contains no root `src/`, `tests/`, `tools/`, `Interface/`, Cargo.toml/Cargo.lock/build.rs changes. Historical copies under evidence are inputs, not runtime edits. Read verify skill, wiki index, all six specs/audits, runner and retained proof/control artifacts.

No repo edits, commits, pushes, merges, deployments, network, delegation, Bash or cwd switching. No Cargo/check/build/fmt/default reproduction/full-suite execution. Synthetic later-audit changes occurred only inside the existing gate's ephemeral detached test fixture; it removed that fixture afterward.

## Fresh per-page proof

Each owned test_source_accounting.py executed exactly once; each owned historical validate.py exactly once; each original archive extracted into its own fresh `/tmp` root and relocated validate.py executed exactly once. All 18 commands exit 0. Source tests **50/50**. Historical validators **6/6**, copied archive validators **6/6**. Fresh copied summaries equal retained-input summaries except seal counts reflecting historical archive versus append-only outer maps.

| Page | SOURCE tests | Nonblank rows | API occurrences | UNPROVEN contracts | Current retained / archived seals |
|---|---:|---:|---:|---:|---:|
| 2.5.1 | 9/9 | 627 | 573 | 576 | 29 / 29 |
| 2.5.2 | 7/7 | 198 | 145 | 151 | 61 / 57 |
| 2.5.3 | 7/7 | 103 | 49 | 50 | 42 / 39 |
| 2.5.4 | 8/8 | 461 | 412 | 414 | 27 / 27 |
| 2.5.5 | 10/10 | 5 | 0 | 4 | 35 / 32 |
| 2.5.6 | 9/9 | 6 | 0 | 1 | 33 / 30 |

Totals: **1,400 literal rows; 1,179 API occurrences; 1,196 explicit UNPROVEN contract records**. Rows and contracts intentionally differ; occurrence counts are not unique identities. 2.5.4 has 413 UNPROVEN rows but 414 contracts because a linked-resource row holds two diff contracts. Across pages: 1,187 UNPROVEN rows and 213 metadata rows. No explicit callable signatures; linked/transcluded content remains unexpanded. All validators report zero runtime/native observations; no new model proof.

Archive commands use `/usr/bin/python3 -I -B`, PATH `/nonexistent`, empty PYTHONPATH, fresh HOME; copied roots contain no `.git`, `target`, current root tools/src/Cargo configuration. Historical configured-input copies remain inside owned evidence. Runner Git scope queries on these intentionally Git-free roots fail as expected; origin revision is separately recorded in aggregate receipts, not fabricated as an archive HEAD.

## Preserved seals and retained controls

All original archive hashes/members and all current original/receipt/portable seal maps match before and after commands. Original maps and archive bytes never rewritten. Archived seals: **214**, members including six maps: **220**. Archives, in page order: **272042, 283099, 244231, 265094, 230038, 228401 bytes**. Separate receipt maps: 2.5.1 seven seals; 2.5.4 six seals. Current map entries including separate receipt maps total 240.

Retained serialized controls: ledger/log rejection and exact hash restoration for every page, plus source-proof rejection/restoration for 2.5.4: **13 retained rejection/restoration controls**, all reject with exit 1 at named artifact seals. Retained copied replay exit 0 for each. Exact tested inputs remain sealed, so these controls were inspected, not rerun; no replay_controls.py capture or full tamper matrix rerun. Seals establish byte integrity against committed anchors, not external authenticity/native correctness.

## Integrated comparisons

Independently rederived section/symbol row-pair comparisons from integrated literal inventories, checking receipt input hashes:

- 2.5.1 vs 2.5.2/2.5.3/2.5.4: **3/2/3 pairs** (not unique identities; graphicsTextureFiltering occurs twice in the first comparison).
- 2.5.2 vs 2.5.3: removals of AcknowledgeAADCAlert, SHOW_AADC_ALERT, seenAADCAlert; vs 2.5.4: RAIDVolumeFog removal.
- 2.5.3 vs 2.5.4: **zero overlap** across 49/412 occurrences.
- 2.5.5 and 2.5.6: **zero explicit API identities**, not empty-inventory parity.

These comparisons remain separate from original historical seals/receipts. Source TOCs **20501–20506** are not configured Anniversary **11507**. No Retail/Wrath/Era/Forever supersession, runtime retirement, state/security/native closure or blanket unsupported-API diagnosis.

## Required global gate — FAIL, executed once

Exact command: `python3 -B tools/check_patch_validators.py fcdf1d151`, cwd stable worktree, through `/tmp/p434-acceptance-runner.py`, PYTHONDONTWRITEBYTECODE=1. Runner exit **1**; full stdout/stderr retained. No command-path constraint blocked normal fixture operation.

| Phase | Revision | Passed | Failed |
|---|---|---:|---:|
| clean | fcdf1d151371e894aa19d800eeaaf17aa0a3674a | 87/90 | 3 |
| synthetic later audit | 596fcccc7eec4f6d0447932dab598a55253e3a82 | 88/91 | 3 |

All six owned page validators pass both phases. The three failures in both phases are retained template copies, not the six owned validators:

1. `data/patch-api/evidence/2.5.2-session-2026-10-09/template-inputs/2.5.5/validate.py:22` — FileNotFoundError reading adjacent `seals.json`.
2. `data/patch-api/evidence/2.5.2-session-2026-10-09/template-inputs/2.5.6/validate.py:24` — same missing adjacent map.
3. `data/patch-api/evidence/2.5.2-session-2026-10-09/template-inputs/3.4.3/validate.py:19` — same missing adjacent map.

Observed cause: `tools/check_patch_validators.py:27` recursively discovers every `validate.py` under evidence, including these incomplete read-only template snapshots, and executes them as standalone proof roots. No fix or alternative infrastructure invented; task prohibits shared-tool/input edits. Global acceptance remains failed. No rerun performed.

Both gate phases report **zero artifacts over 5,000,000 bytes**. Gate size scope is tracked `data/patch-api/evidence`, not all repository assets. Read-only postcheck: **13,304 tracked evidence artifacts**, maximum **1,703,614 bytes**, `data/patch-api/evidence/7.3.0-session-2026-10-08/p730-integrated-sound-callers.json`; none oversized.

## Raw receipts and proof boundaries

- Aggregate full per-page/global receipts: `/tmp/p251-256-independent-raw-receipts.json`.
- Per page: `/tmp/p251-independent-raw-receipts.json` through `/tmp/p256-independent-raw-receipts.json`.
- Exact runner command receipts/logs: `/tmp/p25N-{tests,historical,archive}-{start,result}.json` and `/tmp/p25N-{tests,historical,archive}.log`, N=1..6.
- Global: `/tmp/p251-256-global-start.json`, `/tmp/p251-256-global-result.json`, `/tmp/p251-256-global.log`.
- Original control/archive/seal inspection: `/tmp/p251-256-independent-inspection.json`; poststate `/tmp/p251-256-independent-final-state.json`.
- Inherited environment supplement: `/tmp/p251-256-independent-environment.json` (mode 0600); exact per-command overrides and cwd recorded in runner receipts. Runner pin: `/tmp/p251-256-independent-runner-pin.json`. Scope receipts preserve full HEAD for Git-backed runs; copied-root origin is explicit above.

Historical/default parent evidence was not rerun or promoted: 81 registers/78 extracts at 98f2, 82 default outcomes unchanged at efbe, retained full suite FAIL 23+1+6 and 74 publication passes remain separate parent-scoped evidence. No latest-HEAD native acceptance, simulator compatibility completion or passing global-gate claim. **Handoff: bounded SOURCE proof passes; required global gate failure is the outstanding result.**
