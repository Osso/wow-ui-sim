# Handoff: 12.1.0 audit moves from desktop (OssoBuild) to laptop — 2026-10-04

The desktop session (Claude, conversation `f86e1850`) was stopped mid-round because the
16 GB WSL VM was starved. Everything it produced is now on the laptop.

## Governing goal (user)

Continue the patch API audit: one page at a time, meaningful modeled behavior. User's
latest direction: do not dwell on problematic cases; finish 12.1.0, then older patches;
run 1–2 agents at a time depending on available RAM.

## State on arrival

- `master` = `15d53a259` (134 commits ahead of `origin/master`, not pushed).
- 12.1.0 ledger (`data/patch-api/sources/12.1.0-page-coverage.json`): 151 bounded,
  505 partial, 391 pending, 64 metadata of 1,111 rows. 391 pending = 147 sweep gaps
  (`tests/data/patch_12_1_0_sweep_known_gaps.json`) + 244 extract rows (enums,
  structures, blue-post prose).
- Wiki: `docs/wiki/investigations/patch-12-1-0-page-audit.md`.
- Audit work dir copied to `~/.cache/wow-ui-sim-audit/` (was
  `/home/osso-test/.cache/wow-ui-sim-audit/`; old result files cite desktop paths).

## Interrupted work (redo; nothing was committed)

1. **Round 3 — close sweep gaps** (desktop branch `p1210-r3`, no commits). Result so far:
   `~/.cache/wow-ui-sim-audit/p1210-r3-result.md`. Order: G1 about 85 missing FrameXML
   helpers — sampled ones exist in cached load-on-demand bootstrap Lua and discovery
   already supports `BootstrapOnly`, so investigate the preload path, don't define
   helpers by hand; G2 gate removed symbols published natively (leave ones cached
   Blizzard Lua republishes); G3 about 15 widget methods/events/CVars.
2. **Missing 12.1.0 Global API functions** (read-only author, about 31). Partial
   contract ledger: `~/.cache/wow-ui-sim-audit/handoff-p1210-globals.md`, staging in
   `staging/p1210-globals/`.

After those: extract enumeration/structure proof batches and modelable prose rows, then
older patches.

## Build notes

- Laptop `~/.config/game-engine/build-host` is `desktop`: pass `--build-host local`
  (laptop: 24 CPUs, 54 GB) or builds go back to the desktop.
- `scripts/build-host.py` passes its own `-j8`; `~/.cache/wow-ui-sim-audit/p1210-r2-capped-build.py`
  rewrites it to `-j4`. Unneeded on the laptop's RAM.

## Open user decisions carried over

- rilua `sim-host-apis` push.
- `script-execution-limits` branch from the other machine.
- Keep or revert the ten promoted 12.0.5 rows.
