# Patch 5.4.8 API audit

Pinned pageid 177816, revision 1736772 (2014-05-16). This is the 2014 retail Mists line, not Mists Classic. The API page itself has no literal interface number; its linked parent Patch 5.4.8, revision 6611068, states `toc = 50400` and `Release = May 20, 2014`. The API navigation leads to retail 6.0.1.

## Source and scope

The page contains 27 changed CVar identities and one `SetUIVisibility` combat restriction, not API additions/removals. Default extraction retains all prose and yields 32 supplemental occurrences. An opt-in `--combat-restriction-bullets` generator flag preserves the 28 inventory occurrences; prior defaults remain unchanged.

The prefork sweep applies current-retail publication/absence only. Real merged 6.0.1 and 6.0.2 registers precede 6.1.0 and the later retail chain. Classic 5.5.x registers are excluded.

## Retirement and caller evidence

No removals on this page and no runtime retirements. All 29 changed/writer identities have untruncated whole-word `/usr/bin/grep` scans in the session directory: retail cached consumers excluding Documentation, plus all src/tests callers including `pcall(Name, ...)` and `and Name then`. Master and queued p602/p601 register sets are recorded using `git ls-tree` and pinned revisions, without reading their live worktrees.

## Capability matrix

| Capability | Exact scope | Proof level |
|---|---|---|
| CVar combat write protection | `nameplateOverlapH/V`, `nameplateShowEnemies`, `nameplateShowEnemyGuardians/Pets/Totems`, `nameplateShowFriends`, `showArenaEnemyFrames/Pets`, `showPartyPets`, `showTargetOfTarget`, `uiScale`, `useUiScale` (13 identities) | Backed by CVar storage, player combat state and active Lua caller taint; Bare/cached targeted behavior passes |
| UI visibility combat protection | `SetUIVisibility(false)` blocked for insecure combat; `true` allowed; secure/out-of-combat transitions retained | Existing UIParent visibility model; Bare/cached targeted behavior passes |
| Publication gap | `bloatTest`, `bloatnameplates`, `bloatthreat`, `consolidateBuffs`, `maxAlgoplates`, `repositionfrequency`, `targetOfTargetMode` | Seven precise nil value/default gaps; no later merged or queued retail inventory accounts for them |
| Historical inactive behavior | Above seven plus `alwaysShowActionBars`, `fullSizeFocusFrame`, `nameplateMotion`, `nameplateShowFriendlyGuardians/Pets/Totems`, `useCompactPartyFrames` (14 identities) | No currently readable backing state; no default invented or setting republished |
| Native historical protection policy | Exact error/notification text, historical defaults, linked forum discussion | Unproved; no native/historical parity credit |

The CVar policy lives under `src/c_api/` because it backs `C_CVar`; legacy globals call the same access check before persistence, scale and event mutation. Bitfield and case-varied writes cannot bypass it. Restriction applies to retail only, not inferred Classic 5.5.x semantics. No Blizzard Lua patch or new shim.

All 60 occurrences accounted: 28 inventory plus 32 supplemental; 36 bounded, 21 pending, three metadata. Meaningful models cover 13 currently readable CVars plus UI visibility. Seven inventory gaps and 14 inactive behavioral rows remain distinct. Seven later-superseded absent settings pass publication/absence only; this audit makes no retirements.

## Integrated proof on master a9d7c9566

Real 6.0.1/6.0.2 supersession registers leave all 28 own observations and seven gaps unchanged; no attributable replacement is needed. All 55 pages reproduce their expected gaps, and all 9,712 observations on the other 54 pages equal fresh pinned-master outputs byte-for-value. Saved-source reproduction passes for 55 registers and 52 extracts; the same inherited 12.0.5/12.0.7/12.1.0 failures remain.

Nine original/rebased audit commits and one external queued-register revision have explicit patch IDs, trees and blob mappings. All 226 historical artifacts remain preserved, including the original validator; its complete original invariants replay successfully through the pinned mapping. Source-directory comparisons use preserved original blob identities, not rebased trees that include unrelated 6.0.x runtime changes.

Untruncated caller and protected-name scans, cache input digests, and concrete combat-fixture review are retained in [integrated caller review](../../../data/patch-api/evidence/5.4.8-session-2026-10-08/integrated/caller-scan-summary.md). Retail integration selectors run 1,076 passing cases on branch versus 1,075 on master (the extra bare combat contract). Three failures are identical on both, including exact normalized panic locations, assertion values and messages: existing C_ChatInfo placement, EditMode defensive-icon enum (22 versus 21), and TargetFrame portrait pixel (unchanged `[63, 63, 80, 255]`). These remain out of scope. Format, all 85 Python fixtures, warning-clean non-vendor Mists check, and all 36 pinned prior validators pass. Remaining profile/prefork/startup and portability results are recorded below when complete.

Prefork supports only one positional filter; rejected batched invocations have no behavior coverage and are superseded by individual selector receipts. Its Cargo target requires `client-retail`, so no Mists prefork cases exist; Mists uses the integration target rather than adding a compatibility path.

## Historical proof status

Initial RED reproduced insecure combat writes; the first fixture was incorrectly a map rather than a sequence, then corrected. Discovery found the seven gaps above; cached modeled behavior passed. Targeted acceptance passes: 54/54 publication/factory cases, all 9,083 observations retained, and all 52 other pages match pinned master exactly. Cached combat plus CVar/world-map/keybinding filters pass (1/26/26/13); bare combat and existing CVar/bitfield/display/UI-visibility integration scopes pass (1/18/2/9/3). All four Python fixture scripts pass (4/36/34/8), format passes, and Mists `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` passes with zero non-vendor warnings. Addons-enabled branch and master startup outputs both equal `[]`.

All 53 registers reproduce byte-identically; 50 extracts reproduce, with the unchanged inherited 12.0.5/12.0.7/12.1.0 failures retained. Every earlier source byte and extractor mode is preserved. Negative control adds exactly one gap (seven → eight). No full integration suite was run.

Portability gate PASS at `e9b815c8d`: clean detached checkout 34/34 validators, unrelated synthetic later audit 35/35 (includes its dummy validator). Own validator passes with counts derived from retained files; own-log tampering is rejected and original bytes restored. Shared sources/register/sweep sets resolve at recorded Git revisions, not live files. No ignored/uncommitted validation inputs. Preliminary discovery receipts are not final acceptance where later test/fixture changes supersede their scope.

## Remaining limits

Seven publication gaps, 14 inactive historical behavioral rows and exact native failure/default/notification parity remain precisely accounted, not solved with shims. Three inherited extract reproduction failures remain unchanged. Real merged 6.0.1/6.0.2 registers are integrated without changing an own observation. This branch was not pushed or merged.

## Sources

- [Pinned source/provenance](../../../data/patch-api/sources/5.4.8-api-changes.provenance.json).
- [Own session evidence](../../../data/patch-api/evidence/5.4.8-session-2026-10-08/).
- [Sweep spec](../../specs/patch-5-4-8-publication-sweep.md).

## See Also

- [[patch-6-1-0-api-audit]] — next merged retail register.
- [[patch-audit-validator-portability]] — historical clean/later-audit gate.
