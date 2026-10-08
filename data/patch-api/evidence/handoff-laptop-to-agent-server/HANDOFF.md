# Handoff: patch API audit, laptop → agent-server

Written 2026-10-08 by the laptop main session (Claude). Master at handoff: `5131632c8`
(8.3.0 page merged, CI green). Read [CLAUDE.md](../../../../CLAUDE.md) and
[docs/wiki/index.md](../../../../docs/wiki/index.md) first.

## Goal and standing instructions (from the user)

- Audit the Warcraft Wiki "Patch X/API changes" pages one at a time, newest to oldest,
  with meaningful modeled behavior; don't dwell on problematic cases (record precise
  reasons and move on). 12.1.0 → 8.3.0 are done and merged. Continue with older pages.
- Delegate each page to a Sol subagent (`pi-delegate:sol`, gpt-6.1-sol). Run 1–2 agents
  at a time; this host has 32 threads / 46 GB, `agents.slice` caps 42 GB.
- Pushing master is approved (`git push origin master:master`); pushing rilua is approved.
- Never present work as done while CI fails. Fix root causes; never suppress warnings;
  never edit Blizzard/vendor/Wowless files or monkey-patch Blizzard Lua.
- Commit each coherent change immediately. Worktrees go in `~/.worktrees/<repo>-<branch>`.

## Ownership: this session owns everything now

The laptop session stopped its two in-flight agents mid-work and pushed their branches.
Nothing runs on the laptop any more. Resume both from these branches (not from scratch):

1. `origin/p825-page` (3 commits, agent stopped mid-audit) — the 8.2.5 audit.
   `769f10bfb` already wrote `data/patch-api/sources/api-change-pages-remaining.json`:
   every remaining page older than 8.3.0 (this answers "where does the series end" —
   report it to the user). Then a discovery sweep was added; gap accounting, retirements,
   ledger, validator and wiki are not done. Its sweep may still have the 8.3.0
   placeholder: 8.3.0 is on master now, so add that register.
2. `origin/prefork-migrate-1` (user-approved: move slow integration tests into the prefork
   harness). `ac46bb90a` committed the classification in
   `data/test-perf/prefork-migration-plan.json`; 7 modules migrated (garrison UI, static
   popup game, static popup, social toast, spell diminish UI, simple checkout, player
   spells, player choice). The last commit `0074cd5b2` is an UNVERIFIED WIP migration of
   the new-player-experience-guide cases — verify or redo it. Then verify the batch
   (all prefork cases pass, integration no longer lists migrated tests, no assertion
   weakened), merge, and continue with the remaining batches from the plan.

Create local worktrees from these branches
(`git worktree add ~/.worktrees/wow-ui-sim-<branch> <branch>` after `git fetch`).

## Test infrastructure on this host

- Full suite in the background: `tools/full_suite.py submit [ref]` (installed copy:
  `~/bin/full-suite`), then `full-suite status [ref]`. Runs integration (nextest, 16
  workers), prefork and lib in `~/Projects/wow/full-suite-checkout` with its own target
  dir; results in `~/Projects/wow/full-suite-results/<sha>.json`, with `new_failures`
  relative to the last master run. Integration takes ~11 min here vs ~45 min on the laptop.
- GitHub CI does NOT run the integration suite (its Blizzard-UI fetch is disabled), so
  run `full-suite` on master after any change that touches loader/runtime/shared helpers
  or retires globals, and before trusting CI.
- Known environment gaps here: no WoW install, so ~16 tests that load game textures
  through CASC fail (texture loading, hero-talent render visuals, `rendering_pipeline`
  layer1/2/5). Pending user decision on copying the 194 GB `Data/` store. Laptop
  integration baseline failures: `laptop-integration-failures-ed1822abf.txt` here
  (e.g. garrison explicit load, tooltip clamp, maw border atlas, 6 lib tests).
- Prefork harness: `cargo test --test prefork_full_ui -- <one filter>`. Publication
  sweeps are prefork cases: `cargo test --test prefork_full_ui -- publication_sweep`.

## Per-page integration procedure (main thread)

1. Retirement scan yourself: extract the page's `RETIRED_X_Y_Z_MEMBERS` from
   `src/c_api/patch_retired_members.rs` and any `global_stubs.rs` gates; for each,
   `rg -n "Ns\.Member\b"` (whole word!) and the bare name in
   `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns` excluding `*Documentation*`.
   Any current consumer → the retirement is wrong (lesson: 11.2.0 `Browser:NavigateTo`).
2. Caller scan in `src/` and `tests/`, untruncated, including `pcall(Name, ...)` and
   `and Name then` (lesson: the 10.2.0 legacy addon globals broke keybinding actions and
   ~50 tests because a truncated scan missed callers).
3. Rebase onto master. Expected conflicts: `patch_retired_members.rs` (keep both lists,
   merge the `//!` header patch list), `docs/wiki/index.md`/`log.md` (keep both),
   extractor/generator tools (keep both behaviors; new behavior must be opt-in and every
   saved extract/register must still reproduce).
4. Replace the agent's placeholder with the newer page's register in its sweep's
   `later_registers`. If the sweep reports resolved gaps, remove them from
   `tests/data/patch_X_sweep_known_gaps.json`, move those ledger rows to
   `bounded-coverage` with a "superseded by <patch> removal" note, and refresh the
   audit's evidence (results, gap review, negative control, summary).
   `tools/extend_patch_audit_receipts.py <worktree> <evidence-dir> <prefix> <note> <patch>`
   adds missing register-reproduction rows and sweep-summary rows; run the page's
   `validate.py` until it passes.
5. Checks: all publication sweeps; integration + prefork tests for every area touched by
   retirements; `python3 tools/test_extract_patch_non_inventory.py`;
   `python3 tools/test_gen_patch_wikitext_register.py`; every extract reproduces with its
   recorded `extractor_flags` (12.0.5/12.0.7/12.1.0 were already non-reproducible); every
   register regenerates byte-identically with its recorded `generator_flags`; Mists
   `cargo check --no-default-features --features sound,gui,casc,client-mists --tests`
   with zero non-vendor warnings.
   Run `python3 tools/check_patch_validators.py` before merging.
6. Fast-forward master, push with explicit refspec, then check CI by commit:
   `gh api "repos/Osso/wow-ui-sim/actions/runs?head_sha=<sha>"` (`gh run list` returned
   stale results). Remove the worktree and branch.

## Parallel pages

Two page agents can run at once: start the older page from master with a one-line
placeholder at the start of its `later_registers` for the newer, unmerged page. Integrate
in order (newer first). Tell agents to derive validator counts from files, not hard-code
them.

## Page-agent prompt template

Use the 8.2.5 prompt as the template (worktree from master, explicit cwd for every command,
own target dir, no push/merge/spawning, retirement and caller-scan rules, opt-in tooling,
validator guidance, targeted verification list, wiki update, final report fields). The
laptop session's prompts are in the session transcript; the key constraints are all
listed in the procedure above.

## Open user decisions

1. Copy the WoW install (`Data/`, 194 GB) to this host so texture tests run here?
2. Page cutoff: none set; the series is finite (see the remaining-pages file).
3. 16-shard GitHub CI job: parked; the background `full-suite` runner here replaces it
   for now.
