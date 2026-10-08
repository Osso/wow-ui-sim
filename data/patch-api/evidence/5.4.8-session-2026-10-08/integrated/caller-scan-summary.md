# Caller and combat-fixture review

Scans retain full `/usr/bin/grep -R -nE` output, including direct calls, aliases, `pcall` and registrations: 162 src/tests lines, 469 retail-cache lines and 465 Mists-cache lines. `caller-scans.json` records argv, exit, count and digest; `cached-caller-inputs.json` records 338 referenced cache-file digests. No validator reads the live caches. Protected-name scans supplement the writer scan because Settings forwards a variable name through CVar accessors.

| Protected readable CVar | Cached retail writers / consumers | src/tests writes and combat fixture |
|---|---|---|
| `nameplateOverlapH`, `nameplateOverlapV` | No named writer in cached Lua/XML | 5.4.8 register-driven combat test only |
| `nameplateShowEnemies` | New-player/boost tutorials, Mainline and Classic commentator direct writes; Settings Nameplates and NAMEPLATES/ALLNAMEPLATES bindings route through Settings/CVar accessor | `set_cvar_global` ordinary boolean/default writes outside combat; 5.4.8 combat test |
| `nameplateShowEnemyGuardians`, `nameplateShowEnemyPets`, `nameplateShowEnemyTotems` | Mainline and Classic commentator direct writes | 5.4.8 combat test only |
| `nameplateShowFriends` | No named cached writer; modern FRIENDNAMEPLATES uses distinct `nameplateShowFriendlyPlayers` | 5.4.8 combat test only |
| `showArenaEnemyFrames`, `showArenaEnemyPets` | Deprecated ArenaUI reads/listens; no named write | 5.4.8 combat test only |
| `showPartyPets` | PartyFrame/PartyMemberFrame reads/listens; no named write | 5.4.8 combat test only |
| `showTargetOfTarget` | Settings Combat checkbox through CVar accessor; TargetFrame reads/listens | 5.4.8 combat test only |
| `uiScale`, `useUiScale` | Settings Graphics accessor closures; many unrelated local `uiScale` variables are not CVar writes | `set_cvar_global`, captured `frame_position_replay`, `screen_mode` ordinary writes outside combat; 5.4.8 combat test |

`combat-fixtures.json` retains relevant test source digests and state-setting lines. `c_system_api` merely reads the console registry; it is not a writer. Captured frame-position replay explicitly asserts `not InCombatLockdown()`. No other fixture containing a named protected write enables combat. Generic CVar fixture writers use their own fixture/retired/gamepad/display names; they do not select these 13 names in combat.

`patch_5_4_8_combat_restrictions` exercises every readable protected name in both bare and cached full UI: tainted global, namespace, upper-case and bitfield writes fail without value/event/scale mutation; secure combat and tainted out-of-combat writes succeed. An unprotected `nameplateShowAll` tainted combat control remains writable. Missing historical defaults remain unmodeled, not republished.

UI visibility: retail TOGGLEUI binding hides/shows UIParent, panel manager and MovieFrame show it. Ordinary `ui_visibility_globals` fixtures hide/show and maximize the world map outside combat. The same bare/cached 5.4.8 test explicitly covers insecure combat hide denial, insecure combat show, secure combat hide, and insecure out-of-combat hide. Other cached profile binding files are retained in the scans, not claimed as active Mainline entry points.

Policy is retail-gated. Mists cvar/taint checks cover the shared storage/taint code without claiming 2014 policy for Mists Classic. Settings, edit-mode, keybinding, nameplate, action-bar, chat, combat/security, visibility and screenshot selectors are recorded separately in `regression-comparison.json`; load tests do not imply each consumer was invoked while in combat.
