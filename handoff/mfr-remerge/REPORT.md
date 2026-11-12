# MFR re-merge gates

**Lane:** `mfr-gates3`  
**Source tested:** `248a27c194e049477708b50868c3ab6a153ed716`  
**Worktree:** `/home/osso/.worktrees/wow-ui-sim-mists-frame-residuals`  
**Date:** 2026-11-10

All profile test-builds used `cargo test --all-targets --no-run --locked --offline --message-format=json` with the profile feature set. Each accepted build receipt has `reason=build-finished`, `success=true`, and zero compiler warnings. Tests ran immediately after their profile build, from its receipt executable. `frame_positions` uses `harness = false`, so it has no Rust `--list` interface; each execution runs its 28 cases.

| Profile | Build receipt | `frame_positions` | `lib` |
|---|---|---|---|
| Retail | `mfr-gates3-retail-all-retry.json`, 0 warnings | VERIFIED: 28/28 pass; no failures | Not requested |
| PTR | `mfr-gates3-ptr.json`, 0 warnings | VERIFIED: 28/28 pass; no failures | Not requested |
| Forever | `mfr-gates3-forever.json`, 0 warnings | VERIFIED: 28/28 pass; no failures | Not requested |
| Mists | `mfr-gates3-mists.json`, 0 warnings | VERIFIED: 25/28 pass; failures: `micro_menu`, `micro_menu_container`, `casting_bar` | VERIFIED: 2027 passed, 0 failed, 1 ignored |
| Era | `mfr-gates3-era.json`, 0 warnings | VERIFIED: 26/28 pass; failures: `bags_bar`, `chat_frame` | VERIFIED: 1984 passed, 0 failed, 1 ignored |
| Anniversary | `mfr-gates3-anniversary.json`, 0 warnings | VERIFIED: 26/28 pass; failures: `bags_bar`, `chat_frame` | VERIFIED: 1984 passed, 0 failed, 1 ignored |
| Wrath | `mfr-gates3-wrath.json`, `cargo check --all-targets`, 0 warnings | Not requested | Not requested |

**Manifest bin test:** VERIFIED: `every_checked_in_patch_manifest_matches_repository_artifacts` passed (1 passed, 0 failed, 129 filtered out); receipt `mfr-gates3-manifest.json`, 0 warnings.

All receipts and Cargo JSONL/stderr files are under `/home/osso/.cache/wow-ui-sim-targets/slot-3/receipts/`. Run logs are under `/var/tmp/osso/prship/`; per-run JSON records are in `/home/osso/.local/state/wow-ui-sim/handoff/mfr-remerge/`.

Each test run used a fresh child `XDG_DATA_HOME` seeded from the current CVar file via `O_NOATIME`. VERIFIED: the original CVar hash remained `8d1709049bc9acdbed0465889701a0e370667ef18cf06b71db479b3972aa350c`, and its before/after stat values matched. The source worktree was clean at the tested SHA.

**Gate notes:** The first Retail all-target test-build receipt recorded Cargo success and 0 warnings, but slot certification rejected a stale `debug/wow-sim`; the permitted same-source retry succeeded with no rejected artifacts. The first Mists test launch used an invalid systemd option and did not start the test executable; the corrected run produced the results above. No merge defect appeared, so no simulator source was changed.
