# EllesmereUI Forever compatibility

Cached EllesmereUI 9.2.2 establishes bounded simulator defects from a real Forever startup, but not addon compatibility: several suite modules intentionally stand down on Camelot, and full startup revalidation remains open.

## Runtime boundary

The unchanged cached CurseForge file `8936131` contains 21 addon folders. Forever TOC filters, load-on-demand options/locales, and explicit suite stand-down rules mean that folder count is not an active-module count.

At the initial reproduction, real startup recorded 14 error records and 31 occurrences. The result is failure evidence only; duplicate error-handler presentations do not establish 14 independent causes.

## Confirmed producers

`3b00f9c5e` returns configured non-null override, vehicle, and temporary-shapeshift action-bar indices even when their bars are inactive. `a59688a0d` supplies the focused regression coverage; 13 target tests pass. This fixes the actual action-bar paging concatenation boundary, not every action-bar lifecycle.

The specialization diagnosis is an incorrectly exposed `GetSpecialization`, not a missing `GetSpecializationInfo`. Ellesmere takes its legacy branch because the simulator exposes the first global while the second is absent. The native `Blizzard_DeprecatedSpecialization` TOC excludes Camelot, so adding the excluded legacy alias would model the wrong runtime. The remaining correction is to remove or profile-gate the extra global.

A pure Lua reduction of Ellesmere chat's disabled-timestamp path exposed a rilua compiler error: `LOADNIL` coalescing crossed a deferred conditional-jump target, leaving locals stale. Rilua commit `1a6d3e44d4bca99f6f5cfa38bf85b3df610b9ef9` adds the pending-jump barrier and was published to `Osso/rilua:main` at user direction. wow-ui-sim pin `88be5d1fa` updates the dependency. `8ddf0908d` now binds bytecode headers and keys to the exact locked Rilua revision and ignores legacy artifacts, so an old compiler pack cannot replay automatically. This remains implementation evidence: the parent-owned real Ellesmere cold/stale/warm replay has not run.

`cc57bea8d` makes the existing base forbidden-aspect capability available to Forever with its thirteen native masks and FrameRef query/mutation methods. The animation masks (`QueryAnimationProgress`, `AddAnimations`) are a distinct PTR 12.1.5+/Forever extension; Retail 12.1 retains its eleven-bit surface. This does not enable unrelated access restrictions, `GetObjectTable`, or `ClearScripts`. Existing `SetParent`/`SetPoint` inheritance enforcement is shared. RED evidence reaches the unchanged `SecureHandlers.lua:592` static query and tainted AuraKit's `UntrustedScriptExecution` enum lookup; compiled GREEN and full replay remain pending. `HasAnyForbiddenAspects` still ignores its optional mask argument, which the demonstrated SecureHandlers call does not supply.

## Open boundaries

- Implement source-backed nilable cast/channel duration producers over existing state; no fabricated active cast.
- `d93621f42` implements the narrow `on-update-modes` capability for Retail 12.1+ and Forever: numeric values 0–4, XML-name conversion, pre-callback one-shot reset, and removal of PTR's string/alias producer. All five focused tests are RED, including the real ManagedAuraContainer dirty path; GREEN and runtime replay remain pending.
- `0b95bed3e` restores the native initial-anchor phase before simulator replay. Failure-time instrumentation from `/tmp/ellesmere-forever/queue-runtime/mutation-boundary.stderr` saw the 45×45 QueueStatusButton with zero anchors and nil center when earlier MicroMenu, action-bar and Minimap callbacks invoked Camelot `QueueStatusButtonMixin:UpdateDefaultAnchor`; final startup geometry was therefore not relevant evidence. Pinned `Blizzard_EditMode/Shared/EditModeManager.lua:995-1013` orders `InitSystemAnchors()` before `UpdateSystems()`, while `:1454-1466` initializes a `TOPLEFT` anchor for registered non-managed-default systems. The initial RED executes the exact regression Lua blocks in `/tmp/ellesmere-forever/queue-regression-red.stderr`; focused GREEN and real startup replay remain pending. No coordinate fabrication or vendor/addon edit is part of the fix.
- Re-run isolated startup and reachable Ellesmere interactions after the remaining fixes. No full-startup or full-addon compatibility claim is current.

## Sources

- [EllesmereUI Forever compatibility](../../ellesmereui-forever.md) — cached package identity, startup evidence, and scope
- [Forever report](../../wowforever-1.60.1.md) — profile-wide runtime evidence and limits
- [Forever addon comparison](../../forever-addon-comparison.md) — separate cached-addon audit boundary
- `Blizzard_DeprecatedSpecialization/Blizzard_DeprecatedSpecialization.toc` in the pinned Forever cache — Camelot exclusion
- `/tmp/ellesmere-forever/nil-initialization.lua` and `/tmp/ellesmere-forever/rilua-nil-ledger.json` — reduced compiler reproduction and focused proof
- [compiler bytecode cache spec](../../specs/compiler-bytecode-cache.md) — locked-compiler cache contract and pending replay acceptance
- [Edit Mode initial-anchor spec](../../specs/edit-mode-initial-anchors.md) — native initialization ordering and pending regression proof
- [OnUpdate-mode spec](../../specs/on-update-modes.md) — numeric contract, XML mapping, and pending focused proof
- [Forever forbidden-aspect consumers](../../specs/forever-forbidden-aspects.md) — base masks, animation extension, exclusions, and pending GREEN

## See Also

- [[forever-clean-startup]] — distinct Blizzard-only sustained runtime proof
- [[forever-addon-comparison]] — broader cached-addon comparison, not Ellesmere acceptance
- [[client-profiles]] — Forever/Camelot profile routing
- [[bytecode-cache-growth]] — persisted-pack identity and storage bounds
- [[on-update-dirty]] — existing update-dispatch behavior and dirty scheduling
- [Forbidden-aspect inheritance](../../specs/forbidden-aspect-inheritance.md) — shared propagation and relationship constraints
