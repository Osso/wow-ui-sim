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
| Specialization guards | Extra legacy `GetSpecialization` exists while `GetSpecializationInfo` is absent; actual suite guards consequently take the wrong branch | Correct the extra publication, not the missing alias. Native deprecated-specialization TOC excludes Camelot |
| Idle unit-frame paint | `UnitChannelDuration` called before the no-channel early return; global absent | Implement documented nilable unit-duration queries over existing cast/channel state; no fake active cast |
| Managed aura dirty update | `Enum.OnUpdateMode` absent under Forever | Shared enum/method/state-machine capability needs source-backed exposure, not just an enum stub |
| QueueStatus/Edit Mode | Transient unresolved center during simulator post-load action-bar replay; geometry is valid later | Trace readiness and replay ordering before changing layout or returning fabricated coordinates |
| Chat rebuild without timestamps | Uninitialized `nowT`/`nowG` retain stale function/table values | Pure Lua compiler defect reproduced independently of frames/GetTime; rilua correction published and dependency pin updated; persistent cache integration still required |

## Lua conditional-local initialization

The actual chat function declares two locals after a conditional nil assignment and initializes them only when timestamping is enabled. With timestamps disabled, Lua semantics require both locals to be nil. The old compiler could merge their `LOADNIL` into the preceding conditional instruction and then skip that merged initialization at a deferred jump target.

The reduced reproduction in `/tmp/ellesmere-forever/nil-initialization.lua` returns `function, nil` under the old simulator runtime but `nil, nil` under LuaJIT. Enabling timestamps assigns the locals explicitly and avoids the symptom; changing `GetTime` or addon arithmetic would therefore patch the wrong layer.

Rilua commit `1a6d3e44d4bca99f6f5cfa38bf85b3df610b9ef9` marks the target before queuing deferred jumps. Four behavioral regressions cover conditional guard temporaries, disabled/repeated timestamp paths, elseif joins and loop exits. Development RED reproduced two failures; GREEN passes four tests and the original reproduction. The change was published alone to `Osso/rilua:main` after the user's direction to use main.

The dependency pin update does not by itself prove warm-cache recovery: old compiled chunks must not remain reusable under the new compiler. `/tmp/ellesmere-forever/rilua-nil-ledger.json` records library proof; end-to-end cached-loader and final simulator verification remain open.

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
