# Patch 5.4.8 API audit

Pinned pageid 177816, revision 1736772 (2014-05-16). This is the 2014 retail Mists line, not Mists Classic. The API page itself has no literal interface number; its linked parent Patch 5.4.8, revision 6611068, states `toc = 50400` and `Release = May 20, 2014`. The API navigation leads to retail 6.0.1.

## Source and scope

The page contains 27 changed CVar identities and one `SetUIVisibility` combat restriction, not API additions/removals. Default extraction retains all prose and yields 32 supplemental occurrences. An opt-in `--combat-restriction-bullets` generator flag preserves the 28 inventory occurrences; prior defaults remain unchanged.

The prefork sweep applies current-retail publication/absence only. Two ordered integration placeholders remain: 6.0.1 then 6.0.2, followed by the merged 6.1.0 and later retail chain. Classic 5.5.x registers are excluded.

## Retirement and caller evidence

No removals on this page and no runtime retirements. All 29 changed/writer identities have untruncated whole-word `/usr/bin/grep` scans in the session directory: retail cached consumers excluding Documentation, plus all src/tests callers including `pcall(Name, ...)` and `and Name then`. Master and queued p602/p601 register sets are recorded using `git ls-tree` and pinned revisions, without reading their live worktrees.

## Capability matrix

| Capability | Exact scope | Proof level |
|---|---|---|
| CVar combat write protection | `nameplateOverlapH/V`, `nameplateShowEnemies`, `nameplateShowEnemyGuardians/Pets/Totems`, `nameplateShowFriends`, `showArenaEnemyFrames/Pets`, `showPartyPets`, `showTargetOfTarget`, `uiScale`, `useUiScale` (13 identities) | Backed by CVar storage, player combat state and active Lua caller taint; bare/cached final gates pending |
| UI visibility combat protection | `SetUIVisibility(false)` blocked for insecure combat; `true` allowed; secure/out-of-combat transitions retained | Existing UIParent visibility model; cached development proof passed |
| Publication gap | `bloatTest`, `bloatnameplates`, `bloatthreat`, `consolidateBuffs`, `maxAlgoplates`, `repositionfrequency`, `targetOfTargetMode` | Seven precise nil value/default gaps; no later merged or queued retail inventory accounts for them |
| Historical inactive behavior | Above seven plus `alwaysShowActionBars`, `fullSizeFocusFrame`, `nameplateMotion`, `nameplateShowFriendlyGuardians/Pets/Totems`, `useCompactPartyFrames` (14 identities) | No currently readable backing state; no default invented or setting republished |
| Native historical protection policy | Exact error/notification text, historical defaults, linked forum discussion | Unproved; no native/historical parity credit |

The CVar policy lives under `src/c_api/` because it backs `C_CVar`; legacy globals call the same access check before persistence, scale and event mutation. Bitfield and case-varied writes cannot bypass it. Restriction applies to retail only, not inferred Classic 5.5.x semantics. No Blizzard Lua patch or new shim.

All 60 occurrences accounted: 28 inventory plus 32 supplemental; 36 bounded, 21 pending, three metadata. Meaningful models cover 13 currently readable CVars plus UI visibility. Seven inventory gaps and 14 inactive behavioral rows remain distinct. Seven later-superseded absent settings pass publication/absence only; this audit makes no retirements.

## Proof status

Initial RED reproduced insecure combat writes; the first fixture was incorrectly a map rather than a sequence, then corrected. Discovery found the seven gaps above; cached modeled behavior passed. Final targeted acceptance and portability gate are pending. Preliminary discovery receipts are not final acceptance, especially where later test/fixture changes supersede their scope.

## Sources

- [Pinned source/provenance](../../../data/patch-api/sources/5.4.8-api-changes.provenance.json).
- [Own session evidence](../../../data/patch-api/evidence/5.4.8-session-2026-10-08/).
- [Sweep spec](../../specs/patch-5-4-8-publication-sweep.md).

## See Also

- [[patch-6-1-0-api-audit]] — next merged retail register.
- [[patch-audit-validator-portability]] — historical clean/later-audit gate.
