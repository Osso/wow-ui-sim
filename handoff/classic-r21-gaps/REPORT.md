# Classic R21 implementation handoff

Date: 2026-11-10  
Worktree: `/home/osso/.worktrees/wow-ui-sim-classic-r21-gaps`  
Branch: `classic-r21-gaps`  
Implementation head before this report: `9a4c22f0e5f50b51d3c11ba4d1b9097db7570bec`  
Initial comparison base: `origin/master` `93125f591`; acceptance test commit: `060af381c`.

## Result

**VERIFIED:** The Mists R21 reputation-list, watched-slot request, engraving-mode, and backpack-currency work is implemented. The visible reputation rows form a prefix of one backing array; the remaining collapsed rows keep the same one-based positions for index-based APIs. Hidden-tail order is not asserted because native ordering is unstable.

**VERIFIED:** `SetWatchedFactionIndex` maps a backing-row position to the supplied signed reputation slot and records the request on `SimState`. Index 0 and indices at least `N+2` record -1. **INFERRED:** the row field name `reputationIndex` represents native `ReputationIndex`. Native behavior at `N+1` is UNKNOWN and is not asserted. The simulator does not model transport, player/connection checks, or deduplication against the player's current slot.

**VERIFIED:** Mists engraving mode starts false; each setter call stores its boolean and synchronously dispatches `ENGRAVING_MODE_CHANGED`, including repeated values. Mists backpack-currency state is shared with the list projection; the API is not capped at the three slots displayed by TokenUI.

Era and Anniversary behavior remains unchanged where native evidence is UNKNOWN. No Blizzard or vendor Lua was modified. Wrath proofs were excluded under the 2026-11-07 lane rule. No push, merge, or rebase was done.

## Verification

- **VERIFIED:** The new watcher test failed before implementation at commit `249813804` because `SetWatchedFactionIndex` was not published. After implementation, all ten targeted Mists tests passed: four reputation, two currency, and four engraving. The nine original UI tests selected the ReputationFrame, TokenUI, and bag consumers; the watcher test checked supplied row slots directly. Each exact test had a one-test `--list` preflight. The source CVar file remained unchanged under the O_NOATIME guard. Logs: `/var/tmp/osso/prship/classic-r21-final-{01..10}-run.log` and `/var/tmp/osso/prship/classic-r21-watch-red-run2.log`.
- **VERIFIED:** The final Mists all-target `--no-run` build completed with zero compiler warnings or errors. Receipt: `/home/osso/.cache/wow-ui-sim-targets/slot-3/receipts/classic-r21-watch-final-mists-alltargets.json`. Final Mists library tests passed: 2,027 passed, 1 ignored; CVar guard remained intact. Log: `/var/tmp/osso/prship/classic-r21-watch-final-mists-libtests.log`.
- **VERIFIED:** All six all-target profile builds completed with zero warnings. Their library results were Retail 2,243 passed/1 ignored; PTR 2,239/1; Mists 2,027/1; Era 1,984/1; Anniversary 1,984/1; WowForever 2,011/1. Build receipts are under `/home/osso/.cache/wow-ui-sim-targets/slot-3/receipts/` as `classic-r21-final-{retail,ptr,era,anniversary,wowforever}-alltargets.json` and `classic-r21-watch-final-mists-alltargets.json`. Library logs: `/var/tmp/osso/prship/classic-r21-retail-libtests.log`, `classic-r21-ptr-libtests.log`, `classic-r21-era-libtests-final.log`, `classic-r21-anniversary-libtests.log`, `classic-r21-wowforever-libtests.log`, and `classic-r21-watch-final-mists-libtests.log`.
- **VERIFIED against `origin/master` `93125f591`:** Mists `spell` had the same 38 failure names, and `frame_position` had the same single failure, `spellbook_layout::spellbook_frame_positions`.
- **VERIFIED:** `manifest_interface_data::tests::test_manifest_count` passed 1/1. `rustfmt --check` passed for changed Rust files, and `git diff --check` passed for the new commits.

## Key commits

- `a1d754e85` — implement sorted visible-prefix reputation rows.
- `249813804` — add the RED watched-faction publication test.
- `e70ffc9a1` — publish and record watched reputation-slot requests.
- `9a4c22f0e` — update final R21 verification documentation.

Slot 3 was released through `/var/tmp/osso/slot-claim.py`. The implementation head above was 17 commits ahead and 57 behind the then-current `origin/master`; the handoff report is committed separately on this branch.
