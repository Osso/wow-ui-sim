# EllesmereUI Forever compatibility

## Package and scope

- EllesmereUI **9.2.2**, CurseForge project **1477613**, file **8936131**; latest Forever-tagged release observed with browser-cli on 2026-09-21 at 07:56 UTC.
- Existing archive reused without downloading: 19,737,796 bytes; SHA-256 `16c2eb4f0e8370a46b50c91fb535b2241dddfe3ec49526e0105195bc8b5a94f1`.
- Package contains 21 addon folders. This does not mean 21 active Forever modules: TOC game filters, LoadOnDemand options/locales and explicit suite stand-down rules must be distinguished from failures.
- Sources remain unchanged in `/tmp/ellesmere-forever/8936131/Interface/AddOns`; isolated addon-enable state and empty WTF/install roots avoid changing the user's installed addons or SavedVariables.

Goal: actual startup and major reachable interactions, not source-diff-only acceptance. Addon/vendor patches and masking shims are excluded. The user permits informed simulator guesses when explicitly recorded as guesses; those are not native-conformance evidence. User-run Forever probes are unavailable.

## Initial runtime evidence

At base `355e4e97f`, actual package startup produced 14 error records / 31 occurrences. Several are duplicate error-handler presentations, not distinct root causes. Full output and command are recorded in `/tmp/ellesmere-forever/proof-ledger.json`.

| Boundary | Evidence | Current disposition |
| --- | --- | --- |
| Action-bar paging | Configured override/vehicle indices become nil while inactive; real paging-condition concatenation fails | `3b00f9c5e` returns configured indices independently of availability for all three non-null index queries; 13 focused development tests pass |
| Specialization guards | Extra legacy `GetSpecialization` exists while `GetSpecializationInfo` is absent; actual suite guards consequently take the wrong branch | `305a9fe8f` removes the extra publication only for Forever; corrected state-seeded regressions pass 3/3 at `b3071cab8`. Native deprecated-specialization TOC excludes Camelot |
| Idle unit-frame paint | `UnitChannelDuration` called before the no-channel early return; global absent | At `e137d9df6`, duration/interrupt regressions pass 7/7, empowered-stage fractions 2/2, and GUI cast-completion regression 1/1. Actual full cast/channel acceptance remains open |
| Managed aura dirty update | `Enum.OnUpdateMode` absent under Forever | `d93621f42` shares numeric enum/method/state-machine capability; five regressions pass, including actual ManagedAuraContainer dirty updates |
| QueueStatus/Edit Mode | Failure-time instrumentation records a 45×45 button with zero anchors during simulator post-load replay; MicroMenu and Minimap callbacks read its center before its own update | `0b95bed3e` restores native `InitSystemAnchors` before replay; actual callback fixture and grouped regression pass, preserving final authored anchors |
| Chat rebuild without timestamps | Uninitialized `nowT`/`nowG` retain stale function/table values | Rilua correction pinned; compiler-bound persistent cache automatically replaces stale pack. Actual chat rebuild passes timestamps off/on/off in cold and warm processes |

## Lua conditional-local initialization

The actual chat function declares two locals after a conditional nil assignment and initializes them only when timestamping is enabled. With timestamps disabled, Lua semantics require both locals to be nil. The old compiler could merge their `LOADNIL` into the preceding conditional instruction and then skip that merged initialization at a deferred jump target.

The reduced reproduction in `/tmp/ellesmere-forever/nil-initialization.lua` returns `function, nil` under the old simulator runtime but `nil, nil` under LuaJIT. Enabling timestamps assigns the locals explicitly and avoids the symptom; changing `GetTime` or addon arithmetic would therefore patch the wrong layer.

Rilua commit `1a6d3e44d4bca99f6f5cfa38bf85b3df610b9ef9` marks the target before queuing deferred jumps. Four behavioral regressions cover conditional guard temporaries, disabled/repeated timestamp paths, elseif joins and loop exits. Development RED reproduced two failures; GREEN passes four tests and the original reproduction. The change was published alone to `Osso/rilua:main` after the user's direction to use main.

`8ddf0908d` binds pack headers and content keys to the resolved compiler revision and removes unversioned cache migration. An isolated old-runtime pack was retained in place: the new cold process recompiles misses, then the warm process reports 1674/1674 cache hits with zero replay failures. Both pass actual chat rebuild with timestamps off/on/off. Evidence: `/tmp/ellesmere-forever/cache-upgrade-ledger.json`. All 26 cache unit tests pass at `e137d9df6`. Final independent simulator verification remains open; the existing rilua verification ledger has disputed verifier attribution and is not credited as independent proof.

## QueueStatus initialization ordering

A separate temporary diagnostic addon wrapped `FrameUtil.GetScreenQuadrant` without changing its result or suppressing errors. Four failing calls saw `QueueStatusButton` at 45×45 with zero anchor points and nil center; `UIParent` already had valid 1600×1200 geometry. Traces identify MicroMenu updates, action-bar visibility callbacks and Minimap scale callbacks during the simulator's post-load Edit Mode replay. Geometry later becomes valid; checking only settled startup state misses this boundary.

Pinned `Blizzard_EditMode/Shared/EditModeManager.lua` initializes all registered system anchors before updating systems. The simulator's custom replay skipped that phase, allowing an earlier system to query the still-unanchored QueueStatus button. `0b95bed3e` restores it. The regression passes cross-system callbacks and final authored-anchor assertions. Evidence: `/tmp/ellesmere-forever/queue-runtime/mutation-boundary.stderr` and `/tmp/ellesmere-forever/queue-anchor-ledger.json`.

## Later reachable failures

The integrated `b3071cab8` startup leaves two error records: missing `Frame.HasAnyForbiddenAspects` in native `SecureHandlers.lua:592`, and unknown `UntrustedScriptExecution` during AuraKit container creation. `cc57bea8d` shares existing forbidden-aspect methods, enum data and inheritance guards through a narrow capability; animation-mask additions retain their separate version boundary. Follow-ups share `GetObjectTable` and existing secure/public object projection rather than replacing native private mixins. At `e137d9df6`, forbidden consumers pass 2/3 and partition regressions 4/6: native custom-aura creation still encounters missing `Enum.CustomAuraButtonUpdateMode`, and one initializer path rejects its forbidden-object reference. Optional-mask semantics and animation enforcement are not newly claimed.

Actual player health/power and cast interactions progress beyond startup, exposing missing `UnitNameFromGUID` during native interruption handling. `dd5b398d7` publishes the existing modeled name query; the next actual native callback demonstrates `UnitClassFromGUID` too, shared in `a31e12d50`. Both are exercised by the passing native interrupt-text regression. No complete interaction pass is credited yet.

## Integrated checkpoint: `e137d9df6`

- Combined Forever integration/library compilation succeeds. Eight warnings originate in pre-existing Retail-only library-test helpers; they were not suppressed or changed.
- Passing focused filters: cast durations/interrupts 7/7; empower percentages 2/2; cooldown widget 11/11; message frame 30/30; OnUpdate modes 5/5; cache 26/26; GUI cast completion 1/1. `channel_lifecycle::` selects zero Forever tests and is not credited.
- Partial filters: forbidden consumers 2/3; object partitions 4/6; numeric formatter 6/7. The numeric constructor works, but its duration-binding consumer returns nil. These failures block completion.
- Actual headless target fixture passes show → health 25000/50000 → hide. Real-time GUI wrapper records the chat test passing but does not progress to the queued next test within 65 seconds; options/unlock/actions are not credited. The earlier headless action test times out because 500 rapid ticks do not guarantee elapsed wall-clock timer time.
- All 666 package files match the original hashes after producer implementation. No package additions, removals or modifications were found.

Evidence: `/tmp/ellesmere-forever/producer-batch-build-retry-ledger.json`, `producer-batch-tests-ledger.json`, `major-interactions-first-ledger.json`, `gui-interactions-batch-ledger.json`, and `package-immutability.json`. Prior artifact verification failed; current source has not passed the final independent gate. After three aura-path passes, the user selected a full dependency-chain audit. The following checkpoint records its bounded fixes and remaining failures.

## Audited-chain checkpoint: `9476efcf5`

Nested timer queue preservation, duration-binding profile availability, formatter namespace restoration, AuraContainer API/enum publication, direct forbidden argument projection, `AddSecretAspect` publication, and explicit secure-delegate execution are committed. Native Lua remains unchanged.

| Capability | Latest observed proof | Remaining boundary |
| --- | --- | --- |
| Nested timers; formatter restoration | 2 timer tests; cleanup identity regression and binding-availability regression pass at `248f665fd` | Final independent verification |
| Numeric formatter / duration binding | Numeric formatter 7/7 at `248f665fd`; duration-binding copy/native consumer 7/7 after `9476efcf5` | Native formatter threshold parity is not established |
| AuraContainer native initialization | Option processors 8/8, tainted forbidden consumers 3/3, object partitions 8/8, secure delegate taint tests 2/2 | Full addon path reaches missing `AddAccessRestrictions` after login |
| Actual GUI interactions | Chat, options/navigation/unlock, cast/channel/empower, target show/health/hide pass | Action fixture compared a path with canonical file ID `135907`; corrected fixture awaits replay. Aura fixture cannot find its assigned button while native access registration fails |
| Startup | Remaining native TargetFrame callback error reproduced | `SetAuraContainerAnchorsChangedCallback` ownership/load boundary remains under diagnosis; post-cleanup missing globals do not prove mixin-composition failure |

Evidence: `/tmp/ellesmere-forever/audited-chain-tests-ledger.json`, `secure-chain-tests-ledger.json`, and `secure-chain-gui-ledger.json`. GUI completion means all six fixtures returned, **not** that all passed: the recorded result is 4/6. Existing eight profile-gating warnings remain pre-existing. No full compatibility, clean-startup, or independent final acceptance claim is made.

## Runtime data trace: `bff7719e3`

Actual GUI replay now passes **5/6** fixtures: chat, options/navigation/unlock, player cast/channel/empower, action icon/cooldown/reset, and target show/health/hide. Aura display remains blocked. Evidence: `/tmp/ellesmere-forever/traced-producers-gui-ledger.json`.

- TOC parsing retained a tab-separated game-type annotation inside two filenames. `ac9ce1897` fixes that root cause; the latest GUI no longer reports the TargetFrame callback error. A focused native-template regression needed the real startup prerequisite AuraContainer loaded first; that fixture correction is committed but its GREEN is pending.
- Public and secure aura namespaces were identical, but both enumeration methods returned nil because their modeled implementations were Retail-gated. `4dadf6a5a` shares the existing producers; five grouped tests pass. Runtime enumeration now includes injected aura `19750` (instance `6`) under `HELPFUL`, and the real `pball|-` group has ten frames.
- Action cooldown data omitted required `SpellCooldownInfo.isActive`; this was not a fixture error. `bff7719e3` restores the field for Forever/current Retail while preserving the earlier four-field result. Three grouped tests and actual five-second cooldown paint/reset pass.
- The next aura boundaries are `UnitIsPlayerControlledOrGroupMember` publication (`5903290d5`, compiled GREEN pending) and authenticated secret-number inputs rejected by duration setters. A direct untainted probe reproduces all three setters accepting plain numbers but rejecting wrapped equivalents.

After opening options, 17 of the 21 package folders are loaded. Friends, MythicTimer and RaidFrames explicitly require `standard`, which Forever does not advertise. Locales is on-demand for non-English clients. These four unloaded folders are not runtime failures. Loaded module namespaces are recorded in the GUI ledger; presence alone does not prove every module interaction.

### Bounded secret-value assumptions

The user authorized informed guesses and explicitly ruled out user-run Forever probes. [Duration core](specs/duration-core.md) records the selected non-native-verified policy: retain authenticated wrappers in timing storage, restrict timing reads to untainted callers, preserve secrecy through copying/reconfiguration/reset, and clear it with `SetToDefaults`. Plaintext timing must not be exposed through raw Lua fields or addon formatter arguments. Widget/text-binding handoffs retain secret origin and restrict direct readouts. These changes are committed or in progress; their combined compilation, behavioral GREEN, runtime acceptance and independent verification remain pending. General VM secret arithmetic and complete secrecy enforcement are not claimed.

## Coverage still required

- Initialization/active-path assertions for eligible modules, explicitly reporting intentional Forever stand-down.
- Preserve the observed settings/navigation/unlock and cast/channel/target passes through final integration; broader layout editing is not covered.
- Close aura creation/update/removal with observable real-button icon, stack, and cooldown assertions.
- Action-bar/cooldown/resource consumers and sustained updates, including errors swallowed by addon-level protected calls.
- Independent final verification after the last applicable source change; no full-addon compatibility claim from startup alone.

## Sources

- Cached package file `8936131` and previous file `8848122`; [public release page](https://www.curseforge.com/wow/addons/ellesmereui/files/8936131).
- Pinned Forever 1.60.1.69913 Blizzard API documentation and unchanged consumer source.
- [Rilua compiler correction](https://github.com/Osso/rilua/commit/1a6d3e44d4bca99f6f5cfa38bf85b3df610b9ef9).
- [Broader cached-addon comparison](forever-addon-comparison.md) — separate earlier scope.
