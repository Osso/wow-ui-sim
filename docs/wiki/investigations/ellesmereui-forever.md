# EllesmereUI Forever compatibility

Cached EllesmereUI 9.2.2 establishes bounded simulator defects from a real Forever startup, but not addon compatibility: several suite modules intentionally stand down on Camelot, the latest real GUI acceptance is 5/6, and full startup revalidation remains open.

## Runtime boundary

The unchanged cached CurseForge file `8936131` contains 21 addon folders. Forever TOC filters, load-on-demand options/locales, and explicit suite stand-down rules mean that folder count is not an active-module count.

At the initial reproduction, real startup recorded 14 error records and 31 occurrences. The result is failure evidence only; duplicate error-handler presentations do not establish 14 independent causes.

## Confirmed producers

`3b00f9c5e` returns configured non-null override, vehicle, and temporary-shapeshift action-bar indices even when their bars are inactive. `a59688a0d` supplies the focused regression coverage; 13 target tests pass. This fixes the actual action-bar paging concatenation boundary, not every action-bar lifecycle.

The specialization diagnosis is an incorrectly exposed `GetSpecialization`, not a missing `GetSpecializationInfo`. Ellesmere takes its legacy branch because the simulator exposes the first global while the second is absent. The native `Blizzard_DeprecatedSpecialization` TOC excludes Camelot, so adding the excluded legacy alias would model the wrong runtime. The remaining correction is to remove or profile-gate the extra global.

A pure Lua reduction of Ellesmere chat's disabled-timestamp path exposed a rilua compiler error: `LOADNIL` coalescing crossed a deferred conditional-jump target, leaving locals stale. Rilua commit `1a6d3e44d4bca99f6f5cfa38bf85b3df610b9ef9` adds the pending-jump barrier and was published to `Osso/rilua:main` at user direction. wow-ui-sim pin `88be5d1fa` updates the dependency. `8ddf0908d` now binds bytecode headers and keys to the exact locked Rilua revision and ignores legacy artifacts, so an old compiler pack cannot replay automatically. This remains implementation evidence: the parent-owned real Ellesmere cold/stale/warm replay has not run.

`cc57bea8d` makes the existing base forbidden-aspect capability available to Forever with its thirteen native masks and FrameRef query/mutation methods. The animation masks (`QueryAnimationProgress`, `AddAnimations`) are a distinct PTR 12.1.5+/Forever extension; Retail 12.1 retains its eleven-bit surface. Existing `SetParent`/`SetPoint` inheritance enforcement is shared. `HasAnyForbiddenAspects` still ignores its optional mask argument, which the demonstrated SecureHandlers call does not supply.

## AuraContainer dependency audit

A read-only audit of pinned Forever `Blizzard_AuraContainer` establishes a dependency chain, not a completed compatibility result. The TOC uses a secure environment while its XML registers intrinsic and virtual templates globally. `CustomAuraContainer` creates a public AuraButton object-table view for an initializer, then invokes the private `UpdateAuraDisplay` mixin. `UpdateAuraDisplay` is authored Lua private-mixin behavior, overridden by `CustomAuraButtonPrivateMixin`; a generic Rust AuraButton stub would be the wrong fix.

Four independently demonstrated simulator boundaries remain:

- `WowLuaEnv::process_timers()` swapped `rilua_timers` into a local queue, fired due callbacks, then assigned the old requeue back to state. A callback's `C_Timer.After()` appended to the now-empty state queue and was overwritten. `1604954a2` preserves callback-created entries for the next processing pass without changing same-pass dispatch; the new nested fixture covers `After`, `NewTimer`, `NewTicker`, pending entries, repeats, and cancellation. Compiled GREEN remains pending.
- `C_StringUtil.CreateSecondsFormatter` is initially installed by the temporary proxy factory. `Blizzard_EnvironmentCleanup` restoration reruns `register_utility_bootstrap_tables`; `0a6330816` changes `C_StringUtil` registration to retain the existing namespace instead of replacing it, preserving the public factory and public/secure namespace identity. The regression covers initial and post-cleanup formatters, but compiled GREEN and external lifecycle replay remain pending. This is not a new factory or fallback shim.
- `C_DurationUtil.CreateDurationTextBinding` was gated by `select(4, GetBuildInfo()) >= 120007` and its patch-12.1 branch. `9c56b683d` moves availability to Rust profile selection: Forever and Retail-family 12.0.7+ receive the existing binding factory, while color-curve methods remain Forever/12.1+. `c3d11eb23` separately shares the existing nine base `C_AuraContainerUtil` processors, `C_Secrets.GetSpellAuraSecrecy`, and documented `CustomAuraButtonUpdateMode`, stealable-filter, and texture-style enum publication through the narrow `aura-containers` capability. It deliberately leaves native-present `ProcessCustomAuraButtonCasterNameOptions` and application-bar `minApplications` at later gates because no demonstrated consumer requires them. The former nil-binding failure does not establish a numeric formatter defect; compiled GREEN remains pending.
- Retail AuraContainer validation accepts an ordinary public child. Forever first requires `object == GetForbiddenObjectTable(object)` and the same owner relation. `b9dcb571a` projects only direct native-frame arguments at the explicit forbidden XML secure-delegate boundary, including Cooldown, Texture, and FontString children; nils, ordinary tables, and nested table contents retain their original behavior. The public `GetObjectTable` initializer contract remains unchanged. The real addon follows this ordinary-child inbound path, so this is not a reason to relax Forever validation or add a Rust `UpdateAuraDisplay` method.
- Native `CustomAuraButton` calls `AddSecretAspect` on duration cooldowns and application bars. The method was already modeled but registered only for Retail 12.1; `d0d6a346d` shares it through `forbidden-aspects`, preserving older-profile absence. The compiled RED is the native `attempt to call method 'AddSecretAspect' (a nil value)` during custom-button initialization. Targeted Forever GREEN remains pending.
- `secureDelegates="true"` had projected arguments but invoked its secure native delegate through an ordinary Lua call. A tainted addon `AddAuraGroup` therefore reached native `settablesecurity` with caller taint and correctly failed. `9476efcf5` uses `securecallfunction` only for that explicit forbidden delegate boundary: it suspends caller taint for native setup, restores it after return/error, and does not sanitize an addon callback's closure taint. Its focused and native-consumer GREEN remain pending.
- `AuraContainerUtil.ApplyAccessRestrictions` calls the already-modeled frame method before login (deferred to `PLAYER_ENTERING_WORLD`) and immediately after login. `9d1174236` publishes only the existing access-restriction mask registration through `forbidden-aspects`, preserving Retail-only `ClearScripts` and leaving conditional aura-secrecy enforcement unmodeled. Its native pre-login/world-entry/post-login regression is uncompiled.
- `TargetFrame.lua:70` still reaches a missing public `SetAuraContainerAnchorsChangedCallback` during actual startup. `2bf64b02c` adds a pre-cleanup native `Blizzard_UnitFrame` diagnostic that loads the closure manually, preserves warnings and mixin snapshots, then asserts public setter/getter and forbidden callback storage. It is test-only: no mixin-composition root cause or production fix is claimed before its GREEN result.

The combined Forever `--lib --test integration --no-run` build succeeded at `248f665fd`; it is compile proof, not behavior proof for the later `d0d6a346d`, `9476efcf5`, `9d1174236`, or `2bf64b02c` slices. GUI acceptance at `248f665fd` completed four of six interactions: chat, options/unlock, casts, and target passed. Action and aura acceptance remain unproven; the aura fixture's broad empty-helpful-list assertion was corrected to reject only its own spell ID. The audit does not claim secret/access enforcement, private-aura callback parity, complete custom-button rendering, or full Ellesmere compatibility.

`a31e12d50` shares the existing local-player `UnitClassFromGUID` model with the same Retail 12.1+/Forever cast-duration capability as `UnitNameFromGUID`. The actual native interrupt-label consumer reaches `CastingBarFrame.lua:622`; unknown GUIDs return no values. External RED records both the missing query and native consumer. This is not general GUID identity modeling.

### Runtime enumeration and action cooldown follow-up

At `9476efcf5`, scoped native AuraContainer evidence is GREEN: the initializer/partition group passes 8/8, tainted forbidden consumers pass 3/3, and secure XML delegate tests pass 2/2. This does **not** establish full GUI aura display.

The subsequent real GUI trace injected helpful aura instance `7` (`spellId=19750`, icon `135907`, stacks `3`) and found both public and secure `C_UnitAuras.GetUnitAuraInstanceIDs` plus `C_UnitAurasPrivate.GetAllPrivateAuraInstanceIDs` returned nil. Public/secure namespace identity matched, so this was a registration-gate defect rather than a filter, candidate, partition, or fixture-input defect. `4dadf6a5a` shares the existing public/private enumeration through `aura-instance-enumeration` / `aura-containers`; compiled GREEN remains pending.

The same trace found `C_ActionBar.GetActionCooldown(1)` returned start, duration `5`, enabled, and rate but omitted `isActive` while the matching spell query reported true after only 0.665 seconds. Forever declares `SpellCooldownInfo`, which requires `isActive`; the fixture was not wrong. `bff7719e3` publishes the modeled active field on current Retail-family/Forever shapes while retaining the historical four-field payload. Compiled GREEN remains pending.

`ac9ce1897` separately fixes trailing TOC annotation parsing: tab-separated `[AllowLoadGameType mainline] [LoadIntoEnvironment secure]` had been retained in TargetFrame aura Lua paths, causing IO failures and the missing callback symptom. Its parser/secure-environment regressions are committed; compiled confirmation remains pending.

### Secret duration follow-up

After enumeration and action-cooldown publication, the real GUI reached five passing interactions: chat, options/unlock, casts/channels, action icon/cooldown, and target health. The sixth, player-aura display, now reaches native `AuraButton` duration setup. The trace shows helpful aura instance `6` for spell `19750`, public `HELPFUL` IDs including it, an empty private source, and a declared `pball|-` group with ten frames. `5903290d5` separately shares the existing `UnitIsPlayerControlledOrGroupMember` classifier through `aura-containers`; it is uncompiled.

`e6b928a23` addresses the next demonstrated boundary: native `AuraButton` passes `secretwrap(expirationTime, duration, timeMod)` to duration setters, while all three previously rejected wrapped numerics as userdata from an untainted caller. Timing slots now retain authenticated wrappers rather than public numeric values, and setters use rilua's checked unwrap path. `HasSecretValues` derives public metadata from those slots. The following lifecycle/output policy is an **informed simulator guess**, explicitly authorized because Forever-client probes are unavailable: secret timing persists through ordinary reconfiguration and `Reset`; `Copy` preserves wrappers; `Assign` retains existing target secrecy; `SetToDefaults` clears it; and secret timing queries reject tainted callers but return ordinary computed values to untainted native callers. These are not native-conformance claims. Core-only tests and compilation remain pending.

`a9fa01e18` and `3ba3bd429` carry that guessed boundary through duration text binding. Secret duration formatting requires an untainted caller; the native numeric-rule formatter receives an authenticated wrapped number rather than a decoded number supplied to addon callbacks; conversion uses captured bootstrap functions so addon overrides cannot observe a decoded value. Binding output is passed to `SetText` as a wrapped string. `0f33ec35a` adds private Rust-only secret-origin flags for shown, text, and timing state. It accepts wrapped `SetShown` and `SetText` values through checked unwrap, preserves ordinary `SetShown` truthiness, and guards direct tainted reads of the flagged values, including ancestor visibility and relevant cooldown/status-bar reads. This is a **bounded simulator guess**, not general secrecy enforcement, script-object aspect enforcement, or native conformance. The companion tests are uncompiled; no GUI aura GREEN follows from these commits.

`c5f5da7fb` shares the existing `C_StringUtil.CreateNumericRuleFormatter` and rounding enum with Forever through a narrow capability. Ellesmere AuraKit first chooses this documented formatter and only uses its seconds formatter as fallback. Its exact breakpoint table is covered, including the modeled `59.9 → "60"` pre-rounding threshold behavior; native parity at that boundary is unmeasured. Existing-binary RED reports the missing constructor. Compiled GREEN and real AuraKit replay remain pending; no formatter model or fallback policy changed.

## Open boundaries

- Implement source-backed nilable cast/channel duration producers over existing state; no fabricated active cast.
- `d93621f42` implements the narrow `on-update-modes` capability for Retail 12.1+ and Forever: numeric values 0–4, XML-name conversion, pre-callback one-shot reset, and removal of PTR's string/alias producer. All five focused tests are RED, including the real ManagedAuraContainer dirty path; GREEN and runtime replay remain pending.
- `0b95bed3e` restores the native initial-anchor phase before simulator replay. Failure-time instrumentation from `/tmp/ellesmere-forever/queue-runtime/mutation-boundary.stderr` saw the 45×45 QueueStatusButton with zero anchors and nil center when earlier MicroMenu, action-bar and Minimap callbacks invoked Camelot `QueueStatusButtonMixin:UpdateDefaultAnchor`; final startup geometry was therefore not relevant evidence. Pinned `Blizzard_EditMode/Shared/EditModeManager.lua:995-1013` orders `InitSystemAnchors()` before `UpdateSystems()`, while `:1454-1466` initializes a `TOPLEFT` anchor for registered non-managed-default systems. The initial RED executes the exact regression Lua blocks in `/tmp/ellesmere-forever/queue-regression-red.stderr`; focused GREEN and real startup replay remain pending. No coordinate fabrication or vendor/addon edit is part of the fix.
- Run compiled GREEN for `1604954a2` nested timer preservation, then rerun deferred options/action/aura acceptance paths.
- Compile and run focused GREEN for the uncompiled TOC/profile, TargetFrame diagnostic, unit-relationship, secret-duration-core, widget, and duration-text-binding slices. Secret-origin flags guard only the direct modeled reads named above; they do not establish general secrecy enforcement.
- Re-run isolated startup and all six reachable Ellesmere interactions after the secret duration/display handoffs are compiled. No full-startup or full-addon compatibility claim is current.

## Sources

- [EllesmereUI Forever compatibility](../../ellesmereui-forever.md) — cached package identity, startup evidence, and scope
- [Forever report](../../wowforever-1.60.1.md) — profile-wide runtime evidence and limits
- [Forever addon comparison](../../forever-addon-comparison.md) — separate cached-addon audit boundary
- `Blizzard_DeprecatedSpecialization/Blizzard_DeprecatedSpecialization.toc` in the pinned Forever cache — Camelot exclusion
- `/tmp/ellesmere-forever/nil-initialization.lua` and `/tmp/ellesmere-forever/rilua-nil-ledger.json` — reduced compiler reproduction and focused proof
- [compiler bytecode cache spec](../../specs/compiler-bytecode-cache.md) — locked-compiler cache contract and pending replay acceptance
- [Edit Mode initial-anchor spec](../../specs/edit-mode-initial-anchors.md) — native initialization ordering and pending regression proof
- [OnUpdate-mode spec](../../specs/on-update-modes.md) — numeric contract, XML mapping, and pending focused proof
- [Duration text binding spec](../../specs/duration-text-binding.md) — bounded secret binding handoff assumptions
- [Aura secret display spec](../../specs/aura-secret-display.md) — private widget-origin flags and direct-read limits
- [Forever forbidden-aspect consumers](../../specs/forever-forbidden-aspects.md) — base masks, animation extension, exclusions, and pending GREEN
- [Timer After callback dispatch](../../specs/timer-after-callback.md) — callback-created queue preservation and pending compiled GREEN
- [Duration text binding](../../specs/duration-text-binding.md) — Forever availability boundary and pending secret display handoff
- [Duration core](../../specs/duration-core.md) — authenticated secret timing storage and explicitly guessed lifecycle/access policy
- `/tmp/ellesmere-forever/traced-producers-gui-ledger.json` and `duration-secret-input-red.stderr` — 5/6 GUI boundary and wrapped-duration RED
- [Aura container options](../../specs/aura-container-options.md) — narrow public/secure publication and intentionally unexpanded native fields
- [Base spell aura secrecy](../../specs/spell-aura-secrecy.md) — shared classifier availability without Forever data-parity claim
- [SecondsFormatter configuration](../../specs/seconds-formatter-configuration.md) — cleanup restoration contract and pending compiled GREEN
- [Script-object environment crossings](../../specs/script-object-environments.md) — scoped direct-argument projection and pending Forever compiled GREEN
- `/tmp/ellesmere-forever/aura-audit-{load-graph,partitions,native-partition-diff,api-surface,duration-display,scheduling,nested-timers,gate-corrections,acceptance-boundary}.md` — read-only pinned-source and runtime-boundary audit artifacts
- `src/lua_api/env_runtime.rs:230-237`, `src/c_api/duration_text_binding.rs:7-18`, `src/lua_api/globals/enum_data/widget.rs:143-153`, and pinned Forever `Blizzard_AuraContainer` sources — audited producer gates and contracts

## See Also

- [[forever-clean-startup]] — distinct Blizzard-only sustained runtime proof
- [[forever-addon-comparison]] — broader cached-addon comparison, not Ellesmere acceptance
- [[client-profiles]] — Forever/Camelot profile routing
- [[bytecode-cache-growth]] — persisted-pack identity and storage bounds
- [[on-update-dirty]] — existing update-dispatch behavior and dirty scheduling
- [Forbidden-aspect inheritance](../../specs/forbidden-aspect-inheritance.md) — shared propagation and relationship constraints
