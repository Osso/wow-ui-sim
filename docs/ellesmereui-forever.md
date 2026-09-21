# EllesmereUI Forever compatibility

## Package and scope

- EllesmereUI **9.2.2**, CurseForge project **1477613**, file **8936131**; latest Forever-tagged release observed with browser-cli on 2026-09-21 at 07:56 UTC.
- Existing archive reused without downloading: 19,737,796 bytes; SHA-256 `16c2eb4f0e8370a46b50c91fb535b2241dddfe3ec49526e0105195bc8b5a94f1`.
- Package contains 21 addon folders. This does not mean 21 active Forever modules: TOC game filters, LoadOnDemand options/locales and explicit suite stand-down rules must be distinguished from failures.
- Sources remain unchanged in `/tmp/ellesmere-forever/8936131/Interface/AddOns`; isolated addon-enable state and empty WTF/install roots avoid changing the user's installed addons or SavedVariables.

Goal: actual startup and major reachable interactions, not source-diff-only acceptance. No addon/vendor patch, guessed API values or compatibility shim is credited.

## Initial runtime evidence

At base `355e4e97f`, actual package startup produced 14 error records / 31 occurrences. Several are duplicate error-handler presentations, not distinct root causes. Full output and command are recorded in `/tmp/ellesmere-forever/proof-ledger.json`.

| Boundary | Evidence | Current disposition |
| --- | --- | --- |
| Action-bar paging | Configured override/vehicle indices become nil while inactive; real paging-condition concatenation fails | `3b00f9c5e` returns configured indices independently of availability for all three non-null index queries; 13 focused development tests pass |
| Specialization guards | Extra legacy `GetSpecialization` exists while `GetSpecializationInfo` is absent; actual suite guards consequently take the wrong branch | `305a9fe8f` removes the extra publication only for Forever; corrected state-seeded regressions pass 3/3 at `b3071cab8`. Native deprecated-specialization TOC excludes Camelot |
| Idle unit-frame paint | `UnitChannelDuration` called before the no-channel early return; global absent | `180d08b69` shares modeled duration/channel producers; corrected cast-start/completion fixtures (`5823d71a9`) and interrupt-name publication (`dd5b398d7`) await integrated GREEN |
| Managed aura dirty update | `Enum.OnUpdateMode` absent under Forever | `d93621f42` shares numeric enum/method/state-machine capability; five regressions pass, including actual ManagedAuraContainer dirty updates |
| QueueStatus/Edit Mode | Failure-time instrumentation records a 45×45 button with zero anchors during simulator post-load replay; MicroMenu and Minimap callbacks read its center before its own update | `0b95bed3e` restores native `InitSystemAnchors` before replay; actual callback fixture and grouped regression pass, preserving final authored anchors |
| Chat rebuild without timestamps | Uninitialized `nowT`/`nowG` retain stale function/table values | Rilua correction pinned; compiler-bound persistent cache automatically replaces stale pack. Actual chat rebuild passes timestamps off/on/off in cold and warm processes |

## Lua conditional-local initialization

The actual chat function declares two locals after a conditional nil assignment and initializes them only when timestamping is enabled. With timestamps disabled, Lua semantics require both locals to be nil. The old compiler could merge their `LOADNIL` into the preceding conditional instruction and then skip that merged initialization at a deferred jump target.

The reduced reproduction in `/tmp/ellesmere-forever/nil-initialization.lua` returns `function, nil` under the old simulator runtime but `nil, nil` under LuaJIT. Enabling timestamps assigns the locals explicitly and avoids the symptom; changing `GetTime` or addon arithmetic would therefore patch the wrong layer.

Rilua commit `1a6d3e44d4bca99f6f5cfa38bf85b3df610b9ef9` marks the target before queuing deferred jumps. Four behavioral regressions cover conditional guard temporaries, disabled/repeated timestamp paths, elseif joins and loop exits. Development RED reproduced two failures; GREEN passes four tests and the original reproduction. The change was published alone to `Osso/rilua:main` after the user's direction to use main.

`8ddf0908d` binds pack headers and content keys to the resolved compiler revision and removes unversioned cache migration. An isolated old-runtime pack was retained in place: the new cold process recompiles misses, then the warm process reports 1674/1674 cache hits with zero replay failures. Both pass actual chat rebuild with timestamps off/on/off. Evidence: `/tmp/ellesmere-forever/cache-upgrade-ledger.json`. Cache unit GREEN and final independent simulator verification remain open; the existing rilua verification ledger has disputed verifier attribution and is not credited as independent proof.

## QueueStatus initialization ordering

A separate temporary diagnostic addon wrapped `FrameUtil.GetScreenQuadrant` without changing its result or suppressing errors. Four failing calls saw `QueueStatusButton` at 45×45 with zero anchor points and nil center; `UIParent` already had valid 1600×1200 geometry. Traces identify MicroMenu updates, action-bar visibility callbacks and Minimap scale callbacks during the simulator's post-load Edit Mode replay. Geometry later becomes valid; checking only settled startup state misses this boundary.

Pinned `Blizzard_EditMode/Shared/EditModeManager.lua` initializes all registered system anchors before updating systems. The simulator's custom replay skipped that phase, allowing an earlier system to query the still-unanchored QueueStatus button. `0b95bed3e` restores it. The regression passes cross-system callbacks and final authored-anchor assertions. Evidence: `/tmp/ellesmere-forever/queue-runtime/mutation-boundary.stderr` and `/tmp/ellesmere-forever/queue-anchor-ledger.json`.

## Later reachable failures

The integrated `b3071cab8` startup leaves two error records: missing `Frame.HasAnyForbiddenAspects` in native `SecureHandlers.lua:592`, and unknown `UntrustedScriptExecution` during AuraKit container creation. `cc57bea8d` shares existing forbidden-aspect methods, enum data and inheritance guards through a narrow capability; animation-mask additions retain their separate version boundary. Compiled GREEN remains pending. Optional-mask semantics and animation enforcement are not newly claimed.

Actual player health/power and cast interactions progress beyond startup, exposing missing `UnitNameFromGUID` during native interruption handling. `dd5b398d7` publishes the existing modeled query through the cast-duration capability without exposing adjacent `UnitClassFromGUID`. No complete interaction pass is credited yet.

## Coverage still required

- Initialization/active-path assertions for eligible modules, explicitly reporting intentional Forever stand-down.
- Settings open/navigation/close through `/eui` and actual module pages; unlock/layout lifecycle.
- Unit/target changes, aura updates and cast/channel transitions with observable output assertions.
- Action-bar/cooldown/resource consumers and sustained updates, including errors swallowed by addon-level protected calls.
- Independent final verification after the last applicable source change; no full-addon compatibility claim from startup alone.

## Sources

- Cached package file `8936131` and previous file `8848122`; [public release page](https://www.curseforge.com/wow/addons/ellesmereui/files/8936131).
- Pinned Forever 1.60.1.69913 Blizzard API documentation and unchanged consumer source.
- [Rilua compiler correction](https://github.com/Osso/rilua/commit/1a6d3e44d4bca99f6f5cfa38bf85b3df610b9ef9).
- [Broader cached-addon comparison](forever-addon-comparison.md) — separate earlier scope.
